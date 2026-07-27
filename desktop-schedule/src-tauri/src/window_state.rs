use parking_lot::Mutex;
use tauri::State;

/// 全局窗口锁定状态。锁定后窗口不可被拖拽。
#[derive(Default)]
pub struct WindowState {
    locked: Mutex<bool>,
}

impl WindowState {
    pub fn is_locked(&self) -> bool {
        *self.locked.lock()
    }

    pub fn toggle_locked(&self) -> bool {
        let mut g = self.locked.lock();
        *g = !*g;
        *g
    }
}

/// 从 Tauri State 获取锁定状态的便捷 trait
pub trait WindowStateExt {
    fn is_locked(&self) -> bool;
    fn toggle_locked(&self) -> bool;
}

impl WindowStateExt for State<'_, WindowState> {
    fn is_locked(&self) -> bool {
        WindowState::is_locked(self)
    }
    fn toggle_locked(&self) -> bool {
        WindowState::toggle_locked(self)
    }
}
