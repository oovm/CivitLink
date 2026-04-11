use gg_core::GResult;

/// 平台通用时间接口
///
/// 为各平台提供时间操作的统一接口。
pub trait PlatformTime {
    /// 获取当前时间戳（毫秒）
    fn now(&self) -> u64;

    /// 获取当前时间戳（微秒）
    fn now_micros(&self) -> u64;

    /// 获取当前时间戳（纳秒）
    fn now_nanos(&self) -> u64;

    /// 获取自启动以来的时间（毫秒）
    fn elapsed(&self) -> u64;

    /// 获取自启动以来的时间（微秒）
    fn elapsed_micros(&self) -> u64;

    /// 获取自启动以来的时间（纳秒）
    fn elapsed_nanos(&self) -> u64;

    /// 设置时间缩放因子
    fn set_time_scale(&mut self, scale: f64);

    /// 获取时间缩放因子
    fn time_scale(&self) -> f64;
}
