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

/// 视口矩形
///
/// 定义渲染目标中的矩形区域，使用归一化坐标（0.0~1.0）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// 视口左下角 X 坐标（归一化）
    pub x: f32,
    /// 视口左下角 Y 坐标（归一化）
    pub y: f32,
    /// 视口宽度（归一化）
    pub width: f32,
    /// 视口高度（归一化）
    pub height: f32,
}

impl Viewport {
    /// 全屏视口
    pub const FULL: Self = Self { x: 0.0, y: 0.0, width: 1.0, height: 1.0 };

    /// 创建全屏视口
    pub fn new() -> Self {
        Self::FULL
    }

    /// 创建指定区域的视口
    ///
    /// # 参数
    ///
    /// - `x` - 左下角 X 坐标（归一化）
    /// - `y` - 左下角 Y 坐标（归一化）
    /// - `width` - 宽度（归一化）
    /// - `height` - 高度（归一化）
    pub fn with_rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

/// 2D 增强相机
///
/// 扩展基础 [`Camera`] 的功能，支持视口裁剪和正交尺寸控制。
/// 可用于编辑器中的多视口渲染或分屏游戏。
#[derive(Debug, Clone, PartialEq)]
pub struct Camera2D {
    /// 相机位置 `[x, y]`，世界坐标中的偏移
    pub position: [f32; 2],
    /// 缩放倍数，`1.0` 为原始大小
    pub zoom: f32,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// 渲染视口（归一化坐标）
    pub viewport: Viewport,
    /// 正交尺寸（世界单位）
    ///
    /// 当 `ortho_size > 0` 时，使用正交尺寸计算投影矩阵，
    /// 相机可见高度为 `ortho_size * 2` 世界单位。
    /// 当 `ortho_size == 0` 时，使用像素对齐模式（1:1 映射）。
    pub ortho_size: f32,
}

impl Camera2D {
    /// 创建默认 2D 相机
    ///
    /// 位于原点，无缩放无旋转，全屏视口，正交尺寸为 0（像素对齐）。
    pub fn new() -> Self {
        Self { position: [0.0, 0.0], zoom: 1.0, rotation: 0.0, viewport: Viewport::FULL, ortho_size: 0.0 }
    }

    /// 设置相机位置
    ///
    /// # 参数
    ///
    /// - `x` - X 坐标
    /// - `y` - Y 坐标
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    /// 设置缩放倍数
    ///
    /// # 参数
    ///
    /// - `zoom` - 缩放倍数
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    /// 设置旋转角度
    ///
    /// # 参数
    ///
    /// - `rotation` - 旋转角度（弧度）
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置渲染视口
    ///
    /// # 参数
    ///
    /// - `viewport` - 视口矩形
    pub fn with_viewport(mut self, viewport: Viewport) -> Self {
        self.viewport = viewport;
        self
    }

    /// 设置渲染视口为指定区域
    ///
    /// # 参数
    ///
    /// - `x` - 左下角 X 坐标（归一化）
    /// - `y` - 左下角 Y 坐标（归一化）
    /// - `w` - 宽度（归一化）
    /// - `h` - 高度（归一化）
    pub fn with_viewport_rect(mut self, x: f32, y: f32, w: f32, h: f32) -> Self {
        self.viewport = Viewport::with_rect(x, y, w, h);
        self
    }

    /// 设置正交尺寸
    ///
    /// # 参数
    ///
    /// - `size` - 正交尺寸（世界单位），可见高度为 `size * 2`
    pub fn with_ortho_size(mut self, size: f32) -> Self {
        self.ortho_size = size;
        self
    }

    /// 计算视图投影矩阵
    ///
    /// 根据相机参数和渲染表面尺寸计算 4x4 视图投影矩阵。
    /// 矩阵采用列主序存储，原点在左上角，Y 轴向下。
    ///
    /// # 参数
    ///
    /// - `surface_width` - 渲染表面宽度（像素）
    /// - `surface_height` - 渲染表面高度（像素）
    pub fn view_projection(&self, surface_width: f32, surface_height: f32) -> [[f32; 4]; 4] {
        let projection = if self.ortho_size > 0.0 {
            let scale = surface_height / (self.ortho_size * 2.0);
            orthographic(surface_width / scale, surface_height / scale)
        } else {
            orthographic(surface_width, surface_height)
        };

        let half_w = surface_width * 0.5;
        let half_h = surface_height * 0.5;

        let view_t = translate(-self.position[0], -self.position[1]);
        let view_r = rotate(-self.rotation);
        let view_s = scale(self.zoom, self.zoom);
        let center_t = translate(half_w, half_h);
        let center_t_inv = translate(-half_w, -half_h);

        let view = mat4_mul(
            &center_t,
            &mat4_mul(&view_r, &mat4_mul(&view_s, &mat4_mul(&center_t_inv, &view_t))),
        );

        let vp_viewport = translate(self.viewport.x * surface_width, self.viewport.y * surface_height);
        let vs_viewport = scale(self.viewport.width, self.viewport.height);

        mat4_mul(&mat4_mul(&vp_viewport, &vs_viewport), &mat4_mul(&projection, &view))
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Camera> for Camera2D {
    fn from(camera: Camera) -> Self {
        Self { position: camera.position, zoom: camera.zoom, rotation: camera.rotation, viewport: Viewport::FULL, ortho_size: 0.0 }
    }
}

/// 计算正交投影矩阵
///
/// 原点在左上角，X 轴向右，Y 轴向下，Z 轴范围 `[0, 1]`。
fn orthographic(width: f32, height: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, 0.5, 0.0],
        [-1.0, 1.0, 0.5, 1.0],
    ]
}

/// 计算平移矩阵
fn translate(tx: f32, ty: f32) -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [tx, ty, 0.0, 1.0]]
}

/// 计算旋转矩阵
fn rotate(angle: f32) -> [[f32; 4]; 4] {
    let (s, c) = (angle.sin(), angle.cos());
    [[c, s, 0.0, 0.0], [-s, c, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
}

/// 计算缩放矩阵
fn scale(sx: f32, sy: f32) -> [[f32; 4]; 4] {
    [[sx, 0.0, 0.0, 0.0], [0.0, sy, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]
}

/// 4x4 矩阵乘法（列主序）
fn mat4_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            result[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
        }
    }
    result
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
