#![warn(missing_docs)]

//! 线程/运行时抽象层
//! 提供跨平台的线程和异步任务调度接口

/// 线程/运行时抽象 trait
pub trait RuntimeThread: Send + Sync + 'static {
    /// 生成一个异步任务
    ///
    /// 在桌面平台上使用独立线程执行，在 Web 平台上使用 spawn_local。
    fn spawn(&self, f: Box<dyn FnOnce() + Send>);

    /// 生成一个阻塞任务
    ///
    /// 在桌面平台上使用独立线程执行，在 Web 平台上在主线程同步执行。
    fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send>);

    /// 获取当前线程的唯一标识
    fn current_id(&self) -> u64;

    /// 获取可用的并行度
    ///
    /// 返回建议的并行线程数。在 Web 平台上返回 1。
    fn available_parallelism(&self) -> usize;
}
