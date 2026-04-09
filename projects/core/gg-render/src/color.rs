/// RGBA 颜色类型
///
/// 每个通道的值范围为 `[0.0, 1.0]`，
/// 其中 `0.0` 表示该通道的最小值，`1.0` 表示最大值。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// 红色通道
    pub r: f32,
    /// 绿色通道
    pub g: f32,
    /// 蓝色通道
    pub b: f32,
    /// 透明度通道
    pub a: f32,
}

impl Color {
    /// 创建一个新的颜色
    ///
    /// # 参数
    ///
    /// - `r` - 红色通道值，范围 `[0.0, 1.0]`
    /// - `g` - 绿色通道值，范围 `[0.0, 1.0]`
    /// - `b` - 蓝色通道值，范围 `[0.0, 1.0]`
    /// - `a` - 透明度通道值，范围 `[0.0, 1.0]`
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// 白色 `(1.0, 1.0, 1.0, 1.0)`
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// 黑色 `(0.0, 0.0, 0.0, 1.0)`
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// 完全透明 `(0.0, 0.0, 0.0, 0.0)`
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// 红色 `(1.0, 0.0, 0.0, 1.0)`
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    /// 绿色 `(0.0, 1.0, 0.0, 1.0)`
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    /// 蓝色 `(0.0, 0.0, 1.0, 1.0)`
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}
