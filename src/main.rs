mod overlay;
mod utils;

use overlay::{cleanup_hooks, create_overlay_window, run_message_loop_with_tray, setup_ime_hook};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

fn main() -> windows::core::Result<()> {
    // オーバーレイウィンドウを作成
    let window_handle = create_overlay_window()?;

    // IMEグローバルフックを設定
    setup_ime_hook(window_handle, 3000)?;

    // システムトレイメニューを作成
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&quit_item).unwrap();

    // システムトレイアイコンを作成（シンプルな赤いアイコン）
    let icon_rgba = create_simple_icon();
    let icon = Icon::from_rgba(icon_rgba, 32, 32).unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("IME Indicator")
        .with_icon(icon)
        .build()
        .unwrap();

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

    // メッセージループを実行（トレイイベント処理を含む）
    let result = run_message_loop_with_tray(quit_item.id().clone(), should_quit);

    // 正常終了時もフックをクリーンアップ
    cleanup_hooks();

    result
}

/// 簡単なアイコンを作成（32x32の赤い円）
fn create_simple_icon() -> Vec<u8> {
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
