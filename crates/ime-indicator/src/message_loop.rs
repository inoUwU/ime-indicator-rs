use log::debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::menu::{MenuEvent, MenuId};
use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

/// メッセージループを実行（トレイイベント処理付き）
pub fn run(
    window_handle: HWND,
    quit_menu_id: MenuId,
    settings_menu_id: MenuId,
    should_quit: Arc<AtomicBool>,
) -> windows::core::Result<()> {
    unsafe {
        let mut message = MSG::default();

        loop {
            // ===メニューイベントをチェック===

            // 設定メニューがクリックされた場合
            if let Ok(event) = MenuEvent::receiver().try_recv()
                && event.id == settings_menu_id
            {
                debug!("Settings menu item clicked");
                // 設定GUIを別プロセスとして起動
                launch_settings_gui();
            }

            // Quitメニューがクリックされた場合、終了フラグを設定
            if let Ok(event) = MenuEvent::receiver().try_recv()
                && event.id == quit_menu_id
            {
                debug!("Quit menu item clicked");
                should_quit.store(true, Ordering::SeqCst);
            }

            // ============================

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

/// 設定GUIを起動
fn launch_settings_gui() {
    // 実行ファイルと同じディレクトリからsettings-gui.exeを探す
    let exe_path = std::env::current_exe().ok();
    let settings_exe = exe_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("settings-gui.exe"));

    if let Some(path) = settings_exe {
        debug!("Launching settings GUI: {:?}", path);
        match std::process::Command::new(&path).spawn() {
            Ok(_) => debug!("Settings GUI launched successfully"),
            Err(e) => {
                log::error!("Failed to launch settings GUI: {}", e);
                // フォールバック: カレントディレクトリから試す
                if let Err(e2) = std::process::Command::new("settings-gui.exe").spawn() {
                    log::error!("Fallback also failed: {}", e2);
                }
            }
        }
    } else {
        log::error!("Could not determine settings GUI path");
    }
}
