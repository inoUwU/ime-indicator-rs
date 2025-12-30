mod overlay;
mod utils;

use overlay::{cleanup_hooks, create_overlay_window, run_message_loop, setup_ime_hook};

fn main() -> windows::core::Result<()> {
    // オーバーレイウィンドウを作成
    let window_handle = create_overlay_window()?;

    // IMEグローバルフックを設定
    setup_ime_hook(window_handle, 3000)?;

    // Ctrl+Cハンドラを設定
    ctrlc::set_handler(move || {
        println!("Received interrupt signal, cleaning up...");
        cleanup_hooks();
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    println!("IME Indicator is running with global hooks. Press Ctrl+C to exit.");

    // メッセージループを実行
    let result = run_message_loop();

    // 正常終了時もフックをクリーンアップ
    cleanup_hooks();

    result

    // TODO: add icon from icon file.
    // TODO: add menu items to tray menu. quit , positon etc.
    // TODO: handle menu item click events.
    // TODO: finally make a ime indicator. e.g. show current ime mode use windows api and windows hooks.

    /*
    let tray_menu = Menu::new();
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("ime-indeicator")
        .with_icon(Icon::from_rgba(vec![255u8; 32 * 32 * 4], 32, 32).unwrap())
        .build()
        .unwrap();

    tray_icon.set_visible(true).unwrap();
    loop {
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {:?}", event);
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            println!("menu event: {:?}", event);
        }
    }
    */
}
