use gg_core::{GError, GResult};

/// 生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    /// 已创建
    Created,
    /// 已启动
    Started,
    /// 已暂停
    Paused,
    /// 已恢复
    Resumed,
    /// 已停止
    Stopped,
    /// 已销毁
    Destroyed,
}

/// iOS 平台生命周期管理器
///
/// 为 iOS 平台提供生命周期管理的具体实现。
pub struct IOSLifecycleManager {
    lifecycle: crate::lifecycle::IOSLifecycle,
}

impl IOSLifecycleManager {
    /// 创建 iOS 生命周期管理器实例
    pub fn new() -> Self {
        Self { lifecycle: crate::lifecycle::IOSLifecycle::new() }
    }

    /// 获取当前生命周期状态
    pub fn state(&self) -> LifecycleState {
        self.lifecycle.state()
    }

    /// 处理应用启动
    pub fn start(&mut self) -> GResult<()> {
        self.lifecycle.on_start()
    }

    /// 处理应用暂停
    pub fn pause(&mut self) -> GResult<()> {
        self.lifecycle.on_pause()
    }

    /// 处理应用恢复
    pub fn resume(&mut self) -> GResult<()> {
        self.lifecycle.on_resume()
    }

    /// 处理应用停止
    pub fn stop(&mut self) -> GResult<()> {
        self.lifecycle.on_stop()
    }

    /// 处理应用销毁
    pub fn destroy(&mut self) -> GResult<()> {
        self.lifecycle.on_destroy()
    }
}
