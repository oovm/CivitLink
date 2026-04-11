#![warn(missing_docs)]

//! GG 引擎编辑器渲染模块
//!
//! 桥接编辑器 UI 树与 WgpuRenderer，
//! 提供编辑器专用的渲染管线封装。

use gg_core::{GResult, WindowEvent};
use gg_render::{RenderContext, Renderer, SurfaceInfo};
use gg_render_wgpu::WgpuRenderer;
use gg_ui::UiRenderer;

/// 编辑器渲染器
///
/// 封装 WgpuRenderer，提供编辑器专用的渲染流程。
/// 负责将 UI 树转换为绘制命令并提交到 GPU 渲染。
pub struct EditorRenderer {
    /// 底层 WGPU 渲染器实例
    renderer: WgpuRenderer,
}

#[cfg(not(target_arch = "wasm32"))]
impl EditorRenderer {
    /// 创建新的编辑器渲染器（桌面平台）
    ///
    /// 使用指定的 winit 事件循环和渲染表面信息创建渲染器。
    ///
    /// # 参数
    ///
    /// - `event_loop` - winit 事件循环引用
    /// - `surface_info` - 渲染表面信息
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>, surface_info: SurfaceInfo) -> GResult<Self> {
        let renderer = WgpuRenderer::new(event_loop, surface_info)?;
        Ok(Self { renderer })
    }
}

impl EditorRenderer {
    /// 渲染一帧
    ///
    /// 将 UI 树转换为绘制命令并提交渲染。
    /// 完整流程：开始帧 → 构建 RenderContext → 渲染 UI → 绘制 → 呈现。
    ///
    /// # 参数
    ///
    /// - `ui_tree` - 要渲染的 UI 节点树
    pub fn render_frame(&mut self, ui_tree: &gg_ui::UiTree) -> GResult<()> {
        self.renderer.begin_frame()?;

        let info = self.renderer.surface_info();
        let mut context = RenderContext::new(info.width, info.height);

        UiRenderer::render(ui_tree, &mut context);

        self.renderer.draw(&context)?;
        self.renderer.present()?;

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
        }
    }
}
