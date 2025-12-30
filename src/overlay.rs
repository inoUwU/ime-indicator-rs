use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject,
        DrawTextW, EndPaint, FillRect, InvalidateRect, PAINTSTRUCT, SetBkMode, TRANSPARENT,
    },
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::Accessibility::*,
    Win32::UI::Input::Ime::*,
    Win32::UI::WindowsAndMessaging::*,
    core::*,
};

// 安全なグローバル状態管理
static WINDOW_HANDLE: OnceLock<isize> = OnceLock::new(); // HWNDをisizeとして保存
static IME_STATE: OnceLock<Arc<Mutex<ImeState>>> = OnceLock::new();
static KEYBOARD_HOOK: OnceLock<isize> = OnceLock::new(); // HHOOKをisizeとして保存
static EVENT_HOOK: OnceLock<isize> = OnceLock::new(); // HWINEVENTHOOKをisizeとして保存
static LAST_CHECK_TIME: OnceLock<Arc<Mutex<Instant>>> = OnceLock::new(); // デバウンス用の最終チェック時刻

// IME状態を管理する構造体
#[derive(Debug, Clone)]
struct ImeState {
    is_active: bool,
    mode_description: String,
    should_show: bool,
    last_update: Instant,
}

impl std::ops::Deref for ImeState {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.should_show
    }
}

const WM_IME_STATUS_CHANGED: u32 = WM_USER + 1;
const IMC_GETOPENSTATUS: u32 = 5;
const DEBOUNCE_DURATION_MS: u64 = 50; // 50ms以内の連続チェックを防ぐ（反応性重視）
const IME_KEY_DEBOUNCE_MS: u64 = 20; // IME切り替えキー用の短い遅延

/// デバウンス機能付きでIME状態をチェックして更新
fn check_ime_status_and_update_with_debounce(window_handle: HWND) {
    check_ime_status_and_update_with_custom_debounce(window_handle, DEBOUNCE_DURATION_MS);
}

/// カスタム遅延でIME状態をチェックして更新
fn check_ime_status_and_update_with_custom_debounce(window_handle: HWND, debounce_ms: u64) {
    // デバウンスチェック - 短時間内の連続呼び出しを防ぐ
    if let Some(last_check) = LAST_CHECK_TIME.get() {
        if let Ok(mut last_time) = last_check.try_lock() {
            let now = Instant::now();
            if now.duration_since(*last_time) < Duration::from_millis(debounce_ms) {
                return; // 短時間内の連続呼び出しをスキップ
            }
            *last_time = now;
        } else {
            return; // ロックが取得できない場合はスキップ
        }
    }

    check_ime_status_and_update(window_handle);
}

// Low-Level Keyboard Hook Procedure
extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        if n_code >= 0 {
            // IME切り替えに関連するキーイベントのみを監視
            match w_param.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    // キーコードを取得してIME関連キーかどうかをチェック
                    let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
                    let vk_code = kbd_struct.vkCode;

                    // IME関連キー、英数字キー、または修飾キーの場合のみチェック
                    let should_check = matches!(vk_code,
                        // 半角/全角キー
                        0xF3 | 0xF4 |  // VK_DBE_SBCSCHAR, VK_DBE_DBCSCHAR
                        0x19 |        // VK_KANJI (半角/全角)
                        0x1C |        // VK_CONVERT (変換)
                        0x1D |        // VK_NONCONVERT (無変換)
                        // 修飾キー
                        0x12 |        // VK_MENU (Alt)
                        0x11 |        // VK_CONTROL (Ctrl)
                        0x10 |        // VK_SHIFT (Shift)
                        // 英数字キー範囲
                        0x30..=0x39 | // 0-9
                        0x41..=0x5A   // A-Z
                    );

                    // IME切り替えキー（半角/全角など）かどうかを判定
                    let is_ime_toggle_key = matches!(vk_code, 0xF3 | 0xF4 | 0x19);

                    if should_check && let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
                        let window_handle = HWND(window_handle_raw as *mut _);
                        // IME切り替えキーの場合は短い遅延、その他は通常の遅延
                        let debounce_time = if is_ime_toggle_key {
                            IME_KEY_DEBOUNCE_MS
                        } else {
                            DEBOUNCE_DURATION_MS
                        };
                        check_ime_status_and_update_with_custom_debounce(
                            window_handle,
                            debounce_time,
                        );
                    }
                }
                _ => {} // その他のイベントは無視
            }
        }
        // 次のフックプロシージャにチェーンする
        CallNextHookEx(None, n_code, w_param, l_param)
    }
}

// Window Event Hook Procedure
extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dw_ms_event_time: u32,
) {
    // フォーカス変更やウィンドウ状態変更の際にIME状態をチェック（デバウンス付き）
    match event {
        EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => {
            if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
                let window_handle = HWND(window_handle_raw as *mut _);
                check_ime_status_and_update_with_debounce(window_handle);
            }
        }
        _ => {}
    }
}

/// 現在のIME状態を取得する
fn get_current_ime_status() -> bool {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.is_invalid() {
            return false;
        }

        // IMEコンテキストを取得
        let ime_context = ImmGetDefaultIMEWnd(foreground_window);

        if !ime_context.is_invalid() {
            // IMEのオープン状態を取得
            let ime_open = SendMessageA(
                ime_context,
                WM_IME_CONTROL,
                WPARAM(IMC_GETOPENSTATUS as usize),
                LPARAM(0),
            );
            return ime_open.0 != 0;
        } else {
            // フォールバック：直接InputContextから取得を試みる
            let thread_id = GetWindowThreadProcessId(foreground_window, None);
            if thread_id != 0 {
                let himc = ImmGetContext(foreground_window);
                if !himc.is_invalid() {
                    let is_active = ImmGetOpenStatus(himc).as_bool();
                    let _ = ImmReleaseContext(foreground_window, himc);
                    return is_active;
                }
            }
        }

        false
    }
}

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
                    // IME監視タイマー（バックアップ目的、デバウンス付き）
                    check_ime_status_and_update_with_debounce(window);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                cleanup_hooks(); // フックを解除
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

        // 現在の実際のIME状態を取得して初期化
        let current_ime_status = get_current_ime_status();
        let initial_description = if current_ime_status { "あ" } else { "A" };
        let now = Instant::now();

        println!(
            "Initial IME status: {} ({})",
            if current_ime_status {
                "Active"
            } else {
                "Inactive"
            },
            initial_description
        );

        // デバウンス用の最終チェック時刻を初期化
        let _ = LAST_CHECK_TIME.set(Arc::new(Mutex::new(now)));

        let _ = IME_STATE.set(Arc::new(Mutex::new(ImeState {
            is_active: current_ime_status,
            mode_description: initial_description.to_string(),
            should_show: false,
            last_update: now,
        })));

        Ok(hwnd)
    }
}

/// グローバルIMEフックを設定します
pub fn setup_ime_hook(_window_handle: HWND, _display_duration: u32) -> windows::core::Result<()> {
    unsafe {
        println!("Setting up global IME hook...");

        let instance = GetModuleHandleA(None)?;
        println!("Got module handle: {:?}", instance);

        // Low-Level Keyboard Hookを設定
        println!("Setting up keyboard hook...");
        let keyboard_hook = SetWindowsHookExA(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            Some(instance.into()),
            0,
        );

        match keyboard_hook {
            Ok(hook) => {
                println!("Keyboard hook established successfully: {:?}", hook);
                let _ = KEYBOARD_HOOK.set(hook.0 as isize);
            }
            Err(e) => {
                println!("Failed to set keyboard hook: {:?}", e);
                return Err(e);
            }
        }

        // Window Event Hookを設定（フォーカス変更を監視）
        println!("Setting up window event hook...");
        let event_hook = SetWinEventHook(
            EVENT_OBJECT_FOCUS,
            EVENT_SYSTEM_FOREGROUND,
            Some(instance),
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if event_hook.is_invalid() {
            println!("Failed to set window event hook");
            // Window Event Hookの失敗は重要ではないので続行
        } else {
            println!(
                "Window event hook established successfully: {:?}",
                event_hook
            );
            let _ = EVENT_HOOK.set(event_hook.0 as isize);
        }

        // バックアップとして軽量なタイマーも設定（1秒間隔に短縮）
        if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
            let window_handle = HWND(window_handle_raw as *mut _);
            SetTimer(Some(window_handle), 999, 1000, None); // バックアップタイマー：1秒間隔
            println!("Backup timer set successfully");
        }

        println!("Global IME hook established successfully");

        // 初期状態を確認して表示が必要ならオーバーレイを表示
        if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
            let window_handle = HWND(window_handle_raw as *mut _);
            // 初期状態を表示（3秒間）
            show_overlay_window(window_handle);
        }

        Ok(())
    }
}

/// フックを解除します
pub fn cleanup_hooks() {
    unsafe {
        if let Some(&keyboard_hook_raw) = KEYBOARD_HOOK.get() {
            let keyboard_hook = HHOOK(keyboard_hook_raw as *mut _);
            let _ = UnhookWindowsHookEx(keyboard_hook);
        }

        if let Some(&event_hook_raw) = EVENT_HOOK.get() {
            let event_hook = HWINEVENTHOOK(event_hook_raw as *mut _);
            let _ = UnhookWinEvent(event_hook);
        }

        println!("Hooks cleaned up");
    }
}

/// IME状態をチェックして更新する（グローバルフック対応版）
fn check_ime_status_and_update(window_handle: HWND) {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.is_invalid() {
            return;
        }

        // IMEコンテキストを取得
        let ime_context = ImmGetDefaultIMEWnd(foreground_window);
        let mut is_ime_active = false;

        if !ime_context.is_invalid() {
            // IMEのオープン状態を取得
            let ime_open = SendMessageA(
                ime_context,
                WM_IME_CONTROL,
                WPARAM(IMC_GETOPENSTATUS as usize),
                LPARAM(0),
            );
            is_ime_active = ime_open.0 != 0;
        } else {
            // フォールバック：直接InputContextから取得を試みる
            let thread_id = GetWindowThreadProcessId(foreground_window, None);
            if thread_id != 0 {
                let himc = ImmGetContext(foreground_window);
                if !himc.is_invalid() {
                    is_ime_active = ImmGetOpenStatus(himc).as_bool();
                    let _ = ImmReleaseContext(foreground_window, himc);
                }
            }
        }

        // 現在の状態と比較して変更があった場合のみ更新
        if let Some(ime_state) = IME_STATE.get()
            && let Ok(mut state) = ime_state.try_lock()
            && state.is_active != is_ime_active
        {
            let now = Instant::now();
            state.is_active = is_ime_active;
            state.mode_description = if is_ime_active {
                "あ".to_string()
            } else {
                "A".to_string()
            };
            state.last_update = now;

            println!(
                "IME status changed: {}",
                if is_ime_active {
                    "Active (あ)"
                } else {
                    "Inactive (A)"
                }
            );

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
