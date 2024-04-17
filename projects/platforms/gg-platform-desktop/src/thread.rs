#![warn(missing_docs)]

use std::sync::atomic::{AtomicU64, Ordering};

use gg_core::platform::RuntimeThread;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 桌面平台线程实现
///
/// 基于 `std::thread` 提供桌面平台的线程和异步任务调度能力。
pub struct DesktopThread {
    /// 当前线程 ID
    current_id: u64,
}

impl DesktopThread {
    /// 创建新的桌面线程实例
    pub fn new() -> Self {
        let id = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { current_id: id }
    }
}

impl RuntimeThread for DesktopThread {
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
