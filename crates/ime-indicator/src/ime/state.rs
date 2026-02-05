use std::sync::{Arc, Mutex};
use std::time::Instant;

/// IME状態を管理する構造体
#[derive(Debug, Clone)]
pub struct ImeState {
    pub is_active: bool,
    pub mode_description: String,
    pub should_show: bool,
    pub last_update: Instant,
}

impl ImeState {
    /// 新しいIME状態を作成
    pub fn new(is_active: bool) -> Self {
        let mode_description = if is_active { "あ" } else { "A" };
        ImeState {
            is_active,
            mode_description: mode_description.to_string(),
            should_show: false,
            last_update: Instant::now(),
        }
    }

    /// IME状態を更新
    pub fn update(&mut self, is_active: bool) {
        self.is_active = is_active;
        self.mode_description = if is_active {
            "あ".to_string()
        } else {
            "A".to_string()
        };
        self.last_update = Instant::now();
    }
}

impl std::ops::Deref for ImeState {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.should_show
    }
}

/// スレッドセーフなIME状態
pub type SharedImeState = Arc<Mutex<ImeState>>;
