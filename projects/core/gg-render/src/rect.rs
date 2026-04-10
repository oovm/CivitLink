/// 矩形区域
///
/// 使用左上角坐标和尺寸定义一个二维矩形区域。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// 左上角 X 坐标
    pub x: f32,
    /// 左上角 Y 坐标
    pub y: f32,
    /// 宽度
    pub width: f32,
    /// 高度
    pub height: f32,
}

impl Rect {
    /// 创建一个新的矩形
    ///
    /// # 参数
    ///
    /// - `x` - 左上角 X 坐标
    /// - `y` - 左上角 Y 坐标
    /// - `width` - 宽度
    /// - `height` - 高度
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// 判断给定点是否在矩形内部
    ///
    /// # 参数
    ///
    /// - `x` - 点的 X 坐标
    /// - `y` - 点的 Y 坐标
    ///
    /// # 返回值
    ///
    /// 如果点在矩形内部（含边界）返回 `true`，否则返回 `false`
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}
