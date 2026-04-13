//! 场景右键菜单

/// 场景右键菜单
///
/// 管理场景视图右键上下文菜单的显示状态和位置信息。
pub struct SceneContextMenu {
    /// 菜单是否可见
    pub visible: bool,
    /// 菜单屏幕坐标位置
    pub position: (f32, f32),
    /// 菜单对应的世界坐标位置，用于在该位置创建实体
    pub world_position: (f32, f32),
}

impl SceneContextMenu {
    /// 创建默认的隐藏右键菜单
    pub fn new() -> Self {
        Self { visible: false, position: (0.0, 0.0), world_position: (0.0, 0.0) }
    }

    /// 在指定位置显示右键菜单
    ///
    /// `position` 为屏幕坐标，`world_position` 为对应的世界坐标。
    pub fn show(&mut self, position: (f32, f32), world_position: (f32, f32)) {
        self.visible = true;
        self.position = position;
        self.world_position = world_position;
    }

    /// 隐藏右键菜单
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Default for SceneContextMenu {
    fn default() -> Self {
        Self::new()
    }
}
