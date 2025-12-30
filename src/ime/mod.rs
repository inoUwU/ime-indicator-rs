pub mod state;

use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::{
    Win32::Foundation::*,
    Win32::UI::Accessibility::*,
    Win32::UI::Input::Ime::*,
    Win32::UI::WindowsAndMessaging::*,
};

use state::{ImeState, SharedImeState};

// 安全なグローバル状態管理
static WINDOW_HANDLE: OnceLock<isize> = OnceLock::new();
static IME_STATE: OnceLock<SharedImeState> = OnceLock::new();
static KEYBOARD_HOOK: OnceLock<isize> = OnceLock::new();
static EVENT_HOOK: OnceLock<isize> = OnceLock::new();
static LAST_CHECK_TIME: OnceLock<Arc<Mutex<Instant>>> = OnceLock::new();
static IS_CHECKING: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

const DEBOUNCE_DURATION_MS: u64 = 50;
const IME_KEY_DEBOUNCE_MS: u64 = 20;
pub const WM_IME_STATUS_CHANGED: u32 = WM_USER + 1;
const IMC_GETOPENSTATUS: u32 = 5;

/// IMEマネージャーを初期化
pub fn initialize(window_handle: HWND) -> windows::core::Result<()> {
    let _ = WINDOW_HANDLE.set(window_handle.0 as isize);
    let _ = LAST_CHECK_TIME.set(Arc::new(Mutex::new(Instant::now())));
    let _ = IS_CHECKING.set(Arc::new(Mutex::new(false)));

    let current_ime_status = get_current_ime_status();
    let _ = IME_STATE.set(Arc::new(Mutex::new(ImeState::new(current_ime_status))));

    println!(
        "Initial IME status: {}",
        if current_ime_status { "Active (あ)" } else { "Inactive (A)" }
    );

    Ok(())
}

/// グローバルIMEフックを設定
pub fn setup_hooks() -> windows::core::Result<()> {
    unsafe {
        println!("Setting up global IME hook...");

        let instance = windows::Win32::System::LibraryLoader::GetModuleHandleA(None)?;
        println!("Got module handle: {:?}", instance);

        // Low-Level Keyboard Hook
        println!("Setting up keyboard hook...");
        let keyboard_hook = SetWindowsHookExA(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            Some(instance.into()),
            0,
        )?;

        println!("Keyboard hook established: {:?}", keyboard_hook);
        let _ = KEYBOARD_HOOK.set(keyboard_hook.0 as isize);

        // Window Event Hook
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
        } else {
            println!("Window event hook established: {:?}", event_hook);
            let _ = EVENT_HOOK.set(event_hook.0 as isize);
        }

        // バックアップタイマー
        if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
            let window_handle = HWND(window_handle_raw as *mut _);
            SetTimer(Some(window_handle), 999, 1000, None);
            println!("Backup timer set");
        }

        println!("Global IME hook established successfully");

        // 初期表示
        if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
            let window_handle = HWND(window_handle_raw as *mut _);
            crate::overlay::show_overlay(window_handle);
        }

        Ok(())
    }
}

/// フックをクリーンアップ
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

/// IME状態をチェックして更新
pub fn check_ime_status_with_debounce(window_handle: HWND) {
    check_ime_status_with_custom_debounce(window_handle, DEBOUNCE_DURATION_MS);
}

/// カスタム遅延でIME状態をチェック
fn check_ime_status_with_custom_debounce(window_handle: HWND, debounce_ms: u64) {
    // 処理中フラグをチェック
    if let Some(is_checking) = IS_CHECKING.get() {
        if let Ok(mut checking) = is_checking.try_lock() {
            if *checking {
                return;
            }
            *checking = true;
        } else {
            return;
        }
    }

    // デバウンスチェック
    let should_process = if let Some(last_check) = LAST_CHECK_TIME.get() {
        if let Ok(mut last_time) = last_check.try_lock() {
            let now = Instant::now();
            if now.duration_since(*last_time) < Duration::from_millis(debounce_ms) {
                false
            } else {
                *last_time = now;
                true
            }
        } else {
            false
        }
    } else {
        false
    };

    if should_process {
        check_and_notify_ime_change(window_handle);
    }

    // 処理中フラグをリセット
    if let Some(is_checking) = IS_CHECKING.get()
        && let Ok(mut checking) = is_checking.try_lock()
    {
        *checking = false;
    }
}

/// IME状態を取得
pub fn get_current_ime_status() -> bool {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.is_invalid() {
            return false;
        }

        let ime_context = ImmGetDefaultIMEWnd(foreground_window);

        if !ime_context.is_invalid() {
            let ime_open = SendMessageA(
                ime_context,
                WM_IME_CONTROL,
                WPARAM(IMC_GETOPENSTATUS as usize),
                LPARAM(0),
            );
            return ime_open.0 != 0;
        } else {
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

/// IME状態変更をチェックして通知
fn check_and_notify_ime_change(window_handle: HWND) {
    let is_ime_active = get_current_ime_status();

    if let Some(ime_state) = IME_STATE.get()
        && let Ok(mut state) = ime_state.try_lock()
        && state.is_active != is_ime_active
    {
        state.update(is_ime_active);

        println!(
            "IME status changed: {}",
            if is_ime_active { "Active (あ)" } else { "Inactive (A)" }
        );

        drop(state);

        unsafe {
            let _ = PostMessageA(
                Some(window_handle),
                WM_IME_STATUS_CHANGED,
                WPARAM(0),
                LPARAM(0),
            );
        }
    }
}

/// IME状態を取得（外部から参照用）
pub fn get_ime_state() -> Option<bool> {
    IME_STATE
        .get()
        .and_then(|state| state.try_lock().ok())
        .map(|state| state.is_active)
}

// Low-Level Keyboard Hook
extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        if n_code >= 0 {
            match w_param.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
                    let vk_code = kbd_struct.vkCode;

                    let should_check = matches!(
                        vk_code,
                        0xF3 | 0xF4 | 0x19 | 0x1C | 0x1D | 0x12 | 0x11 | 0x10 | 0x30..=0x39
                            | 0x41..=0x5A
                    );

                    let is_ime_toggle_key = matches!(vk_code, 0xF3 | 0xF4 | 0x19);

                    if should_check && let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
                        let window_handle = HWND(window_handle_raw as *mut _);
                        let debounce_time = if is_ime_toggle_key {
                            IME_KEY_DEBOUNCE_MS
                        } else {
                            DEBOUNCE_DURATION_MS
                        };
                        check_ime_status_with_custom_debounce(window_handle, debounce_time);
                    }
                }
                _ => {}
            }
        }
        CallNextHookEx(None, n_code, w_param, l_param)
    }
}

// Window Event Hook
extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dw_ms_event_time: u32,
) {
    match event {
        EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => {
            if let Some(&window_handle_raw) = WINDOW_HANDLE.get() {
                let window_handle = HWND(window_handle_raw as *mut _);
                check_ime_status_with_custom_debounce(window_handle, DEBOUNCE_DURATION_MS);
            }
        }
        _ => {}
    }
}
