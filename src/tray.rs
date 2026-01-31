use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// システムトレイを管理する構造体
pub struct TrayManager {
    _icon: TrayIcon,
    pub quit_menu_id: MenuId,
    pub settings_menu_id: MenuId,
}

impl TrayManager {
    /// 新しいシステムトレイマネージャーを作成
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // メニューを作成
        let tray_menu = Menu::new();

        // ※登録した順番でメニューに表示される

        // 設定メニューのシステムトレイを設定
        let setting_item = MenuItem::new("Settings", true, None);

        let settings_menu_id = setting_item.id().clone();
        tray_menu.append(&setting_item)?;

        // 終了メニューアイテムを追加
        let quit_item = MenuItem::new("Quit", true, None);
        let quit_menu_id = quit_item.id().clone();
        tray_menu.append(&quit_item)?;

        // アイコンを作成
        let icon = Icon::from_path("assets/icon.ico", None).expect("Failed to load ico");

        // トレイアイコンをビルド
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("IME Indicator")
            .with_icon(icon)
            .build()?;

        Ok(TrayManager {
            _icon: tray_icon,
            quit_menu_id,
            settings_menu_id,
        })
    }
}
