//! 立绘布局计算模块
//! 提供立绘位置计算功能，根据屏幕尺寸和位置枚举确定立绘坐标

use gg_galgame_schema::components::PortraitPosition;

/// 立绘布局计算器
///
/// 根据屏幕尺寸和位置枚举计算立绘在屏幕上的坐标位置。
pub struct PortraitLayout {
    /// 屏幕宽度
    pub screen_width: f32,
    /// 屏幕高度
    pub screen_height: f32,
}

impl PortraitLayout {
    /// 创建新的立绘布局计算器
    ///
    /// # 参数
    ///
    /// - `screen_width` - 屏幕宽度
    /// - `screen_height` - 屏幕高度
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
    }

    /// 根据位置枚举计算立绘坐标
    ///
    /// # 位置映射规则
    ///
    /// - `Left` => 屏幕左侧 (25%, 50%)
    /// - `Center` => 屏幕中央 (50%, 50%)
    /// - `Right` => 屏幕右侧 (75%, 50%)
    /// - `Custom { x, y }` => 自定义坐标 (x, y)
    pub fn calculate_position(&self, position: &PortraitPosition) -> (f32, f32) {
        match position {
            PortraitPosition::Left => (self.screen_width * 0.25, self.screen_height * 0.5),
            PortraitPosition::Center => (self.screen_width * 0.5, self.screen_height * 0.5),
            PortraitPosition::Right => (self.screen_width * 0.75, self.screen_height * 0.5),
            PortraitPosition::Custom { x, y } => (*x, *y),
        }
    }
}
