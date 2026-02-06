use log::debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::menu::{MenuEvent, MenuId};
use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

use crate::ime;

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

            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == settings_menu_id {
                    debug!("Settings menu item clicked");
                    // 設定GUIを別プロセスとして起動
                    launch_settings_gui(window_handle);
                } else if event.id == quit_menu_id {
                    debug!("Quit menu item clicked");
                    should_quit.store(true, Ordering::SeqCst);
                } else {
                    debug!("Unhandled menu item clicked: {:?}", event.id);
                }
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
fn launch_settings_gui(window_handle: HWND) {
    // 実行ファイルと同じディレクトリからsettings-gui.exeを探す
    let exe_path = std::env::current_exe().ok();
    let settings_exe = exe_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|dir| dir.join("settings-gui.exe"));

    if let Some(path) = settings_exe {
        debug!("Launching settings GUI: {:?}", path);

        // HWNDは Send ではないため、生のポインタに変換
        let window_handle_raw = window_handle.0 as isize;

        match std::process::Command::new(&path).spawn() {
            Ok(mut child) => {
                debug!("Settings GUI launched successfully");

                // プロセスIDを取得
                let pid = child.id();
                debug!("Settings GUI process ID: {}", pid);

                // 別スレッドでプロセス終了を監視
                std::thread::spawn(move || {
                    // プロセスの終了を待機
                    let _ = child.wait();
                    debug!("Settings GUI process has terminated");

                    // 生のポインタからHWNDを復元
                    let window_handle = HWND(window_handle_raw as *mut _);

                    // 設定変更通知をメインウィンドウに送信
                    unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                            Some(window_handle),
                            ime::WM_CONFIG_CHANGED,
                            WPARAM(0),
                            LPARAM(0),
                        );
                    }
                });
            }
            Err(e) => {
                log::error!("Failed to launch settings GUI: {}", e);
                // フォールバック: カレントディレクトリから試す
                if let Ok(mut child) = std::process::Command::new("settings-gui.exe").spawn() {
                    debug!("Settings GUI launched from fallback path");

                    std::thread::spawn(move || {
                        let _ = child.wait();
                        debug!("Settings GUI process (fallback) has terminated");

                        let window_handle = HWND(window_handle_raw as *mut _);

                        unsafe {
                            let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                                Some(window_handle),
                                ime::WM_CONFIG_CHANGED,
                                WPARAM(0),
                                LPARAM(0),
                            );
                        }
                    });
                } else {
                    log::error!("Fallback also failed");
                }
            }
        }
    } else {
        log::error!("Could not determine settings GUI path");
    }
}
