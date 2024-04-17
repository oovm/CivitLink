//! 编辑器 UI 示例
//!
//! 展示如何使用 *.widget (VX) 声明式编写编辑器界面，
//! 通过 winit + WgpuRenderer 实现完整渲染管线。

use std::{cell::RefCell, rc::Rc};

use gg_core::GResult;
use gg_render::{RenderContext, Renderer, SurfaceInfo};
use gg_render_wgpu::WgpuRenderer;
use gg_runtime_ui::parse_vx;
use gg_ui::{LayoutEngine, UiRenderer, UiTree, template_node_to_ui_tree};
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const WINDOW_TITLE: &str = "GG Engine Editor UI Example";

fn load_ui_tree() -> GResult<UiTree> {
    let vx_source = std::fs::read_to_string("examples/editor-ui-example/ui/main.vx")
        .or_else(|_| std::fs::read_to_string("ui/main.vx"))
        .map_err(|e| gg_core::GError { kind: gg_core::GErrorKind::Io, message: format!("Failed to read main.vx: {}", e) })?;

    let document = parse_vx(&vx_source).map_err(|e| gg_core::GError {
        kind: gg_core::GErrorKind::Other,
        message: format!("Failed to parse main.vx: {}", e),
    })?;

    let template = document.template.unwrap_or(oak_voc::TemplateNode::Text(String::new()));
    let ui_tree = template_node_to_ui_tree(&template);

    Ok(ui_tree)
}

fn main() -> GResult<()> {
    let event_loop = EventLoop::new().map_err(|e| gg_core::GError {
        kind: gg_core::GErrorKind::Platform,
        message: format!("Failed to create event loop: {}", e),
    })?;

    let surface_info = SurfaceInfo::new(WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE.to_string());
    let mut renderer = WgpuRenderer::new(&event_loop, surface_info)?;

    let mut ui_tree = load_ui_tree()?;

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => {
                renderer.handle_window_event(&event);
                match &event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if *state == ElementState::Pressed && *button == MouseButton::Left {
                            let _ = button;
                        }
                    }
                    _ => {}
                }
                if renderer.should_close() {
                    elwt.exit();
                }
            }
            Event::AboutToWait => {
                if let Err(_) = render_frame(&mut renderer, &mut ui_tree) {
                    elwt.exit();
                }
            }
            _ => {}
        })
        .map_err(|e| gg_core::GError { kind: gg_core::GErrorKind::Runtime, message: format!("Event loop error: {}", e) })?;

    Ok(())
}

fn render_frame(renderer: &mut WgpuRenderer, ui_tree: &mut UiTree) -> GResult<()> {
    renderer.begin_frame()?;

    let width = renderer.surface_info().width as f32;
    let height = renderer.surface_info().height as f32;

    let mut context = RenderContext::new(width as u32, height as u32);

    LayoutEngine::compute(ui_tree, width, height);
    UiRenderer::render(ui_tree, &mut context);

    renderer.draw(&context)?;
    renderer.end_frame()?;
    renderer.present()?;

    Ok(())
}
