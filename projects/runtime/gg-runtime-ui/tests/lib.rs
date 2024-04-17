use gg_runtime_ui::{DynamicWidget, EventContext, EventPhase, GuiEvent, MouseButton, WidgetLifecycle};
use oak_voc::TemplateNode;

#[test]
fn test_dynamic_widget_new() {
    let comp = DynamicWidget::new("test-id");
    assert_eq!(comp.get_id(), "test-id");
    assert_eq!(comp.render_template(), TemplateNode::text(String::new()));
    assert_eq!(comp.get_style(), None);
    assert_eq!(comp.script_source(), None);
    assert_eq!(comp.lifecycle(), WidgetLifecycle::Created);
}

#[test]
fn test_dynamic_widget_render_template_with_node() {
    let template = TemplateNode::element("div", vec![], vec![TemplateNode::text("Hello")]);
    let comp = DynamicWidget::from_document("test", Some(template.clone()), None, None);
    assert_eq!(comp.render_template(), template);
}

#[test]
fn test_dynamic_widget_render_template_none() {
    let comp = DynamicWidget::from_document("test", None, None, None);
    assert_eq!(comp.render_template(), TemplateNode::text(String::new()));
}

#[test]
fn test_dynamic_widget_get_style_some() {
    let comp = DynamicWidget::from_document("test", None, Some(".box { color: red; }".to_string()), None);
    assert_eq!(comp.get_style(), Some(".box { color: red; }"));
}

#[test]
fn test_dynamic_widget_get_style_empty() {
    let comp = DynamicWidget::from_document("test", None, Some(String::new()), None);
    assert_eq!(comp.get_style(), None);
}

#[test]
fn test_dynamic_widget_get_style_none() {
    let comp = DynamicWidget::from_document("test", None, None, None);
    assert_eq!(comp.get_style(), None);
}

#[test]
fn test_dynamic_widget_script_setup() {
    let mut comp = DynamicWidget::from_document("test", None, None, Some("let x = 1;".to_string()));
    comp.script_setup();
    assert_eq!(comp.script_source(), Some("let x = 1;"));
}

#[test]
fn test_dynamic_widget_handle_event_noop() {
    let mut comp = DynamicWidget::new("test");
    let event = GuiEvent::MouseClick { x: 10.0, y: 20.0, button: MouseButton::Left };
    comp.handle_event(&event, &mut EventContext::new(EventPhase::AtTarget));
}

#[test]
fn test_dynamic_widget_lifecycle() {
    let mut comp = DynamicWidget::new("test");
    assert_eq!(comp.lifecycle(), WidgetLifecycle::Created);

    comp.on_mount();
    assert_eq!(comp.lifecycle(), WidgetLifecycle::Mounted);

    comp.on_update();
    assert_eq!(comp.lifecycle(), WidgetLifecycle::Updated);

    comp.on_cleanup();
    assert_eq!(comp.lifecycle(), WidgetLifecycle::Unmounted);
}

#[test]
fn test_parse_vx() {
    let result = gg_runtime_ui::parse_vx("<template><div>Hello</div></template>");
    assert!(result.is_ok());
    let doc = result.unwrap();
    assert!(doc.template.is_some());
}
