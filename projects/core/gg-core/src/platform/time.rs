#![warn(missing_docs)]

//! 时间抽象层
//! 提供跨平台的时间测量接口

/// 时间抽象 trait
pub trait Time {
    /// 获取上一帧到当前帧的间隔时间
    fn delta(&self) -> std::time::Duration;

    /// 获取自时间系统启动以来的总耗时
    fn elapsed(&self) -> std::time::Duration;

    /// 更新时间状态，应在每帧调用
    fn update(&mut self);
}
