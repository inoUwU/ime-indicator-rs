//! 設定の読み書きと構造体定義

use core::fmt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// オーバーレイの設定
    pub overlay: OverlayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayPosition {
    TopLeft,
    TopRight,
    Center,
    BottomRight,
    BottomLeft,
}

impl fmt::Display for DisplayPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TopLeft => write!(f, "TopLeft"),
            Self::TopRight => write!(f, "TopRight"),
            Self::Center => write!(f, "Center"),
            Self::BottomRight => write!(f, "BottomRight"),
            Self::BottomLeft => write!(f, "BottomLeft"),
        }
    }
}

/// オーバーレイ表示設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    pub display_pos: DisplayPosition,
    /// IME ON時の背景色 (RGBA)
    pub color_on: [u8; 4],
    /// IME OFF時の背景色 (RGBA)
    pub color_off: [u8; 4],
    /// 表示時間（ミリ秒）
    pub display_duration_ms: u32,
    /// TODO:マルチモニタ対応:有効化するとウィンドウが表示されているモニタにオーバーレイを表示
    pub enable_multi_monitor: bool,
}

impl OverlayConfig {
    /// RGBA配列からCOLOREREF形式に変換（Windows GDI用）
    /// COLORREF は 0x00BBGGRR 形式 (Blue-Green-Red)
    pub fn color_on_as_colorref(&self) -> u32 {
        let [r, g, b, _a] = self.color_on;
        ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
    }

    /// RGBA配列からCOLOREREF形式に変換（Windows GDI用）
    /// COLORREF は 0x00BBGGRR 形式 (Blue-Green-Red)
    pub fn color_off_as_colorref(&self) -> u32 {
        let [r, g, b, _a] = self.color_off;
        ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
    }
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            display_pos: DisplayPosition::Center,
            color_on: [255, 165, 0, 255],    // オレンジ
            color_off: [128, 128, 128, 255], // グレー
            display_duration_ms: 3000,
            enable_multi_monitor: false,
        }
    }
}

/// 設定ファイルのパスを取得
pub fn config_path() -> PathBuf {
    // TODO:XDG Base Directory 準拠
    // let config_dir = std::env::home_dir().and;

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    exe_dir.join("ime-indicator-config.toml")
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
