//! 設定の読み書きと構造体定義

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// オーバーレイの設定
    pub overlay: OverlayConfig,
}

/// オーバーレイ表示設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// X座標（右端からのオフセット）
    pub offset_x: i32,
    /// Y座標（上端からのオフセット）
    pub offset_y: i32,
    /// 幅
    pub width: i32,
    /// 高さ
    pub height: i32,
    /// IME ON時の背景色 (RGBA)
    pub color_on: [u8; 4],
    /// IME OFF時の背景色 (RGBA)
    pub color_off: [u8; 4],
    /// 表示時間（ミリ秒）
    pub display_duration_ms: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            overlay: OverlayConfig::default(),
        }
    }
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            offset_x: 50,
            offset_y: 50,
            width: 50,
            height: 50,
            color_on: [255, 165, 0, 255],    // オレンジ
            color_off: [128, 128, 128, 255], // グレー
            display_duration_ms: 3000,
        }
    }
}

/// 設定ファイルのパスを取得
pub fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config.toml")
}

/// 設定を読み込む
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => return config,
                Err(e) => eprintln!("Failed to parse config: {}", e),
            },
            Err(e) => eprintln!("Failed to read config: {}", e),
        }
    }
    AppConfig::default()
}

/// 設定を保存する
pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    let content = toml::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
