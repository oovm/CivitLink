use gg_core::GResult;

use crate::lifecycle::MobileLifecycle;

/// 移动平台生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// 应用正在运行
    Running,
    /// 应用已暂停（进入后台）
    Paused,
    /// 应用已销毁
    Destroyed,
}

/// 移动平台生命周期管理器
///
/// 实现 `MobileLifecycle` trait，维护应用生命周期状态，
/// 并提供外部接口供原生桥接层（iOS/Android JNI）调用。
pub struct MobileLifecycleManager {
    /// 当前生命周期状态
    state: LifecycleState,
}

impl MobileLifecycleManager {
    /// 创建新的生命周期管理器
    pub fn new() -> Self {
        Self {
            state: LifecycleState::Running,
        }
    }

    /// 获取当前生命周期状态
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// 检查应用是否正在运行
    pub fn is_running(&self) -> bool {
        self.state == LifecycleState::Running
    }

    /// 检查应用是否已暂停
    pub fn is_paused(&self) -> bool {
        self.state == LifecycleState::Paused
    }

    /// 检查应用是否已销毁
    pub fn is_destroyed(&self) -> bool {
        self.state == LifecycleState::Destroyed
    }
}

impl MobileLifecycle for MobileLifecycleManager {
    fn on_pause(&mut self) -> GResult<()> {
        self.state = LifecycleState::Paused;
        Ok(())
    }

    fn on_resume(&mut self) -> GResult<()> {
        self.state = LifecycleState::Running;
        Ok(())
    }

    fn on_destroy(&mut self) -> GResult<()> {
        self.state = LifecycleState::Destroyed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::MobileLifecycle;

    #[test]
    fn test_initial_state_is_running() {
        let manager = MobileLifecycleManager::new();
        assert!(manager.is_running());
        assert!(!manager.is_paused());
        assert!(!manager.is_destroyed());
    }

    #[test]
    fn test_on_pause() {
        let mut manager = MobileLifecycleManager::new();
        manager.on_pause().unwrap();
        assert!(!manager.is_running());
        assert!(manager.is_paused());
        assert!(!manager.is_destroyed());
        assert_eq!(manager.state(), LifecycleState::Paused);
    }

    #[test]
    fn test_on_resume() {
        let mut manager = MobileLifecycleManager::new();
        manager.on_pause().unwrap();
        assert!(manager.is_paused());
        manager.on_resume().unwrap();
        assert!(manager.is_running());
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_on_destroy() {
        let mut manager = MobileLifecycleManager::new();
        manager.on_destroy().unwrap();
        assert!(manager.is_destroyed());
        assert!(!manager.is_running());
        assert_eq!(manager.state(), LifecycleState::Destroyed);
    }

    #[test]
    fn test_lifecycle_transitions() {
        let mut manager = MobileLifecycleManager::new();
        assert!(manager.is_running());
        manager.on_pause().unwrap();
        assert!(manager.is_paused());
        manager.on_resume().unwrap();
        assert!(manager.is_running());
        manager.on_destroy().unwrap();
        assert!(manager.is_destroyed());
    }
}
