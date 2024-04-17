use std::sync::atomic::{AtomicU64, Ordering};

use gg_core::platform::RuntimeThread;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 移动平台线程实现
///
/// 基于 `std::thread` 提供移动平台的线程和异步任务调度能力。
/// 当前为占位实现，未来将针对移动平台的线程限制进行优化。
pub struct MobileThread {
    /// 当前线程 ID
    current_id: u64,
}

impl MobileThread {
    /// 创建新的移动线程实例
    pub fn new() -> Self {
        let id = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { current_id: id }
    }
}

impl RuntimeThread for MobileThread {
    fn spawn(&self, f: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(f);
    }

    fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(f);
    }

    fn current_id(&self) -> u64 {
        self.current_id
    }

    fn available_parallelism(&self) -> usize {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    }
}
