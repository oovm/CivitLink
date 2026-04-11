#![warn(missing_docs)]

//! GG 引擎编辑器渲染模块
//!
//! 桥接编辑器 UI 树与 WgpuRenderer，
//! 提供编辑器专用的渲染管线封装。
//! 支持场景 RTT 渲染和帧率控制。

use std::collections::HashMap;
use std::time::{Duration, Instant};

use gg_core::{GResult, WindowEvent};
use gg_render::{RenderContext, Renderer, SurfaceInfo, TextureId};
use gg_render_wgpu::{RenderTarget, WgpuRenderer};
use gg_ui::UiRenderer;

/// 渲染目标信息
///
/// 封装离屏渲染目标及其元数据，
/// 用于编辑器中将游戏场景渲染到纹理后再显示。
pub struct RenderTargetInfo {
    /// 底层渲染目标对象
    render_target: RenderTarget,
    /// 渲染目标宽度（像素）
    width: u32,
    /// 渲染目标高度（像素）
    height: u32,
}

impl RenderTargetInfo {
    /// 获取渲染目标的纹理标识符
    ///
    /// 返回的标识符可用于精灵绘制命令中的纹理引用，
    /// 例如在 UI 中将渲染目标作为图片显示。
    pub fn texture_id(&self) -> TextureId {
        self.render_target.texture_id()
    }

    /// 获取渲染目标宽度（像素）
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取渲染目标高度（像素）
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 获取底层渲染目标对象的引用
    ///
    /// 用于调用 `WgpuRenderer::draw_to_target` 进行离屏渲染。
    pub fn render_target(&self) -> &RenderTarget {
        &self.render_target
    }
}

/// 编辑器渲染器
///
/// 封装 WgpuRenderer，提供编辑器专用的渲染流程。
/// 负责将 UI 树转换为绘制命令并提交到 GPU 渲染，
/// 同时支持场景 RTT 渲染和帧率控制。
pub struct EditorRenderer {
    /// 底层 WGPU 渲染器实例
    renderer: WgpuRenderer,
    /// 场景渲染目标映射表
    scene_render_targets: HashMap<String, RenderTargetInfo>,
    /// 目标帧率
    target_fps: u32,
    /// 上一帧时间
    last_frame_time: Instant,
}

#[cfg(not(target_arch = "wasm32"))]
impl EditorRenderer {
    /// 创建新的编辑器渲染器（桌面平台）
    ///
    /// 使用指定的 winit 事件循环和渲染表面信息创建渲染器。
    /// 默认目标帧率为 60 FPS。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环引用
    /// - `surface_info` - 渲染表面信息
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>, surface_info: SurfaceInfo) -> GResult<Self> {
        let renderer = WgpuRenderer::new(event_loop, surface_info)?;
        Ok(Self {
            renderer,
            scene_render_targets: HashMap::new(),
            target_fps: 60,
            last_frame_time: Instant::now(),
        })
    }
}

impl EditorRenderer {
    /// 创建场景渲染目标
    ///
    /// 创建一个指定尺寸的离屏渲染目标并注册到映射表中。
    /// 渲染目标可用于将游戏场景渲染到纹理，
    /// 然后在编辑器 UI 中作为图片显示。
    ///
    /// # 参数
    ///
    /// - `name` - 渲染目标名称，用于后续查找
    /// - `width` - 渲染目标宽度（像素）
    /// - `height` - 渲染目标高度（像素）
    ///
    /// # 返回值
    ///
    /// 成功时返回渲染目标的纹理标识符
    pub fn create_scene_render_target(&mut self, name: &str, width: u32, height: u32) -> GResult<TextureId> {
        let render_target = self.renderer.create_render_target(width, height)?;
        let texture_id = render_target.texture_id();
        let info = RenderTargetInfo {
            render_target,
            width,
            height,
        };
        self.scene_render_targets.insert(name.to_string(), info);
        Ok(texture_id)
    }

    /// 获取场景渲染目标
    ///
    /// 根据名称查找已注册的场景渲染目标。
    ///
    /// # 参数
    ///
    /// - `name` - 渲染目标名称
    ///
    /// # 返回值
    ///
    /// 存在时返回渲染目标信息的引用
    pub fn get_scene_render_target(&self, name: &str) -> Option<&RenderTargetInfo> {
        self.scene_render_targets.get(name)
    }

    /// 移除场景渲染目标
    ///
    /// 从映射表中移除指定名称的渲染目标。
    /// 移除后渲染目标的纹理将不再可用。
    ///
    /// # 参数
    ///
    /// - `name` - 要移除的渲染目标名称
    pub fn remove_scene_render_target(&mut self, name: &str) {
        self.scene_render_targets.remove(name);
    }

    /// 设置目标帧率
    ///
    /// # 参数
    ///
    /// - `fps` - 目标帧率（FPS）
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
    }

    /// 获取当前目标帧率
    ///
    /// # 返回值
    ///
    /// 当前目标帧率（FPS）
    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    /// 执行帧率限制
    ///
    /// 根据目标帧率计算帧间隔，如果当前帧提前完成则休眠等待。
    fn limit_frame_rate(&self) {
        let frame_duration = Duration::from_secs_f32(1.0 / self.target_fps as f32);
        let elapsed = self.last_frame_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    /// 渲染一帧
    ///
    /// 将 UI 树转换为绘制命令并提交渲染。
    /// 完整流程：帧率限制 → 开始帧 → 构建 RenderContext → 渲染 UI → 绘制 → 呈现。
    ///
    /// # 参数
    ///
    /// - `ui_tree` - 要渲染的 UI 节点树
    pub fn render_frame(&mut self, ui_tree: &gg_ui::UiTree) -> GResult<()> {
        self.limit_frame_rate();

        self.renderer.begin_frame()?;

        let info = self.renderer.surface_info();
        let mut context = RenderContext::new(info.width, info.height);

        UiRenderer::render(ui_tree, &mut context);

        self.renderer.draw(&context)?;
        self.renderer.present()?;

        self.last_frame_time = Instant::now();

        Ok(())
    }

    /// 渲染一帧（含场景渲染回调）
    ///
    /// 先调用场景渲染回调将游戏场景渲染到 RTT 纹理，
    /// 然后渲染编辑器 UI（UI 中可引用场景渲染目标的纹理）。
    /// 完整流程：帧率限制 → 场景渲染回调 → 开始帧 → 构建 RenderContext → 渲染 UI → 绘制 → 呈现。
    ///
    /// # 参数
    ///
    /// - `ui_tree` - 要渲染的 UI 节点树
    /// - `scene_renderer` - 场景渲染回调，接收 WgpuRenderer 和渲染目标映射表的引用
    ///
    /// # 类型参数
    ///
    /// - `F` - 场景渲染回调类型
    pub fn render_frame_with_scene<F>(&mut self, ui_tree: &gg_ui::UiTree, scene_renderer: F) -> GResult<()>
    where
        F: FnOnce(&mut WgpuRenderer, &HashMap<String, RenderTargetInfo>),
    {
        self.limit_frame_rate();

        scene_renderer(&mut self.renderer, &self.scene_render_targets);

        self.renderer.begin_frame()?;

        let info = self.renderer.surface_info();
        let mut context = RenderContext::new(info.width, info.height);

        UiRenderer::render(ui_tree, &mut context);

        self.renderer.draw(&context)?;
        self.renderer.present()?;

        self.last_frame_time = Instant::now();

        Ok(())
    }

    /// 调整渲染表面尺寸
    ///
    /// 当窗口大小改变时调用此方法更新渲染器内部尺寸。
    ///
    /// # 参数
    ///
    /// - `width` - 新的宽度（像素）
    /// - `height` - 新的高度（像素）
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    /// 轮询窗口事件
    ///
    /// 返回并清空内部事件缓冲区中的所有窗口事件。
    pub fn poll_events(&mut self) -> Vec<gg_render::WindowEvent> {
        self.renderer.poll_events()
    }

    /// 检查窗口是否应该关闭
    pub fn should_close(&self) -> bool {
        self.renderer.should_close()
    }

    /// 处理窗口事件
    ///
    /// 根据窗口事件类型执行相应操作。
    /// 目前处理窗口大小改变事件，其他事件暂为空操作。
    ///
    /// # 参数
    ///
    /// - `event` - 窗口事件引用
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized { width, height } => {
                self.resize(*width, *height);
            }
            WindowEvent::CloseRequested => {}
            WindowEvent::Focused => {}
            WindowEvent::Unfocused => {}
            WindowEvent::WindowCreated { .. } => {}
            WindowEvent::WindowDestroyed { .. } => {}
        }
    }
}
