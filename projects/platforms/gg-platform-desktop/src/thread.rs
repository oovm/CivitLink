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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use gg_core::platform::RuntimeThread;

    #[test]
    fn test_spawn_executes() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        let thread = DesktopThread::new();
        thread.spawn(Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_available_parallelism() {
        let thread = DesktopThread::new();
        assert!(thread.available_parallelism() >= 1);
    }

    #[test]
    fn test_current_id() {
        let thread = DesktopThread::new();
        assert!(thread.current_id() < 1000);
    }
}
