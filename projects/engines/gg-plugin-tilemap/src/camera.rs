//! 摄像机视口模块
//! 提供瓦片地图渲染的视口裁剪功能

/// 摄像机视口资源
///
/// 定义当前摄像机可见的矩形区域，用于瓦片地图的视口裁剪。
#[derive(Debug, Clone, Copy)]
pub struct CameraViewport {
    /// 视口左上角 X 坐标（像素）
    pub x: f32,
    /// 视口左上角 Y 坐标（像素）
    pub y: f32,
    /// 视口宽度（像素）
    pub width: f32,
    /// 视口高度（像素）
    pub height: f32,
}

impl Default for CameraViewport {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, width: 800.0, height: 600.0 }
    }
}

impl CameraViewport {
    /// 创建新的摄像机视口
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// 检查指定矩形区域是否与视口重叠
    pub fn intersects(&self, tile_x: f32, tile_y: f32, tile_w: f32, tile_h: f32) -> bool {
        tile_x + tile_w > self.x && tile_x < self.x + self.width && tile_y + tile_h > self.y && tile_y < self.y + self.height
    }
}
