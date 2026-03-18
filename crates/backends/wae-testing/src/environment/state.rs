//! 测试环境状态模块

/// 测试环境状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEnvState {
    /// 未初始化
    Uninitialized,
    /// 正在初始化
    Initializing,
    /// 已初始化
    Initialized,
    /// 正在销毁
    Destroying,
    /// 已销毁
    Destroyed,
}
