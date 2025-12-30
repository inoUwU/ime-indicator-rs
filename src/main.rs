mod ime;
mod message_loop;
mod overlay;
mod tray;
mod utils;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() -> windows::core::Result<()> {
    // オーバーレイウィンドウを作成
    let window_handle = overlay::create_window()?;

    // IMEマネージャーを初期化
    ime::initialize(window_handle)?;

    // IMEフックを設定
    ime::setup_hooks()?;

    // システムトレイを作成
    let tray_manager = tray::TrayManager::new().expect("Failed to create system tray");

    // 終了フラグを作成
    let should_quit = Arc::new(AtomicBool::new(false));
    let should_quit_clone = Arc::clone(&should_quit);

    // Ctrl+Cハンドラを設定
    ctrlc::set_handler(move || {
        println!("Received interrupt signal, cleaning up...");
        should_quit_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    println!("IME Indicator is running. Right-click the system tray icon to quit.");

    // メッセージループを実行
    let result = message_loop::run(
        window_handle,
        tray_manager.quit_menu_id.clone(),
        should_quit,
    );

    // 正常終了時もフックをクリーンアップ
    ime::cleanup_hooks();

    result
}
