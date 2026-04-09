//! 视口状态管理

/// 视口状态
///
/// 管理场景视图的平移偏移、缩放和尺寸，
/// 提供世界坐标与屏幕坐标之间的转换功能。
#[derive(Debug, Clone)]
pub struct ViewportState {
    /// 平移偏移 (x, y)
    pub offset: (f32, f32),
    /// 缩放因子
    pub zoom: f32,
    /// 视口尺寸 (宽, 高)
    pub size: (f32, f32),
}

impl ViewportState {
    /// 创建默认视口状态
    ///
    /// 默认偏移为 (0, 0)，缩放为 1.0，尺寸为 (1280, 720)。
    pub fn new() -> Self {
        Self {
            offset: (0.0, 0.0),
            zoom: 1.0,
            size: (1280.0, 720.0),
        }
    }

    /// 将世界坐标转换为屏幕坐标
    ///
    /// 应用偏移和缩放变换：`screen = (world - offset) * zoom`
    pub fn world_to_screen(&self, world_pos: (f32, f32)) -> (f32, f32) {
        (
            (world_pos.0 - self.offset.0) * self.zoom,
            (world_pos.1 - self.offset.1) * self.zoom,
        )
    }

    /// 将屏幕坐标转换为世界坐标
    ///
    /// 应用逆变换：`world = screen / zoom + offset`
    pub fn screen_to_world(&self, screen_pos: (f32, f32)) -> (f32, f32) {
        (
            screen_pos.0 / self.zoom + self.offset.0,
            screen_pos.1 / self.zoom + self.offset.1,
        )
    }

    /// 平移视口
    ///
    /// 将指定增量添加到当前偏移。
    pub fn pan(&mut self, delta: (f32, f32)) {
        self.offset.0 += delta.0;
        self.offset.1 += delta.1;
    }

    /// 缩放视口到指定中心点
    ///
    /// 以屏幕坐标 `center` 为中心，将缩放因子乘以 `factor`，
    /// 同时调整偏移以保持中心点位置不变。
    pub fn zoom_to(&mut self, factor: f32, center: (f32, f32)) {
        let old_zoom = self.zoom;
        self.zoom *= factor;
        self.offset.0 = center.0 / old_zoom + self.offset.0 - center.0 / self.zoom;
        self.offset.1 = center.1 / old_zoom + self.offset.1 - center.1 / self.zoom;
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new()
    }
}
