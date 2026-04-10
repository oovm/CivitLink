use std::path::Path;

use gg_core::{GError, GErrorKind, GResult};

use crate::{DrawCommand, Rect, SurfaceInfo, TextureId};

/// 相机
///
/// 定义渲染时的视口变换参数。
/// 所有绘制命令的坐标将经过相机变换后渲染。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// 相机位置 `[x, y]`，世界坐标中的偏移
    pub position: [f32; 2],
    /// 缩放倍数，`1.0` 为原始大小
    pub zoom: f32,
    /// 旋转角度（弧度）
    pub rotation: f32,
}

impl Camera {
    /// 默认相机，位于原点，无缩放无旋转
    pub const IDENTITY: Self = Self { position: [0.0, 0.0], zoom: 1.0, rotation: 0.0 };

    /// 创建默认相机
    pub fn new() -> Self {
        Self::IDENTITY
    }

    /// 设置相机位置
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    /// 设置缩放倍数
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// 设置旋转角度
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// 渲染上下文
///
/// 收集一帧中所有的绘制命令，并传递给渲染器执行。
/// 渲染上下文持有当前渲染表面的尺寸信息、相机变换和裁剪矩形。
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// 绘制命令列表
    commands: Vec<DrawCommand>,
    /// 渲染表面宽度（像素）
    surface_width: u32,
    /// 渲染表面高度（像素）
    surface_height: u32,
    /// 相机变换
    camera: Option<Camera>,
    /// 裁剪矩形
    clip_rect: Option<Rect>,
}

impl RenderContext {
    /// 创建新的渲染上下文
    ///
    /// # 参数
    ///
    /// - `width` - 渲染表面宽度（像素）
    /// - `height` - 渲染表面高度（像素）
    pub fn new(width: u32, height: u32) -> Self {
        Self { commands: Vec::new(), surface_width: width, surface_height: height, camera: None, clip_rect: None }
    }

    /// 添加一条绘制命令
    ///
    /// # 参数
    ///
    /// - `command` - 要添加的绘制命令
    pub fn draw(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    /// 清空所有绘制命令
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 获取所有绘制命令的引用
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// 调整渲染表面尺寸
    ///
    /// # 参数
    ///
    /// - `width` - 新的宽度（像素）
    /// - `height` - 新的高度（像素）
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_width = width;
        self.surface_height = height;
    }

    /// 获取渲染表面宽度
    pub fn surface_width(&self) -> u32 {
        self.surface_width
    }

    /// 获取渲染表面高度
    pub fn surface_height(&self) -> u32 {
        self.surface_height
    }

    /// 设置相机变换
    ///
    /// # 参数
    ///
    /// - `camera` - 相机实例
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    /// 获取相机的引用
    pub fn camera(&self) -> Option<&Camera> {
        self.camera.as_ref()
    }

    /// 清除相机变换，恢复为默认正交投影
    pub fn clear_camera(&mut self) {
        self.camera = None;
    }

    /// 设置裁剪矩形
    ///
    /// 后续绘制命令只在裁剪矩形区域内渲染。
    ///
    /// # 参数
    ///
    /// - `rect` - 裁剪矩形
    pub fn set_clip_rect(&mut self, rect: Rect) {
        self.clip_rect = Some(rect);
    }

    /// 获取裁剪矩形的引用
    pub fn clip_rect(&self) -> Option<&Rect> {
        self.clip_rect.as_ref()
    }

    /// 清除裁剪矩形，恢复为全表面渲染
    pub fn clear_clip_rect(&mut self) {
        self.clip_rect = None;
    }
}

/// 渲染器 trait
///
/// 定义渲染后端必须实现的核心接口。
/// 不同的图形 API（如 Vulkan、Metal、DirectX、OpenGL 等）
/// 通过实现此 trait 来提供具体的渲染能力。
pub trait Renderer {
    /// 开始一帧的渲染
    ///
    /// 在调用任何绘制操作之前必须先调用此方法。
    fn begin_frame(&mut self) -> GResult<()>;

    /// 结束一帧的渲染
    ///
    /// 在所有绘制操作完成后调用此方法。
    fn end_frame(&mut self) -> GResult<()>;

    /// 执行渲染上下文中的所有绘制命令
    ///
    /// # 参数
    ///
    /// - `context` - 包含绘制命令的渲染上下文
    fn draw(&mut self, context: &RenderContext) -> GResult<()>;

    /// 将渲染结果呈现到屏幕
    fn present(&mut self) -> GResult<()>;

    /// 从文件加载纹理资源
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    ///
    /// # 返回值
    ///
    /// 成功时返回纹理标识符
    fn load_texture(&mut self, path: &Path) -> GResult<TextureId>;

    /// 调整渲染器内部尺寸
    ///
    /// # 参数
    ///
    /// - `width` - 新的宽度（像素）
    /// - `height` - 新的高度（像素）
    fn resize(&mut self, width: u32, height: u32);

    /// 获取渲染表面信息
    fn surface_info(&self) -> &SurfaceInfo;

    /// 重新加载纹理（用于 HMR 热更新）
    ///
    /// 从指定路径重新加载纹理数据并更新 GPU 纹理对象。
    /// 默认实现返回不支持错误。
    ///
    /// # 参数
    ///
    /// - `path` - 纹理文件路径
    fn reload_texture(&mut self, path: &str) -> GResult<()> {
        Err(GError { kind: GErrorKind::Runtime, message: format!("Texture reload not supported: {}", path) })
    }
}
