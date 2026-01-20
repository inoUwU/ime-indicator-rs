use log::debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::menu::{MenuEvent, MenuId};
use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

/// メッセージループを実行（トレイイベント処理付き）
pub fn run(
    window_handle: HWND,
    quit_menu_id: MenuId,
    should_quit: Arc<AtomicBool>,
) -> windows::core::Result<()> {
    unsafe {
        let mut message = MSG::default();

        loop {
            // メニューイベントをチェック
            if let Ok(event) = MenuEvent::receiver().try_recv()
                && event.id == quit_menu_id
            {
                debug!("Quit menu item clicked");
                should_quit.store(true, Ordering::SeqCst);
            }

            // 終了フラグをチェック
            if should_quit.load(Ordering::SeqCst) {
                let _ = PostMessageA(Some(window_handle), WM_CLOSE, WPARAM(0), LPARAM(0));
                break;
            }

            // Windowsメッセージを処理（非ブロッキング）
            if PeekMessageA(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                if message.message == WM_QUIT {
                    break;
                }
                DispatchMessageA(&message);
            } else {
                // メッセージがない場合は少し待機
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        Ok(())
    }
}
