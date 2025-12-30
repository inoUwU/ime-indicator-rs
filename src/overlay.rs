use std::sync::{Arc, Mutex, OnceLock};
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject,
        DrawTextW, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT, SetBkMode, TRANSPARENT,
    },
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::Input::Ime::*,
    Win32::UI::WindowsAndMessaging::*,
    core::*,
};

// 安全なグローバル状態管理
static WINDOW_HANDLE: OnceLock<isize> = OnceLock::new(); // HWNDをisizeとして保存
static IME_STATE: OnceLock<Arc<Mutex<ImeState>>> = OnceLock::new();

// IME状態を管理する構造体
#[derive(Debug, Clone)]
struct ImeState {
    is_active: bool,
    mode_description: String,
    should_show: bool,
}

const WM_IME_STATUS_CHANGED: u32 = WM_USER + 1;
const WM_HIDE_OVERLAY: u32 = WM_USER + 2;
const IMC_GETOPENSTATUS: u32 = 5;

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);

                // IME状態に応じて背景色とテキストを変更
                let (bg_color, text) = if let Some(ime_state) = IME_STATE.get() {
                    if let Ok(state) = ime_state.try_lock() {
                        if state.is_active {
                            (0x004080FF, "あ") // オレンジ系背景で"あ"
                        } else {
                            (0x00808080, "A") // グレー背景で"A"
                        }
                    } else {
                        (0x00808080, "A")
                    }
                } else {
                    (0x00808080, "A")
                };

                let brush = CreateSolidBrush(COLORREF(bg_color));
                FillRect(hdc, &ps.rcPaint, brush);
                // ブラシリソースを解放
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
                LRESULT(0)
            }
            WM_IME_STATUS_CHANGED => {
                // IME状態が変更された時の処理
                println!("IME status changed");
                show_overlay_window(window);
                LRESULT(0)
            }
            WM_TIMER => {
                if wparam.0 == 1 {
                    // 表示タイマー（3秒後の自動非表示）
                    hide_overlay_window(window);
                } else if wparam.0 == 999 {
                    // IME監視タイマー（100ms間隔での状態チェック）
                    check_ime_status_and_update(window);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}

/// 透明オーバーレイウィンドウを作成します
pub fn create_overlay_window() -> windows::core::Result<HWND> {
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
        let x = screen_width - window_width - 50; // 右端から50px離れた位置
        let y = 50; // 上端から50px離れた位置

        let hwnd = CreateWindowExA(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            window_class,
            s!("IME Indicator Overlay"),
            WS_POPUP,      // 初期状態では非表示
            x,             // x position (top-right)
            y,             // y position (top-right)
            window_width,  // width
            window_height, // height
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        // 半透明設定（アルファ値180で30%透明）
        SetLayeredWindowAttributes(hwnd, COLORREF(0), 180, LWA_ALPHA)?;

        // グローバル変数にウィンドウハンドルを保存
        let _ = WINDOW_HANDLE.set(hwnd.0 as isize);

        // IME状態を初期化
        let _ = IME_STATE.set(Arc::new(Mutex::new(ImeState {
            is_active: false,
            mode_description: "A".to_string(),
            should_show: false,
        })));

        Ok(hwnd)
    }
}

/// IME監視タイマーを設定します（より安全な方式）
pub fn setup_ime_hook(_window_handle: HWND, _display_duration: u32) -> windows::core::Result<()> {
    unsafe {
        // タイマーを使用してIME状態を定期的にチェック（100ms間隔）
        if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
            let window_handle = HWND(window_handle_raw as *mut _);
            SetTimer(Some(window_handle), 999, 100, None); // ID=999でIME監視タイマー
        }

        println!("IME monitoring timer started successfully");
        Ok(())
    }
}

/// IME状態をチェックして更新する
fn check_ime_status_and_update(window_handle: HWND) {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.is_invalid() {
            return;
        }

        let _thread_id = GetWindowThreadProcessId(foreground_window, None);
        let ime_context = ImmGetDefaultIMEWnd(foreground_window);

        if !ime_context.is_invalid() {
            // IMEのオープン状態を取得
            let ime_open = SendMessageA(
                ime_context,
                WM_IME_CONTROL,
                WPARAM(IMC_GETOPENSTATUS as usize),
                LPARAM(0),
            );
            let is_ime_active = ime_open.0 != 0;

            // 現在の状態と比較して変更があった場合のみ更新
            if let Some(ime_state) = IME_STATE.get() {
                if let Ok(mut state) = ime_state.try_lock() {
                    if state.is_active != is_ime_active {
                        state.is_active = is_ime_active;
                        state.mode_description = if is_ime_active {
                            "あ".to_string()
                        } else {
                            "A".to_string()
                        };

                        drop(state); // 明示的にlockを解放

                        // ウィンドウに状態変更を通知
                        let _ = PostMessageA(
                            Some(window_handle),
                            WM_IME_STATUS_CHANGED,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                }
            }
        }
    }
}

/// オーバーレイウィンドウを表示する
fn show_overlay_window(window_handle: HWND) {
    unsafe {
        let _ = ShowWindow(window_handle, SW_SHOW);
        let _ = InvalidateRect(Some(window_handle), None, true);

        // 3秒後に非表示にするタイマーを設定
        SetTimer(Some(window_handle), 1, 3000, None);
    }
}

/// オーバーレイウィンドウを非表示にする
fn hide_overlay_window(window_handle: HWND) {
    unsafe {
        let _ = ShowWindow(window_handle, SW_HIDE);
        let _ = KillTimer(Some(window_handle), 1); // タイマーを停止
    }
}

/// メッセージループを実行します
pub fn run_message_loop() -> windows::core::Result<()> {
    unsafe {
        let mut message = MSG::default();

        while GetMessageA(&mut message, None, 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}
