use std::time::Duration;

use gg_core::platform::Time;

/// Web 平台时间实现
///
/// 基于 `performance.now()` 获取高精度时间。
pub struct WebTime {
    /// 引擎启动时间戳（毫秒）
    start_ms: f64,
    /// 上一帧时间戳（毫秒）
    last_update_ms: f64,
    /// 上一帧间隔
    delta: Duration,
}

impl WebTime {
    /// 创建新的 Web 时间实例
    pub fn new() -> Self {
        let now_ms = Self::current_time_ms();
        Self {
            start_ms: now_ms,
            last_update_ms: now_ms,
            delta: Duration::ZERO,
        }
    }

    /// 获取当前时间戳（毫秒）
    fn current_time_ms() -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or_else(js_sys::Date::now)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let duration = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO);
            duration.as_secs_f64() * 1000.0
        }
    }
}

impl Time for WebTime {
    fn delta(&self) -> Duration {
        self.delta
    }

    fn elapsed(&self) -> Duration {
        let now_ms = Self::current_time_ms();
        Duration::from_secs_f64((now_ms - self.start_ms) / 1000.0)
    }

    fn update(&mut self) {
        let now_ms = Self::current_time_ms();
        let delta_ms = now_ms - self.last_update_ms;
        self.delta = Duration::from_secs_f64(delta_ms / 1000.0);
        self.last_update_ms = now_ms;
    }
}
