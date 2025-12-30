use tray_icon::menu::{Menu, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// システムトレイを管理する構造体
pub struct TrayManager {
    _icon: TrayIcon,
    pub quit_menu_id: MenuId,
}

impl TrayManager {
    /// 新しいシステムトレイマネージャーを作成
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // メニューを作成
        let tray_menu = Menu::new();
        let quit_item = MenuItem::new("Quit", true, None);
        let quit_menu_id = quit_item.id().clone();
        tray_menu.append(&quit_item)?;

        // アイコンを作成
        let icon_rgba = Self::create_icon();
        let icon = Icon::from_rgba(icon_rgba, 32, 32)?;

        // トレイアイコンをビルド
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("IME Indicator")
            .with_icon(icon)
            .build()?;

        Ok(TrayManager {
            _icon: tray_icon,
            quit_menu_id,
        })
    }

    /// 簡単なアイコンを作成（32x32の赤い円）
    fn create_icon() -> Vec<u8> {
        let size = 32;
        let mut rgba = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - 16.0;
                let dy = y as f32 - 16.0;
                let distance = (dx * dx + dy * dy).sqrt();

                let idx = (y * size + x) * 4;
                if distance <= 14.0 {
                    rgba[idx] = 255; // R
                    rgba[idx + 1] = 100; // G
                    rgba[idx + 2] = 100; // B
                    rgba[idx + 3] = 255; // A
                } else {
                    rgba[idx + 3] = 0; // 透明
                }
            }
        }

        rgba
    }
}
