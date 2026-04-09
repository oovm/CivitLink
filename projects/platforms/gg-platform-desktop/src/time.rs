use std::time::{Duration, Instant};

use gg_core::platform::Time;

/// 桌面平台时间实现
///
/// 基于 `std::time::Instant` 计算帧间隔和运行时间。
pub struct DesktopTime {
    /// 引擎启动时间
    start: Instant,
    /// 上一帧时间
    last_update: Instant,
    /// 上一帧间隔
    delta: Duration,
}

impl DesktopTime {
    /// 创建新的桌面时间实例
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_update: now,
            delta: Duration::ZERO,
        }
    }
}

impl Time for DesktopTime {
    fn delta(&self) -> Duration {
        self.delta
    }

    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    fn update(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_update);
        self.last_update = now;
    }
}
