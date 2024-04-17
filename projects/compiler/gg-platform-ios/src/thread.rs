use std::sync::atomic::{AtomicU64, Ordering};

use gg_core::platform::RuntimeThread;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

/// iOS 平台线程实现
///
/// 为 iOS 平台提供线程操作的具体实现。
pub struct IOSThread {
    /// 当前线程 ID
    current_id: u64,
}

impl IOSThread {
    /// 创建 iOS 线程实例
    pub fn new() -> Self {
        let id = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { current_id: id }
    }
}

impl RuntimeThread for IOSThread {
    fn spawn(&self, f: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(move || {
            f();
        });
    }

    fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send>) {
        std::thread::spawn(move || {
            f();
        });
    }

    fn current_id(&self) -> u64 {
        self.current_id
    }

    fn available_parallelism(&self) -> usize {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    }
}
