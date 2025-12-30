use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject,
        DrawTextW, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT, SetBkMode, TRANSPARENT,
    },
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::WindowsAndMessaging::*,
    core::s,
};

use crate::ime;

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

        // 画面サイズを取得して右上に配置
        let screen_width = GetSystemMetrics(SM_CXSCREEN);
        let window_width = 80;
        let window_height = 80;
        let x = screen_width - window_width - 50;
        let y = 50;

        let hwnd = CreateWindowExA(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
            window_class,
            s!("IME Indicator Overlay"),
            WS_POPUP,
            x,
            y,
            window_width,
            window_height,
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

        // 既存のタイマーをキャンセルしてから新しいタイマーを設定
        let _ = KillTimer(Some(window_handle), 1);
        SetTimer(Some(window_handle), 1, 3000, None);
    }
}

/// オーバーレイウィンドウを非表示
pub fn hide_overlay(window_handle: HWND) {
    unsafe {
        let _ = ShowWindow(window_handle, SW_HIDE);
        let _ = KillTimer(Some(window_handle), 1);
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
                println!("IME status changed");
                show_overlay(window);
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
                println!("WM_DESTROY");
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

        // IME状態に応じて背景色とテキストを変更
        let (bg_color, text) = match ime::get_ime_state() {
            Some(true) => (0x004080FF, "あ"), // オレンジ系背景で"あ"
            _ => (0x00808080, "A"),           // グレー背景で"A"
        };

        let brush = CreateSolidBrush(COLORREF(bg_color));
        FillRect(hdc, &ps.rcPaint, brush);
        let _ = DeleteObject(brush.into());

        // テキストの背景を透明に設定
        SetBkMode(hdc, TRANSPARENT);

        let mut rect = RECT::default();
        let _ = GetClientRect(window, &mut rect);

        // IME状態に応じたテキストを中央に表示
        let mut text_wide: Vec<u16> = text.encode_utf16().collect();
        DrawTextW(
            hdc,
            &mut text_wide,
            &mut rect,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        let _ = EndPaint(window, &ps);
    }
}
