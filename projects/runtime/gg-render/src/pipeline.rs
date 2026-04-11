//! 渲染管线模块
//!
//! 定义渲染管线和渲染通道的抽象接口，
//! 支持多通道渲染和自定义渲染流程。

use crate::{Camera, Camera2D, DrawCommand, RenderContext, TextureId};

/// 渲染目标
///
/// 指定渲染通道的输出目标。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderTarget {
    /// 渲染到屏幕
    Screen,
    /// 渲染到指定纹理
    Texture(TextureId),
}

/// 绘制命令过滤器
///
/// 用于在渲染通道中筛选需要处理的绘制命令。
pub trait DrawFilter: Send + Sync + DynClone + std::fmt::Debug {
    /// 判断绘制命令是否应被当前通道处理
    fn should_draw(&self, command: &DrawCommand) -> bool;
}

/// 动态克隆 trait
///
/// 为 `Box<dyn DrawFilter>` 提供克隆能力。
pub trait DynClone: Send + Sync {
    /// 克隆为 Box<dyn DrawFilter>
    fn clone_box(&self) -> Box<dyn DrawFilter>;
}

impl<T> DynClone for T
where
    T: DrawFilter + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn DrawFilter> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DrawFilter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// 接受所有绘制命令的过滤器
#[derive(Debug, Clone, Copy)]
pub struct AcceptAllFilter;

impl DrawFilter for AcceptAllFilter {
    fn should_draw(&self, _command: &DrawCommand) -> bool {
        true
    }
}

/// 渲染通道
///
/// 定义一次渲染过程中的一个独立通道，
/// 包含独立的相机、渲染目标和绘制命令过滤器。
#[derive(Debug, Clone)]
pub struct RenderPass {
    /// 通道名称
    pub name: String,
    /// 通道使用的相机
    pub camera: Option<Camera2D>,
    /// 渲染目标
    pub target: RenderTarget,
    /// 绘制命令过滤器
    pub draw_filter: Box<dyn DrawFilter>,
}

impl RenderPass {
    /// 创建新的渲染通道
    ///
    /// # 参数
    ///
    /// - `name` - 通道名称
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), camera: None, target: RenderTarget::Screen, draw_filter: Box::new(AcceptAllFilter) }
    }

    /// 设置相机
    ///
    /// # 参数
    ///
    /// - `camera` - 2D 增强相机
    pub fn with_camera(mut self, camera: Camera2D) -> Self {
        self.camera = Some(camera);
        self
    }

    /// 设置渲染目标为纹理
    ///
    /// # 参数
    ///
    /// - `texture_id` - 目标纹理标识符
    pub fn with_texture_target(mut self, texture_id: TextureId) -> Self {
        self.target = RenderTarget::Texture(texture_id);
        self
    }

    /// 设置绘制命令过滤器
    ///
    /// # 参数
    ///
    /// - `filter` - 绘制命令过滤器
    pub fn with_filter(mut self, filter: Box<dyn DrawFilter>) -> Self {
        self.draw_filter = filter;
        self
    }

    /// 执行通道渲染
    ///
    /// 从渲染上下文中筛选命令，应用相机变换后提交渲染。
    /// 返回一个新的渲染上下文，包含筛选后的命令、通道的相机设置和渲染目标。
    ///
    /// # 参数
    ///
    /// - `context` - 源渲染上下文
    pub fn execute(&self, context: &mut RenderContext) -> RenderContext {
        let mut new_ctx = RenderContext::new(context.surface_width(), context.surface_height());

        if let Some(ref camera2d) = self.camera {
            new_ctx.set_camera(Camera { position: camera2d.position, zoom: camera2d.zoom, rotation: camera2d.rotation });
        }
        else if let Some(cam) = context.camera() {
            new_ctx.set_camera(*cam);
        }

        for command in context.commands() {
            if self.draw_filter.should_draw(command) {
                new_ctx.draw(command.clone());
            }
        }

        new_ctx.set_target(self.target);

        new_ctx
    }
}

/// 渲染管线 trait
///
/// 定义渲染管线的抽象接口，管理多个渲染通道的执行顺序。
pub trait RenderPipeline: Send + Sync {
    /// 添加渲染通道到管线末尾
    ///
    /// # 参数
    ///
    /// - `pass` - 渲染通道
    fn add_pass(&mut self, pass: RenderPass);

    /// 移除指定名称的渲染通道
    ///
    /// # 参数
    ///
    /// - `name` - 通道名称
    ///
    /// # 返回值
    ///
    /// 如果找到并移除了通道返回 `true`，否则返回 `false`
    fn remove_pass(&mut self, name: &str) -> bool;

    /// 执行整个渲染管线
    ///
    /// 按通道添加顺序依次执行，每个通道独立处理绘制命令。
    ///
    /// # 参数
    ///
    /// - `context` - 源渲染上下文
    ///
    /// # 返回值
    ///
    /// 每个通道执行后产生的渲染上下文列表
    fn execute(&mut self, context: &mut RenderContext) -> Vec<RenderContext>;

    /// 获取管线中所有通道的名称
    fn pass_names(&self) -> Vec<&str>;
}

/// 默认渲染管线实现
///
/// 按通道添加顺序依次执行，每个通道独立处理绘制命令。
#[derive(Debug)]
pub struct DefaultRenderPipeline {
    /// 渲染通道列表
    passes: Vec<RenderPass>,
}

impl DefaultRenderPipeline {
    /// 创建空的默认渲染管线
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }
}

impl Default for DefaultRenderPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderPipeline for DefaultRenderPipeline {
    fn add_pass(&mut self, pass: RenderPass) {
        self.passes.push(pass);
    }

    fn remove_pass(&mut self, name: &str) -> bool {
        let len_before = self.passes.len();
        self.passes.retain(|p| p.name != name);
        self.passes.len() != len_before
    }

    fn execute(&mut self, context: &mut RenderContext) -> Vec<RenderContext> {
        self.passes.iter().map(|pass| pass.execute(context)).collect()
    }

    fn pass_names(&self) -> Vec<&str> {
        self.passes.iter().map(|p| p.name.as_str()).collect()
    }
}
