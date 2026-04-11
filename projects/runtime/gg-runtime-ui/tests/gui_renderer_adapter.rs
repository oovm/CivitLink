use std::{cell::RefCell, path::Path, rc::Rc, sync::Arc};

use gg_core::GResult;
use gg_render::{RenderContext, Renderer, SurfaceInfo, TextureId};
use gg_runtime_ui::{GuiEvent, GuiRendererAdapter, Key, KeyModifiers, MouseButton, parse_vx};
use gg_ui::{GuiRenderer, Style, UiNodeData, UiRenderer, UiTree, Widget};
use oak_voc::TemplateNode;

struct MockRenderer {
    surface_info: SurfaceInfo,
}

impl MockRenderer {
    fn new() -> Self {
        Self { surface_info: SurfaceInfo::new(800, 600, "test") }
    }
}

impl Renderer for MockRenderer {
    fn begin_frame(&mut self) -> GResult<()> {
        Ok(())
    }

    fn end_frame(&mut self) -> GResult<()> {
        Ok(())
    }

    fn draw(&mut self, _context: &RenderContext) -> GResult<()> {
        Ok(())
    }

    fn present(&mut self) -> GResult<()> {
        Ok(())
    }

    fn load_texture(&mut self, _path: &Path) -> GResult<TextureId> {
        Ok(TextureId::new(1))
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_info.width = width;
        self.surface_info.height = height;
    }

    fn surface_info(&self) -> &SurfaceInfo {
        &self.surface_info
    }
}

#[test]
fn test_gui_renderer_adapter_new() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let adapter = GuiRendererAdapter::new(renderer);
    assert_eq!(adapter.viewport_width(), 800);
    assert_eq!(adapter.viewport_height(), 600);
}

#[test]
fn test_gui_renderer_adapter_set_viewport_size() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.set_viewport_size(1024, 768);
    assert_eq!(adapter.viewport_width(), 1024);
    assert_eq!(adapter.viewport_height(), 768);
}

#[test]
fn test_gui_renderer_adapter_render() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    let comp = DynamicTestWidget::new("test", TemplateNode::text("Hello"));
    let component: Arc<dyn Widget> = Arc::new(comp);
    adapter.render(component);
}

#[test]
fn test_gui_renderer_adapter_process_events_noop() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert!(events.is_empty());
}

#[test]
fn test_gui_renderer_adapter_update_noop() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.update();
}

#[test]
fn test_push_and_process_mouse_click() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::MouseClick { x: 10.0, y: 20.0, button: MouseButton::Left });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        GuiEvent::MouseClick { x, y, button: MouseButton::Left } if *x == 10.0 && *y == 20.0
    ));
}

#[test]
fn test_push_and_process_key_press() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::KeyPress { key: Key::Enter, modifiers: KeyModifiers::default() });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], GuiEvent::KeyPress { key: Key::Enter, .. }));
}

#[test]
fn test_push_and_process_text_input() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::TextInput { text: "hello".to_string() });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        GuiEvent::TextInput { text } if text == "hello"
    ));
}

#[test]
fn test_push_and_process_mouse_move() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::MouseMove { x: 50.0, y: 60.0 });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        GuiEvent::MouseMove { x, y } if *x == 50.0 && *y == 60.0
    ));
}

#[test]
fn test_multiple_events_batch_processing() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::MouseClick { x: 1.0, y: 2.0, button: MouseButton::Left });
    adapter.push_event(GuiEvent::MouseMove { x: 3.0, y: 4.0 });
    adapter.push_event(GuiEvent::TextInput { text: "a".to_string() });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_drain_events_clears_queue() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::MouseClick { x: 1.0, y: 2.0, button: MouseButton::Left });
    adapter.process_events(None);
    let first = adapter.drain_events();
    assert_eq!(first.len(), 1);
    let second = adapter.drain_events();
    assert!(second.is_empty());
}

#[test]
fn test_process_events_drains_pending() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::MouseClick { x: 1.0, y: 2.0, button: MouseButton::Left });
    adapter.process_events(None);
    adapter.push_event(GuiEvent::MouseMove { x: 5.0, y: 6.0 });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 2);
}

#[test]
fn test_focused_node_default_none() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let adapter = GuiRendererAdapter::new(renderer);
    assert!(adapter.focused_node().is_none());
}

#[test]
fn test_set_focused_node() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.set_focused_node(Some(42));
    assert_eq!(adapter.focused_node(), Some(42));
    adapter.set_focused_node(None);
    assert!(adapter.focused_node().is_none());
}

#[test]
fn test_find_node_at_empty_tree() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let adapter = GuiRendererAdapter::new(renderer);
    assert!(adapter.find_node_at(10.0, 10.0).is_none());
}

#[test]
fn test_parse_vx_integration() {
    let result = parse_vx("<template><div>Hello</div></template>");
    assert!(result.is_ok());
}

/// 测试用动态控件
struct DynamicTestWidget {
    id: String,
    template: TemplateNode,
}

impl DynamicTestWidget {
    fn new(id: &str, template: TemplateNode) -> Self {
        Self { id: id.to_string(), template }
    }
}

impl Widget for DynamicTestWidget {
    fn render_template(&self) -> TemplateNode {
        self.template.clone()
    }

    fn script_setup(&mut self) {}

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, _event: &GuiEvent, _ctx: &mut gg_ui::EventContext) {}

    fn build(&mut self, _tree: &mut UiTree) -> gg_core::GResult<gg_ui::UiNodeId> {
        Err(gg_error::GError::new("not implemented"))
    }

    fn update(&self, _tree: &mut UiTree) {}

    fn node_id(&self) -> Option<gg_ui::UiNodeId> {
        None
    }
}
