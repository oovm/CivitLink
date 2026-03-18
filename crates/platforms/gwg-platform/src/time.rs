//! 时间抽象
//!
//! 提供跨平台的时间接口。

use super::{PlatformError, PlatformResult};

/// 时间 trait
pub trait Time {
    /// 获取自程序启动以来的时间（秒）
    fn elapsed_seconds(&self) -> f64;

    /// 获取自程序启动以来的时间（毫秒）
    fn elapsed_millis(&self) -> u64 {
        (self.elapsed_seconds() * 1000.0) as u64
    }

    /// 获取上一帧的持续时间（秒）
    fn delta_seconds(&self) -> f64;

    /// 获取上一帧的持续时间（毫秒）
    fn delta_millis(&self) -> f64 {
        self.delta_seconds() * 1000.0
    }

    /// 获取当前帧索引
    fn frame_count(&self) -> u64;

    /// 更新时间（每帧开始调用）
    fn update(&mut self);

    /// 休眠指定时间（秒）
    fn sleep(&self, seconds: f64) -> PlatformResult<()>;
}

/// 空时间实现
pub struct NullTime {
    start_time: std::time::Instant,
    last_frame_time: std::time::Instant,
    delta_seconds: f64,
    frame_count: u64,
}

impl NullTime {
    /// 创建新的 NullTime
    pub fn new() -> Self {
        let now = std::time::Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_seconds: 0.0,
            frame_count: 0,
        }
    }
}

impl Default for NullTime {
    fn default() -> Self {
        Self::new()
    }
}

impl Time for NullTime {
    fn elapsed_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    fn delta_seconds(&self) -> f64 {
        self.delta_seconds
    }

    fn frame_count(&self) -> u64 {
        self.frame_count
    }

    fn update(&mut self) {
        let now = std::time::Instant::now();
        self.delta_seconds = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;
        self.frame_count += 1;
    }

    fn sleep(&self, seconds: f64) -> PlatformResult<()> {
        let duration = std::time::Duration::from_secs_f64(seconds);
        std::thread::sleep(duration);
        Ok(())
    }
}
