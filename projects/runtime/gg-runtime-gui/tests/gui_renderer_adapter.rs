use gg_runtime_gui::{components::Text, gui_renderer_adapter::{parse_color, parse_inline_style, template_node_to_ui_tree, GuiRendererAdapter, MockRenderer, TemplateNode, UiNodeData}, GuiEvent, Key, KeyModifiers, MouseButton, Style, Color, SizeValue, Renderer, VxComponent};
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use gg_render::{SurfaceInfo, TextureId, RenderContext};
use gg_core::GResult;
use gg_ui::LayoutResult;

struct MockRenderer {
    surface_info: SurfaceInfo,
}

impl MockRenderer {
    fn new() -> Self {
        Self {
            surface_info: SurfaceInfo::new(800, 600, "test"),
        }
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
fn test_template_node_to_ui_tree_text() {
    let node = TemplateNode::Text("Hello".to_string());
    let tree = template_node_to_ui_tree(&node);
    assert!(tree.root().is_some());
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert_eq!(root.label, "text");
    if let UiNodeData::Text { ref content } = root.data {
        assert_eq!(content, "Hello");
    } else {
        panic!("Expected Text node data");
    }
}

#[test]
fn test_template_node_to_ui_tree_empty_text() {
    let node = TemplateNode::Text(String::new());
    let tree = template_node_to_ui_tree(&node);
    assert!(tree.root().is_none());
}

#[test]
fn test_template_node_to_ui_tree_layout() {
    let node = TemplateNode::Element {
        tag: "Layout".to_string(),
        attributes: vec![("id".to_string(), "main-layout".to_string())],
        children: vec![TemplateNode::Text("Child".to_string())],
    };
    let tree = template_node_to_ui_tree(&node);
    assert!(tree.root().is_some());
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert_eq!(root.label, "main-layout");
    assert!(matches!(root.data, UiNodeData::Container));
    assert_eq!(root.children.len(), 1);
}

#[test]
fn test_template_node_to_ui_tree_text_element() {
    let node = TemplateNode::Element {
        tag: "Text".to_string(),
        attributes: vec![("id".to_string(), "title".to_string())],
        children: vec![TemplateNode::Text("Hello World".to_string())],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    if let UiNodeData::Text { ref content } = root.data {
        assert_eq!(content, "Hello World");
    } else {
        panic!("Expected Text node data");
    }
}

#[test]
fn test_template_node_to_ui_tree_button() {
    let node = TemplateNode::Element {
        tag: "Button".to_string(),
        attributes: vec![("id".to_string(), "btn".to_string())],
        children: vec![],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert!(matches!(root.data, UiNodeData::Container));
    assert!(root.style.background_color.is_some());
    assert!(root.style.border_color.is_some());
}

#[test]
fn test_template_node_to_ui_tree_image() {
    let node = TemplateNode::Element {
        tag: "Image".to_string(),
        attributes: vec![("id".to_string(), "img".to_string())],
        children: vec![],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    if let UiNodeData::Image { ref texture_id, ref size } = root.data {
        assert!(texture_id.is_none());
        assert!(size.is_none());
    } else {
        panic!("Expected Image node data");
    }
}

#[test]
fn test_template_node_to_ui_tree_nested() {
    let node = TemplateNode::Element {
        tag: "Layout".to_string(),
        attributes: vec![],
        children: vec![
            TemplateNode::Element {
                tag: "Text".to_string(),
                attributes: vec![],
                children: vec![TemplateNode::Text("Hello".to_string())],
            },
            TemplateNode::Element {
                tag: "Button".to_string(),
                attributes: vec![],
                children: vec![],
            },
        ],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert_eq!(root.children.len(), 2);
}

#[test]
fn test_template_node_to_ui_tree_unknown_tag() {
    let node = TemplateNode::Element {
        tag: "CustomWidget".to_string(),
        attributes: vec![],
        children: vec![],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert!(matches!(root.data, UiNodeData::Container));
}

#[test]
fn test_parse_inline_style_color() {
    let style = parse_inline_style("color: red");
    assert!(style.font.is_some());
    let font = style.font.as_ref().unwrap();
    assert_eq!(font.color, Color::RED);
}

#[test]
fn test_parse_inline_style_background() {
    let style = parse_inline_style("background-color: blue");
    assert_eq!(style.background_color, Some(Color::BLUE));
}

#[test]
fn test_parse_inline_style_background_shorthand() {
    let style = parse_inline_style("background: green");
    assert_eq!(style.background_color, Some(Color::GREEN));
}

#[test]
fn test_parse_inline_style_font_size() {
    let style = parse_inline_style("font-size: 24px");
    assert!(style.font.is_some());
    let font = style.font.as_ref().unwrap();
    assert_eq!(font.size, 24.0);
}

#[test]
fn test_parse_inline_style_padding() {
    let style = parse_inline_style("padding: 10px");
    assert_eq!(style.layout.padding, 10.0);
}

#[test]
fn test_parse_inline_style_margin() {
    let style = parse_inline_style("margin: 5px");
    assert_eq!(style.layout.margin, 5.0);
}

#[test]
fn test_parse_inline_style_border() {
    let style = parse_inline_style("border: 2px solid red");
    assert_eq!(style.border_width, 2.0);
    assert_eq!(style.border_color, Some(Color::RED));
}

#[test]
fn test_parse_inline_style_width_height() {
    let style = parse_inline_style("width: 100px; height: 200px");
    assert_eq!(style.layout.width, SizeValue::Px(100.0));
    assert_eq!(style.layout.height, SizeValue::Px(200.0));
}

#[test]
fn test_parse_inline_style_multiple() {
    let style = parse_inline_style("color: white; background-color: black; font-size: 16px; padding: 8px");
    assert!(style.font.is_some());
    let font = style.font.as_ref().unwrap();
    assert_eq!(font.color, Color::WHITE);
    assert_eq!(font.size, 16.0);
    assert_eq!(style.background_color, Some(Color::BLACK));
    assert_eq!(style.layout.padding, 8.0);
}

#[test]
fn test_parse_inline_style_empty() {
    let style = parse_inline_style("");
    assert!(style.background_color.is_none());
    assert!(style.font.is_none());
}

#[test]
fn test_parse_inline_style_unknown_property() {
    let style = parse_inline_style("unknown: value");
    assert!(style.background_color.is_none());
}

#[test]
fn test_parse_color_named() {
    assert_eq!(parse_color("red"), Color::RED);
    assert_eq!(parse_color("green"), Color::GREEN);
    assert_eq!(parse_color("blue"), Color::BLUE);
    assert_eq!(parse_color("white"), Color::WHITE);
    assert_eq!(parse_color("black"), Color::BLACK);
    assert_eq!(parse_color("transparent"), Color::TRANSPARENT);
}

#[test]
fn test_parse_color_hex_6() {
    let color = parse_color("#ff0000");
    assert_eq!(color, Color::new(1.0, 0.0, 0.0, 1.0));
}

#[test]
fn test_parse_color_hex_3() {
    let color = parse_color("#f00");
    assert_eq!(color, Color::new(1.0, 0.0, 0.0, 1.0));
}

#[test]
fn test_parse_color_hex_8() {
    let color = parse_color("#ff000080");
    assert_eq!(color, Color::new(1.0, 0.0, 0.0, 128.0 / 255.0));
}

#[test]
fn test_parse_color_unknown() {
    assert_eq!(parse_color("unknown"), Color::WHITE);
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
    let text = Text::new("test", "Hello");
    let component: Arc<dyn VxComponent> = Arc::new(text);
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
    adapter.push_event(GuiEvent::MouseClick {
        x: 10.0,
        y: 20.0,
        button: MouseButton::Left,
    });
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
    adapter.push_event(GuiEvent::KeyPress {
        key: Key::Enter,
        modifiers: KeyModifiers::default(),
    });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        GuiEvent::KeyPress { key: Key::Enter, .. }
    ));
}

#[test]
fn test_push_and_process_text_input() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::TextInput {
        text: "hello".to_string(),
    });
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
    adapter.push_event(GuiEvent::MouseClick {
        x: 1.0,
        y: 2.0,
        button: MouseButton::Left,
    });
    adapter.push_event(GuiEvent::MouseMove { x: 3.0, y: 4.0 });
    adapter.push_event(GuiEvent::TextInput {
        text: "a".to_string(),
    });
    adapter.process_events(None);
    let events = adapter.drain_events();
    assert_eq!(events.len(), 3);
}

#[test]
fn test_drain_events_clears_queue() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);
    adapter.push_event(GuiEvent::MouseClick {
        x: 1.0,
        y: 2.0,
        button: MouseButton::Left,
    });
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
    adapter.push_event(GuiEvent::MouseClick {
        x: 1.0,
        y: 2.0,
        button: MouseButton::Left,
    });
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
fn test_find_node_at_with_layout() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);

    let style = Style::new();
    let node_id = adapter.ui_tree.create_node("button", style, UiNodeData::Container);
    adapter.ui_tree.set_root(node_id);
    if let Some(node) = adapter.ui_tree.get_mut(node_id) {
        node.layout_result = Some(LayoutResult::new(0.0, 0.0, 100.0, 50.0));
    }

    assert_eq!(adapter.find_node_at(50.0, 25.0), Some(node_id));
    assert_eq!(adapter.find_node_at(150.0, 25.0), None);
    assert_eq!(adapter.find_node_at(50.0, 75.0), None);
}

#[test]
fn test_find_node_at_invisible_node() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);

    let style = Style::new();
    let node_id = adapter.ui_tree.create_node("hidden", style, UiNodeData::Container);
    adapter.ui_tree.set_root(node_id);
    if let Some(node) = adapter.ui_tree.get_mut(node_id) {
        node.layout_result = Some(LayoutResult::new(0.0, 0.0, 100.0, 50.0));
        node.visible = false;
    }

    assert!(adapter.find_node_at(50.0, 25.0).is_none());
}

#[test]
fn test_find_node_at_nested() {
    let renderer = Rc::new(RefCell::new(MockRenderer::new()));
    let mut adapter = GuiRendererAdapter::new(renderer);

    let parent_style = Style::new();
    let child_style = Style::new();
    let parent_id = adapter.ui_tree.create_node("panel", parent_style, UiNodeData::Container);
    let child_id = adapter.ui_tree.create_node("button", child_style, UiNodeData::Container);
    adapter.ui_tree.add_child(parent_id, child_id);
    adapter.ui_tree.set_root(parent_id);

    if let Some(node) = adapter.ui_tree.get_mut(parent_id) {
        node.layout_result = Some(LayoutResult::new(0.0, 0.0, 200.0, 200.0));
    }
    if let Some(node) = adapter.ui_tree.get_mut(child_id) {
        node.layout_result = Some(LayoutResult::new(10.0, 10.0, 80.0, 40.0));
    }

    assert_eq!(adapter.find_node_at(50.0, 30.0), Some(child_id));
    assert_eq!(adapter.find_node_at(5.0, 5.0), Some(parent_id));
}

#[test]
fn test_template_node_style_attribute() {
    let node = TemplateNode::Element {
        tag: "Text".to_string(),
        attributes: vec![
            ("id".to_string(), "styled".to_string()),
            ("style".to_string(), "color: red; font-size: 24px".to_string()),
        ],
        children: vec![TemplateNode::Text("Styled Text".to_string())],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert!(root.style.font.is_some());
    let font = root.style.font.as_ref().unwrap();
    assert_eq!(font.color, Color::RED);
    assert_eq!(font.size, 24.0);
}

#[test]
fn test_template_node_class_attribute() {
    let node = TemplateNode::Element {
        tag: "Layout".to_string(),
        attributes: vec![
            ("class".to_string(), "container".to_string()),
            ("id".to_string(), "main".to_string()),
        ],
        children: vec![],
    };
    let tree = template_node_to_ui_tree(&node);
    let root = tree.get(tree.root().unwrap()).unwrap();
    assert_eq!(root.label, "main");
}
