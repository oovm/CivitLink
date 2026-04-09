use std::path::Path;

use gg_core::GResult;

use crate::{DrawCommand, SurfaceInfo, TextureId};

/// 渲染上下文
///
/// 收集一帧中所有的绘制命令，并传递给渲染器执行。
/// 渲染上下文持有当前渲染表面的尺寸信息，用于坐标计算。
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// 绘制命令列表
    commands: Vec<DrawCommand>,
    /// 渲染表面宽度（像素）
    surface_width: u32,
    /// 渲染表面高度（像素）
    surface_height: u32,
}

impl RenderContext {
    /// 创建新的渲染上下文
    ///
    /// # 参数
    ///
    /// - `width` - 渲染表面宽度（像素）
    /// - `height` - 渲染表面高度（像素）
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            commands: Vec::new(),
            surface_width: width,
            surface_height: height,
        }
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
}
