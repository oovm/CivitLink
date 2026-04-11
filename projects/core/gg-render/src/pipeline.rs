//! 渲染管线模块
//!
//! 定义渲染管线和渲染通道的抽象接口，
//! 支持多通道渲染和自定义渲染流程。

use crate::{Camera2D, Camera, DrawCommand, RenderContext, TextureId};

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
    /// 返回一个新的渲染上下文，包含筛选后的命令和通道的相机设置。
    ///
    /// # 参数
    ///
    /// - `context` - 源渲染上下文
    pub fn execute(&self, context: &mut RenderContext) -> RenderContext {
        let mut new_ctx = RenderContext::new(context.surface_width(), context.surface_height());

        if let Some(ref camera2d) = self.camera {
            new_ctx.set_camera(Camera {
                position: camera2d.position,
                zoom: camera2d.zoom,
                rotation: camera2d.rotation,
            });
        } else if let Some(cam) = context.camera() {
            new_ctx.set_camera(*cam);
        }

        for command in context.commands() {
            if self.draw_filter.should_draw(command) {
                new_ctx.draw(command.clone());
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Camera2D, Color, Rect, Transform, Viewport};

    #[test]
    fn test_viewport_full() {
        let vp = Viewport::FULL;
        assert_eq!(vp.x, 0.0);
        assert_eq!(vp.y, 0.0);
        assert_eq!(vp.width, 1.0);
        assert_eq!(vp.height, 1.0);
    }

    #[test]
    fn test_viewport_new() {
        let vp = Viewport::new();
        assert_eq!(vp, Viewport::FULL);
    }

    #[test]
    fn test_viewport_default() {
        let vp = Viewport::default();
        assert_eq!(vp, Viewport::FULL);
    }

    #[test]
    fn test_viewport_with_rect() {
        let vp = Viewport::with_rect(0.25, 0.25, 0.5, 0.5);
        assert_eq!(vp.x, 0.25);
        assert_eq!(vp.y, 0.25);
        assert_eq!(vp.width, 0.5);
        assert_eq!(vp.height, 0.5);
    }

    #[test]
    fn test_camera2d_new() {
        let cam = Camera2D::new();
        assert_eq!(cam.position, [0.0, 0.0]);
        assert_eq!(cam.zoom, 1.0);
        assert_eq!(cam.rotation, 0.0);
        assert_eq!(cam.viewport, Viewport::FULL);
        assert_eq!(cam.ortho_size, 0.0);
    }

    #[test]
    fn test_camera2d_default() {
        let cam = Camera2D::default();
        assert_eq!(cam.position, [0.0, 0.0]);
    }

    #[test]
    fn test_camera2d_builder() {
        let cam = Camera2D::new()
            .with_position(100.0, 200.0)
            .with_zoom(2.0)
            .with_rotation(1.57)
            .with_viewport_rect(0.0, 0.0, 0.5, 0.5)
            .with_ortho_size(5.0);

        assert_eq!(cam.position, [100.0, 200.0]);
        assert_eq!(cam.zoom, 2.0);
        assert_eq!(cam.rotation, 1.57);
        assert_eq!(cam.viewport, Viewport::with_rect(0.0, 0.0, 0.5, 0.5));
        assert_eq!(cam.ortho_size, 5.0);
    }

    #[test]
    fn test_camera2d_builder_with_viewport() {
        let vp = Viewport::with_rect(0.1, 0.2, 0.3, 0.4);
        let cam = Camera2D::new().with_viewport(vp);
        assert_eq!(cam.viewport, vp);
    }

    #[test]
    fn test_camera2d_from_camera() {
        let camera = Camera::new().with_position(10.0, 20.0).with_zoom(3.0).with_rotation(0.5);
        let cam2d: Camera2D = camera.into();

        assert_eq!(cam2d.position, [10.0, 20.0]);
        assert_eq!(cam2d.zoom, 3.0);
        assert_eq!(cam2d.rotation, 0.5);
        assert_eq!(cam2d.viewport, Viewport::FULL);
        assert_eq!(cam2d.ortho_size, 0.0);
    }

    #[test]
    fn test_camera2d_view_projection_identity() {
        let cam = Camera2D::new();
        let vp = cam.view_projection(800.0, 600.0);

        // Identity camera should produce the same result as the orthographic projection alone
        // (since viewport is FULL and camera is at origin with zoom=1)
        // The top-left corner (0,0) should map to (-1, 1) in NDC
        // The bottom-right corner (800,600) should map to (1, -1) in NDC
        assert!(vp[0][0] > 0.0, "first column x scaling should be positive");
        assert!(vp[1][1] < 0.0, "second column y scaling should be negative (Y-down)");
    }

    #[test]
    fn test_camera2d_view_projection_with_position() {
        let cam = Camera2D::new().with_position(100.0, 0.0);
        let vp = cam.view_projection(800.0, 600.0);

        // With position offset, the projection should differ from identity
        let cam_identity = Camera2D::new();
        let vp_identity = cam_identity.view_projection(800.0, 600.0);
        assert_ne!(vp, vp_identity, "camera with position offset should produce different matrix");
    }

    #[test]
    fn test_camera2d_view_projection_with_ortho_size() {
        let cam = Camera2D::new().with_ortho_size(5.0);
        let vp = cam.view_projection(800.0, 600.0);

        // With ortho_size=5.0, visible height = 10 world units
        // scale factor = 600 / 10 = 60
        // projection uses virtual width = 800/60, height = 600/60 = 10
        let cam_pixel = Camera2D::new();
        let vp_pixel = cam_pixel.view_projection(800.0, 600.0);
        assert_ne!(vp, vp_pixel, "ortho_size should change the projection matrix");
    }

    #[test]
    fn test_render_target_equality() {
        assert_eq!(RenderTarget::Screen, RenderTarget::Screen);
        let tex = TextureId::new(42);
        assert_eq!(RenderTarget::Texture(tex), RenderTarget::Texture(tex));
        assert_ne!(RenderTarget::Screen, RenderTarget::Texture(tex));
    }

    #[test]
    fn test_accept_all_filter() {
        let filter = AcceptAllFilter;
        let cmd = DrawCommand::Rect { rect: Rect::new(0.0, 0.0, 10.0, 10.0), color: Color::WHITE, corner_radius: 0.0 };
        assert!(filter.should_draw(&cmd));
    }

    #[test]
    fn test_render_pass_new() {
        let pass = RenderPass::new("main");
        assert_eq!(pass.name, "main");
        assert!(pass.camera.is_none());
        assert_eq!(pass.target, RenderTarget::Screen);
    }

    #[test]
    fn test_render_pass_builder() {
        let tex = TextureId::new(1);
        let pass = RenderPass::new("ui")
            .with_camera(Camera2D::new().with_zoom(2.0))
            .with_texture_target(tex);

        assert_eq!(pass.name, "ui");
        assert!(pass.camera.is_some());
        assert_eq!(pass.target, RenderTarget::Texture(tex));
    }

    #[test]
    fn test_render_pass_execute() {
        let pass = RenderPass::new("main").with_camera(Camera2D::new().with_zoom(2.0));
        let mut ctx = RenderContext::new(800, 600);
        ctx.draw(DrawCommand::Rect { rect: Rect::new(0.0, 0.0, 10.0, 10.0), color: Color::WHITE, corner_radius: 0.0 });

        let result = pass.execute(&mut ctx);
        assert_eq!(result.commands().len(), 1);
        assert!(result.camera().is_some());
        assert_eq!(result.surface_width(), 800);
        assert_eq!(result.surface_height(), 600);
    }

    #[test]
    fn test_default_render_pipeline_add_remove() {
        let mut pipeline = DefaultRenderPipeline::new();
        assert!(pipeline.pass_names().is_empty());

        pipeline.add_pass(RenderPass::new("main"));
        pipeline.add_pass(RenderPass::new("ui"));
        assert_eq!(pipeline.pass_names(), vec!["main", "ui"]);

        assert!(pipeline.remove_pass("main"));
        assert_eq!(pipeline.pass_names(), vec!["ui"]);

        assert!(!pipeline.remove_pass("nonexistent"));
        assert_eq!(pipeline.pass_names(), vec!["ui"]);
    }

    #[test]
    fn test_default_render_pipeline_execute() {
        let mut pipeline = DefaultRenderPipeline::new();
        pipeline.add_pass(RenderPass::new("main"));
        pipeline.add_pass(RenderPass::new("ui"));

        let mut ctx = RenderContext::new(800, 600);
        ctx.draw(DrawCommand::Rect { rect: Rect::new(0.0, 0.0, 10.0, 10.0), color: Color::WHITE, corner_radius: 0.0 });

        let results = pipeline.execute(&mut ctx);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].commands().len(), 1);
        assert_eq!(results[1].commands().len(), 1);
    }

    #[test]
    fn test_render_pass_clone() {
        let pass = RenderPass::new("main").with_camera(Camera2D::new().with_zoom(2.0));
        let cloned = pass.clone();
        assert_eq!(cloned.name, "main");
        assert_eq!(cloned.camera.unwrap().zoom, 2.0);
    }
}
