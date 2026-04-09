/// 触摸点信息
#[derive(Debug, Clone)]
pub struct TouchPoint {
    /// 触摸点唯一标识
    pub id: u64,
    /// 触摸点 X 坐标
    pub x: f32,
    /// 触摸点 Y 坐标
    pub y: f32,
    /// 触摸压力值（0.0 - 1.0），不支持压力感应的设备返回 1.0
    pub pressure: f32,
}

/// 移动平台输入扩展 trait
///
/// 提供移动平台特有的输入能力，如多点触控和加速度计。
/// 此 trait 扩展了 `gg_core::platform::Input`，为移动设备提供更丰富的输入支持。
pub trait MobileInput {
    /// 获取当前所有活跃的触摸点
    ///
    /// 返回当前屏幕上所有正在触摸的点的列表。
    /// 在不支持多点触控的设备上，列表最多包含一个元素。
    fn active_touches(&self) -> Vec<TouchPoint>;

    /// 获取设备加速度计数据
    ///
    /// 返回三元组 `(x, y, z)` 表示三个轴的加速度值（单位：m/s²）。
    /// 如果设备不支持加速度计或数据不可用，返回 `None`。
    fn accelerometer(&self) -> Option<(f32, f32, f32)>;

    /// 获取设备陀螺仪数据
    ///
    /// 返回三元组 `(x, y, z)` 表示三个轴的角速度值（单位：rad/s）。
    /// 如果设备不支持陀螺仪或数据不可用，返回 `None`。
    fn gyroscope(&self) -> Option<(f32, f32, f32)>;
}
