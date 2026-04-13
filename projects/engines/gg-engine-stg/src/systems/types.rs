//! STG 引擎系统共享类型定义

/// 行为树节点执行结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// 执行成功
    Success,
    /// 执行失败
    Failure,
    /// 仍在执行中
    Running,
}
