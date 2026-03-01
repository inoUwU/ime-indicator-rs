use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{
        BeginPaint, CreateFontW, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER,
        DeleteObject, DrawTextW, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT, SelectObject,
        SetBkMode, TRANSPARENT,
    },
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*,
    core::{s, w},
};

use crate::ime;
use log::debug;

static WINDOW_WIDTH: i32 = 80;
static WINDOW_HEIGHT: i32 = 80;

/// 設定から表示位置を計算
fn calculate_position() -> (i32, i32) {
    unsafe {
        // 画面サイズを取得
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let screen_height = GetSystemMetrics(SM_CYSCREEN);

        // 設定から表示位置を取得
        if let Some(config) = ime::get_config() {
            if let Ok(config) = config.lock() {
                match config.overlay.display_pos {
                    shared::DisplayPosition::TopLeft => (50, 50),
                    shared::DisplayPosition::TopRight => (screen_width - WINDOW_WIDTH - 50, 50),
                    shared::DisplayPosition::Center => (
                        (screen_width - WINDOW_WIDTH) / 2,
                        (screen_height - WINDOW_HEIGHT) / 2,
                    ),
                    shared::DisplayPosition::BottomLeft => (50, screen_height - WINDOW_HEIGHT - 50),
                    shared::DisplayPosition::BottomRight => (
                        screen_width - WINDOW_WIDTH - 50,
                        screen_height - WINDOW_HEIGHT - 50,
                    ),
                }
            } else {
                // ロック取得失敗時はデフォルト位置
                (
                    (screen_width - WINDOW_WIDTH) / 2,
                    (screen_height - WINDOW_HEIGHT) / 2,
                )
            }
        } else {
            // 設定がない場合はデフォルト位置
            (
                (screen_width - WINDOW_WIDTH) / 2,
                (screen_height - WINDOW_HEIGHT) / 2,
            )
        }
    }
}

/// オーバーレイウィンドウを作成
pub fn create_window() -> windows::core::Result<HWND> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        let window_class = s!("ime_indicator_overlay");

        let wc = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance.into(),
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&wc);
        debug_assert!(atom != 0);

        // 設定から表示位置を計算
        let (x, y) = calculate_position();

        let hwnd = CreateWindowExA(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            window_class,
            s!("IME Indicator Overlay"),
            WS_POPUP,
            x,
            y,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // 半透明設定（アルファ値180で30%透明）
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 180, LWA_ALPHA)?;

        Ok(hwnd)
    }
}

/// オーバーレイウィンドウを表示
pub fn show_overlay(window_handle: HWND) {
    unsafe {
        let _ = ShowWindow(window_handle, SW_SHOWNA);
        let _ = InvalidateRect(Some(window_handle), None, true);

        // 設定から表示時間を取得
        let display_duration = if let Some(config) = ime::get_config() {
            if let Ok(config) = config.lock() {
                config.overlay.display_duration_ms
            } else {
                3000 // デフォルト値
            }
        } else {
            3000 // デフォルト値
        };

        // 既存のタイマーをキャンセルしてから新しいタイマーを設定
        let _ = KillTimer(Some(window_handle), 1);
        SetTimer(Some(window_handle), 1, display_duration, None);
    }
}

/// オーバーレイウィンドウを非表示
pub fn hide_overlay(window_handle: HWND) {
    unsafe {
        let _ = ShowWindow(window_handle, SW_HIDE);
        let _ = KillTimer(Some(window_handle), 1);
    }
}

/// ウィンドウ位置を更新（設定変更時に呼び出し）
pub fn update_window_position(window_handle: HWND) {
    unsafe {
        let (x, y) = calculate_position();

        // SetWindowPos を使ってウィンドウ位置を更新
        // SWP_NOSIZE: サイズは変更しない
        // SWP_NOZORDER: Z順序は変更しない
        // SWP_NOACTIVATE: ウィンドウをアクティブにしない
        let _ = SetWindowPos(
            window_handle,
            None,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );

        debug!("Window position updated to: ({}, {})", x, y);
    }
}

/// ウィンドウプロシージャ
extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                paint_window(window);
                LRESULT(0)
            }
            ime::WM_IME_STATUS_CHANGED => {
                debug!("IME status changed");
                show_overlay(window);
                LRESULT(0)
            }
            ime::WM_CONFIG_CHANGED => {
                debug!("Configuration changed, reloading...");
                ime::reload_config();
                // ウィンドウ位置を更新
                update_window_position(window);
                // 設定画面終了後、再読込した設定でオーバーレイを即時表示
                show_overlay(window);
                // 設定変更後、オーバーレイを再描画して新しい色を適用
                let _ = InvalidateRect(Some(window), None, true);
                LRESULT(0)
            }
            WM_TIMER => {
                if wparam.0 == 1 {
                    // 表示タイマー（3秒後の自動非表示）
                    hide_overlay(window);
                } else if wparam.0 == 999 {
                    // IME監視タイマー（バックアップ）
                    ime::check_ime_status_with_debounce(window);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                debug!("WM_DESTROY");
                ime::cleanup_hooks();
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

/// ウィンドウを描画
fn paint_window(window: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(window, &mut ps);

        // 読みやすい大きなボールドフォントを設定
        let font = CreateFontW(
            48,                                                    // フォントサイズ（高さ）
            0,                                                     // フォント幅（0で自動）
            0,                                                     // エスケープ角度
            0,                                                     // 配置角度
            windows::Win32::Graphics::Gdi::FW_BOLD.0 as i32,       // 太さ（ボールド）
            0,                                                     // イタリック
            0,                                                     // 下線
            0,                                                     // 取消線
            windows::Win32::Graphics::Gdi::DEFAULT_CHARSET,        // 文字セット
            windows::Win32::Graphics::Gdi::OUT_CHARACTER_PRECIS,   // 出力精度
            windows::Win32::Graphics::Gdi::CLIP_CHARACTER_PRECIS,  // クリップ精度
            windows::Win32::Graphics::Gdi::DEFAULT_QUALITY,        // 品質
            windows::Win32::Graphics::Gdi::DEFAULT_PITCH.0.into(), // ピッチとファミリー
            w!("Yu Gothic UI"),                                    // windows11標準フォント
        );
        let old_font = SelectObject(hdc, font.into());

        // IME状態と設定に応じて背景色とテキストを変更
        let (bg_color, text) = if let Some(config) = ime::get_config() {
            if let Ok(config) = config.lock() {
                match ime::get_ime_state() {
                    Some(true) => (config.overlay.color_on_as_colorref(), "あ"),
                    _ => (config.overlay.color_off_as_colorref(), "A"),
                }
            } else {
                // ロック取得失敗時はデフォルト値
                match ime::get_ime_state() {
                    Some(true) => (0x004080FF, "あ"),
                    _ => (0x00808080, "A"),
                }
            }
        } else {
            // 設定がない場合はデフォルト値
            match ime::get_ime_state() {
                Some(true) => (0x004080FF, "あ"),
                _ => (0x00808080, "A"),
            }
        };

        let brush = CreateSolidBrush(COLORREF(bg_color));
        FillRect(hdc, &ps.rcPaint, brush);
        let _ = DeleteObject(brush.into());

        // テキストの背景を透明に設定
        SetBkMode(hdc, TRANSPARENT);

        let mut rect = RECT::default();
        let _ = GetClientRect(window, &mut rect);

        // テキスト色を白に設定
        windows::Win32::Graphics::Gdi::SetTextColor(hdc, COLORREF(0x00FFFFFF));

        // IME状態に応じたテキストを中央に表示
        let mut text_wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut text_wide,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        // フォントを復元してリソースをクリーンアップ
        let _ = SelectObject(hdc, old_font);
        let _ = DeleteObject(font.into());

        let _ = EndPaint(window, &ps);
    }
}
