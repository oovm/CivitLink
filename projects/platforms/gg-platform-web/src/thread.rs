#![warn(missing_docs)]

use std::sync::atomic::{AtomicU64, Ordering};

use gg_core::platform::RuntimeThread;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Web 平台线程实现
///
/// 在 Web 环境中，所有任务在主线程上通过 `wasm_bindgen_futures::spawn_local` 执行。
/// Web 环境不支持真正的多线程，因此 `available_parallelism` 返回 1。
pub struct WebThread {
    /// 当前线程 ID
    current_id: u64,
}

impl WebThread {
    /// 创建新的 Web 线程实例
    pub fn new() -> Self {
        let id = THREAD_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self { current_id: id }
    }
}

impl RuntimeThread for WebThread {
    fn spawn(&self, f: Box<dyn FnOnce() + Send>) {
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                f();
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            f();
        }
    }

    fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send>) {
        #[cfg(target_arch = "wasm32")]
        {
            f();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            f();
        }
    }

    fn current_id(&self) -> u64 {
        self.current_id
    }

    fn available_parallelism(&self) -> usize {
        1
    }
}
