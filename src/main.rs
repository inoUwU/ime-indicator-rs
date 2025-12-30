mod overlay;
mod utils;

use overlay::{create_overlay_window, run_message_loop, setup_ime_hook};

// use tray_icon::{
//     Icon, TrayIconBuilder, TrayIconEvent,
//     menu::{Menu, MenuEvent},
// };

fn main() -> windows::core::Result<()> {
    // オーバーレイウィンドウを作成
    let window_handle = create_overlay_window()?;

    // IMEフックを設定（3秒後に自動的に非表示にする）
    setup_ime_hook(window_handle, 3000)?;

    // メッセージループを実行
    run_message_loop()?;

    Ok(())

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
