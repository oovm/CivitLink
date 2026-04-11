use gg_runtime_gui::{ComponentLifecycle, DynamicVxComponent, EventContext, EventPhase, GuiEvent, MouseButton, VxDocument};
use oak_voc::{ScriptAst, StyleAst, StyleRule, TemplateNode};

#[test]
fn test_dynamic_vx_component_new() {
    let comp = DynamicVxComponent::new("test-id");
    assert_eq!(comp.get_id(), "test-id");
    assert_eq!(comp.render_template(), TemplateNode::Text(String::new()));
    assert_eq!(comp.get_style(), None);
    assert_eq!(comp.script_source(), None);
    assert_eq!(comp.lifecycle(), ComponentLifecycle::Created);
}

#[test]
fn test_dynamic_vx_component_render_template_with_node() {
    let template = TemplateNode::Element {
        tag: "div".to_string(),
        attributes: vec![],
        children: vec![TemplateNode::Text("Hello".to_string())],
    };
    let comp = DynamicVxComponent::from_document(
        "test",
        Some(template.clone()),
        None,
        None,
    );
    assert_eq!(comp.render_template(), template);
}

#[test]
fn test_dynamic_vx_component_render_template_none() {
    let comp = DynamicVxComponent::from_document("test", None, None, None);
    assert_eq!(comp.render_template(), TemplateNode::Text(String::new()));
}

#[test]
fn test_dynamic_vx_component_get_style_some() {
    let comp = DynamicVxComponent::from_document(
        "test",
        None,
        Some(".box { color: red; }".to_string()),
        None,
    );
    assert_eq!(comp.get_style(), Some(".box { color: red; }"));
}

#[test]
fn test_dynamic_vx_component_get_style_empty() {
    let comp = DynamicVxComponent::from_document("test", None, Some(String::new()), None);
    assert_eq!(comp.get_style(), None);
}

#[test]
fn test_dynamic_vx_component_get_style_none() {
    let comp = DynamicVxComponent::from_document("test", None, None, None);
    assert_eq!(comp.get_style(), None);
}

#[test]
fn test_dynamic_vx_component_script_setup() {
    let mut comp = DynamicVxComponent::from_document(
        "test",
        None,
        None,
        Some("let x = 1;".to_string()),
    );
    comp.script_setup();
    assert_eq!(comp.script_source(), Some("let x = 1;"));
}

#[test]
fn test_dynamic_vx_component_handle_event_noop() {
    let mut comp = DynamicVxComponent::new("test");
    let event = GuiEvent::MouseClick {
        x: 10.0,
        y: 20.0,
        button: MouseButton::Left,
    };
    comp.handle_event(&event, &mut EventContext::new(EventPhase::AtTarget));
}

#[test]
fn test_dynamic_vx_component_lifecycle() {
    let mut comp = DynamicVxComponent::new("test");
    assert_eq!(comp.lifecycle(), ComponentLifecycle::Created);

    comp.on_mount();
    assert_eq!(comp.lifecycle(), ComponentLifecycle::Mounted);

    comp.on_update();
    assert_eq!(comp.lifecycle(), ComponentLifecycle::Updated);

    comp.on_cleanup();
    assert_eq!(comp.lifecycle(), ComponentLifecycle::Unmounted);
}

#[test]
fn test_vx_document_to_component_empty() {
    let doc = VxDocument {
        template: None,
        script: None,
        style: None,
    };
    let comp = doc.to_component();
    assert_eq!(comp.get_id(), "vx-component");
    assert_eq!(comp.render_template(), TemplateNode::Text(String::new()));
    assert_eq!(comp.get_style(), None);
    assert_eq!(comp.script_source(), None);
}

#[test]
fn test_vx_document_to_component_full() {
    let doc = VxDocument {
        template: Some(TemplateNode::Element {
            tag: "div".to_string(),
            attributes: vec![("class".to_string(), "container".to_string())],
            children: vec![TemplateNode::Text("Hello".to_string())],
        }),
        script: Some(ScriptAst {
            raw_source: "let x = 1;".to_string(),
        }),
        style: Some(StyleAst {
            rules: vec![StyleRule {
                selector: ".container".to_string(),
                properties: vec![
                    ("color".to_string(), "red".to_string()),
                    ("margin".to_string(), "10px".to_string()),
                ],
            }],
        }),
    };
    let comp = doc.to_component();
    assert_eq!(comp.get_id(), "vx-component");
    assert_eq!(
        comp.render_template(),
        TemplateNode::Element {
            tag: "div".to_string(),
            attributes: vec![("class".to_string(), "container".to_string())],
            children: vec![TemplateNode::Text("Hello".to_string())],
        }
    );
    assert_eq!(comp.get_style(), Some(".container { color: red; margin: 10px; }"));
    assert_eq!(comp.script_source(), Some("let x = 1;"));
}

#[test]
fn test_vx_document_to_component_multiple_style_rules() {
    let doc = VxDocument {
        template: None,
        script: None,
        style: Some(StyleAst {
            rules: vec![
                StyleRule {
                    selector: ".a".to_string(),
                    properties: vec![("color".to_string(), "blue".to_string())],
                },
                StyleRule {
                    selector: ".b".to_string(),
                    properties: vec![("margin".to_string(), "5px".to_string())],
                },
            ],
        }),
    };
    let comp = doc.to_component();
    assert_eq!(
        comp.get_style(),
        Some(".a { color: blue; }\n.b { margin: 5px; }")
    );
}

#[test]
fn test_vx_document_to_component_template_only() {
    let doc = VxDocument {
        template: Some(TemplateNode::Text("Hello World".to_string())),
        script: None,
        style: None,
    };
    let comp = doc.to_component();
    assert_eq!(comp.render_template(), TemplateNode::Text("Hello World".to_string()));
    assert_eq!(comp.get_style(), None);
    assert_eq!(comp.script_source(), None);
}

#[test]
fn test_vx_document_to_component_script_only() {
    let doc = VxDocument {
        template: None,
        script: Some(ScriptAst {
            raw_source: "function hello() {}".to_string(),
        }),
        style: None,
    };
    let comp = doc.to_component();
    assert_eq!(comp.render_template(), TemplateNode::Text(String::new()));
    assert_eq!(comp.script_source(), Some("function hello() {}"));
}

#[test]
fn test_vx_document_to_component_style_only() {
    let doc = VxDocument {
        template: None,
        script: None,
        style: Some(StyleAst {
            rules: vec![StyleRule {
                selector: ".title".to_string(),
                properties: vec![("font-size".to_string(), "16px".to_string())],
            }],
        }),
    };
    let comp = doc.to_component();
    assert_eq!(comp.get_style(), Some(".title { font-size: 16px; }"));
}

#[test]
fn test_vx_document_to_component_empty_style_rules() {
    let doc = VxDocument {
        template: None,
        script: None,
        style: Some(StyleAst { rules: vec![] }),
    };
    let comp = doc.to_component();
    assert_eq!(comp.get_style(), None);
}
