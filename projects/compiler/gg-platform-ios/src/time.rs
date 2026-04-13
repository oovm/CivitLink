use std::time::{Duration, SystemTime};

use gg_core::platform::Time;

/// iOS 平台时间实现
///
/// 为 iOS 平台提供时间操作的具体实现。
pub struct IOSTime {
    /// 引擎启动时间戳
    start_time: SystemTime,
    /// 上一帧时间戳
    last_update: SystemTime,
    /// 上一帧间隔
    delta: Duration,
}

impl IOSTime {
    /// 创建 iOS 时间实例
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self { start_time: now, last_update: now, delta: Duration::ZERO }
    }
}

impl Time for IOSTime {
    fn delta(&self) -> Duration {
        self.delta
    }

    fn elapsed(&self) -> Duration {
        SystemTime::now().duration_since(self.start_time).unwrap_or(Duration::ZERO)
    }

    fn update(&mut self) {
        let now = SystemTime::now();
        self.delta = now.duration_since(self.last_update).unwrap_or(Duration::ZERO);
        self.last_update = now;
    }
}
