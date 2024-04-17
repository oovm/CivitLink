use std::thread::JoinHandle;

use gg_core::{GError, GResult};

/// 平台通用线程接口
///
/// 为各平台提供线程操作的统一接口。
pub trait PlatformThread {
    /// 创建新线程
    fn spawn(&self, name: Option<&str>, f: Box<dyn FnOnce() + Send + 'static>) -> GResult<JoinHandle<()>>;

    /// 获取当前线程 ID
    fn current_thread_id(&self) -> u64;

    /// 获取当前线程名称
    fn current_thread_name(&self) -> Option<String>;

    /// 让当前线程睡眠指定毫秒数
    fn sleep(&self, ms: u64);

    /// 让出当前线程的执行时间
    fn yield_now(&self);
}
