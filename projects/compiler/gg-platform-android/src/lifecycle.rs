use gg_core::{GError, GResult};

/// Android 平台生命周期
///
/// 为 Android 平台提供生命周期管理的具体实现。
pub struct AndroidLifecycle {
    state: crate::lifecycle_manager::LifecycleState,
}

impl AndroidLifecycle {
    /// 创建 Android 生命周期实例
    pub fn new() -> Self {
        Self { state: crate::lifecycle_manager::LifecycleState::Created }
    }

    /// 获取当前生命周期状态
    pub fn state(&self) -> crate::lifecycle_manager::LifecycleState {
        self.state
    }

    /// 设置生命周期状态
    pub fn set_state(&mut self, state: crate::lifecycle_manager::LifecycleState) {
        self.state = state;
    }

    /// 处理应用启动
    pub fn on_start(&mut self) -> GResult<()> {
        self.state = crate::lifecycle_manager::LifecycleState::Started;
        Ok(())
    }

    /// 处理应用暂停
    pub fn on_pause(&mut self) -> GResult<()> {
        self.state = crate::lifecycle_manager::LifecycleState::Paused;
        Ok(())
    }

    /// 处理应用恢复
    pub fn on_resume(&mut self) -> GResult<()> {
        self.state = crate::lifecycle_manager::LifecycleState::Resumed;
        Ok(())
    }

    /// 处理应用停止
    pub fn on_stop(&mut self) -> GResult<()> {
        self.state = crate::lifecycle_manager::LifecycleState::Stopped;
        Ok(())
    }

    /// 处理应用销毁
    pub fn on_destroy(&mut self) -> GResult<()> {
        self.state = crate::lifecycle_manager::LifecycleState::Destroyed;
        Ok(())
    }
}
