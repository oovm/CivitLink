use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use gg_render::Color;

use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexDirection, FontStyle, LayoutStyle, Overflow, SizeValue, Style},
    widget::Widget,
};

/// 将 TemplateNode 转换为 UiTree
///
/// 递归遍历模板节点树，将每个节点映射为 UiNode 并构建 UiTree 结构。
pub fn template_node_to_ui_tree(node: &oak_voc::TemplateNode) -> UiTree {
    let mut tree = UiTree::new();
    if let Some(root_id) = convert_node(node, &mut tree) {
        tree.set_root(root_id);
    }
    tree
}

fn convert_node(node: &oak_voc::TemplateNode, tree: &mut UiTree) -> Option<UiNodeId> {
    match node {
        oak_voc::TemplateNode::Text(von_str) => {
            if von_str.value.is_empty() {
                return None;
            }
            let style = Style::new().with_font(FontStyle::new());
            let data = UiNodeData::Text { content: von_str.value.clone() };
            let id = tree.create_node("text", style, data);
            Some(id)
        }
        oak_voc::TemplateNode::Element { tag, attributes, children } => {
            let (style, node_id_attr, _node_class) = extract_attributes(attributes, &tag.value);
            let data = match tag.value.as_str() {
                "Text" => {
                    let content = extract_text_content(children);
                    UiNodeData::Text { content }
                }
                "Image" => UiNodeData::Image { texture_id: None, size: None },
                _ => UiNodeData::Container,
            };
            let label = node_id_attr.unwrap_or_else(|| tag.value.clone());
            let id = tree.create_node(label, style, data);
            for child in children {
                if let Some(child_id) = convert_node(child, tree) {
                    tree.add_child(id, child_id);
                }
            }
            Some(id)
        }
    }
}

/// 将 TemplateNode 转换为 UiTree，同时构建 UiNodeId 到组件的映射
///
/// 与 `template_node_to_ui_tree` 类似，但额外跟踪每个 Element 节点对应的子组件，
/// 将映射关系记录到 `component_map` 中。
pub fn template_node_to_ui_tree_mapped(
    node: &oak_voc::TemplateNode,
    component_children: &[Arc<RwLock<dyn Widget>>],
    component_map: &mut HashMap<UiNodeId, Arc<RwLock<dyn Widget>>>,
) -> UiTree {
    let mut tree = UiTree::new();
    if let Some(root_id) = convert_node_mapped(node, &mut tree, component_children, component_map) {
        tree.set_root(root_id);
    }
    tree
}

fn convert_node_mapped(
    node: &oak_voc::TemplateNode,
    tree: &mut UiTree,
    component_children: &[Arc<RwLock<dyn Widget>>],
    component_map: &mut HashMap<UiNodeId, Arc<RwLock<dyn Widget>>>,
) -> Option<UiNodeId> {
    match node {
        oak_voc::TemplateNode::Text(von_str) => {
            if von_str.value.is_empty() {
                return None;
            }
            let style = Style::new().with_font(FontStyle::new());
            let data = UiNodeData::Text { content: von_str.value.clone() };
            let id = tree.create_node("text", style, data);
            Some(id)
        }
        oak_voc::TemplateNode::Element { tag, attributes, children } => {
            let (style, node_id_attr, _node_class) = extract_attributes(attributes, &tag.value);
            let data = match tag.value.as_str() {
                "Text" => {
                    let content = extract_text_content(children);
                    UiNodeData::Text { content }
                }
                "Image" => UiNodeData::Image { texture_id: None, size: None },
                _ => UiNodeData::Container,
            };
            let label = node_id_attr.unwrap_or_else(|| tag.value.clone());
            let id = tree.create_node(label, style, data);

            let mut comp_child_idx = 0;
            for child in children {
                let child_id = match child {
                    oak_voc::TemplateNode::Element { .. } => {
                        if comp_child_idx < component_children.len() {
                            let child_comp = &component_children[comp_child_idx];
                            let grand_children: Vec<Arc<RwLock<dyn Widget>>> = match child_comp.read() {
                                Ok(guard) => guard.children(),
                                Err(_) => Vec::new(),
                            };
                            let cid = convert_node_mapped(child, tree, &grand_children, component_map);
                            if let Some(cid) = cid {
                                component_map.insert(cid, child_comp.clone());
                            }
                            comp_child_idx += 1;
                            cid
                        }
                        else {
                            convert_node(child, tree)
                        }
                    }
                    oak_voc::TemplateNode::Text(_) => convert_node(child, tree),
                };
                if let Some(cid) = child_id {
                    tree.add_child(id, cid);
                }
            }
            Some(id)
        }
    }
}

fn extract_attributes(attributes: &[oak_voc::Attribute], tag: &str) -> (Style, Option<String>, Option<String>) {
    let mut style = default_style_for_tag(tag);
    let mut node_id = None;
    let mut node_class = None;
    for attr in attributes {
        let key = &attr.name.value;
        let value = &attr.value.value;
        match key.as_str() {
            "style" => {
                let inline = parse_inline_style(value);
                style = merge_styles(style, inline);
            }
            "id" => {
                node_id = Some(value.clone());
            }
            "class" => {
                node_class = Some(value.clone());
            }
            "direction" | "orientation" => {
                let dir = match value.as_str() {
                    "row" | "horizontal" => FlexDirection::Row,
                    _ => FlexDirection::Column,
                };
                style.layout.direction = dir;
            }
            "gap" => {
                if let Ok(g) = value.parse::<f32>() {
                    style.layout.gap = g;
                }
            }
            "padding" => {
                let px = value.trim().trim_end_matches("px").trim().parse().unwrap_or(0.0);
                style.layout.padding = px;
            }
            _ => {}
        }
    }
    (style, node_id, node_class)
}

/// 根据标签名返回默认样式
fn default_style_for_tag(tag: &str) -> Style {
    match tag {
        "Button" => Style::new()
            .with_background_color(Color::new(0.2, 0.2, 0.2, 1.0))
            .with_border_color(Color::new(0.5, 0.5, 0.5, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_font(FontStyle::new())
            .with_layout(LayoutStyle::new().with_padding(6.0)),
        "Panel" => Style::new()
            .with_background_color(Color::new(0.15, 0.15, 0.15, 1.0))
            .with_border_color(Color::new(0.3, 0.3, 0.3, 1.0))
            .with_border_width(1.0)
            .with_layout(LayoutStyle::new().with_padding(8.0)),
        "Input" => Style::new()
            .with_background_color(Color::new(0.1, 0.1, 0.1, 1.0))
            .with_border_color(Color::new(0.4, 0.4, 0.4, 1.0))
            .with_border_width(1.0)
            .with_font(FontStyle::new())
            .with_layout(LayoutStyle::new().with_padding(4.0)),
        "ScrollView" => {
            Style::new().with_overflow(Overflow::Clip).with_layout(LayoutStyle::new().with_direction(FlexDirection::Column))
        }
        "Layout" => Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column)),
        "Stack" => Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column)),
        "Text" => Style::new().with_font(FontStyle::new()),
        _ => Style::new(),
    }
}

fn merge_styles(base: Style, overlay: Style) -> Style {
    let mut result = base;
    if overlay.background_color.is_some() {
        result.background_color = overlay.background_color;
    }
    if overlay.border_color.is_some() {
        result.border_color = overlay.border_color;
    }
    if overlay.border_width > 0.0 {
        result.border_width = overlay.border_width;
    }
    if overlay.corner_radius > 0.0 {
        result.corner_radius = overlay.corner_radius;
    }
    if overlay.font.is_some() {
        result.font = overlay.font;
    }
    if overlay.layout.padding != 0.0 {
        result.layout.padding = overlay.layout.padding;
    }
    if overlay.layout.margin != 0.0 {
        result.layout.margin = overlay.layout.margin;
    }
    match overlay.layout.width {
        SizeValue::Px(_) => result.layout.width = overlay.layout.width,
        _ => {}
    }
    match overlay.layout.height {
        SizeValue::Px(_) => result.layout.height = overlay.layout.height,
        _ => {}
    }
    result
}

fn extract_text_content(children: &[oak_voc::TemplateNode]) -> String {
    let mut result = String::new();
    for child in children {
        if let oak_voc::TemplateNode::Text(von_str) = child {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&von_str.value);
        }
    }
    result
}

/// 解析内联样式字符串为 Style
///
/// 支持格式: "key: value; key: value;"
///
/// 已知属性映射:
/// - "color" → 字体颜色
/// - "background-color" / "background" → 背景色
/// - "font-size" → 字体大小
/// - "padding" → 内边距
/// - "margin" → 外边距
/// - "border" → 边框（格式: "1px solid red"）
/// - "width" / "height" → 尺寸
pub fn parse_inline_style(style: &str) -> Style {
    let mut result = Style::new();
    for part in style.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut kv = part.splitn(2, ':');
        let key = kv.next().unwrap_or("").trim();
        let value = kv.next().unwrap_or("").trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        apply_style_property(&mut result, key, value);
    }
    result
}

fn apply_style_property(style: &mut Style, key: &str, value: &str) {
    match key {
        "color" => {
            let color = parse_color(value);
            let font = style.font.take().unwrap_or_else(FontStyle::new);
            style.font = Some(font.with_color(color));
        }
        "background-color" | "background" => {
            style.background_color = Some(parse_color(value));
        }
        "font-size" => {
            let size = parse_px_value(value);
            let font = style.font.take().unwrap_or_else(FontStyle::new);
            style.font = Some(font.with_size(size));
        }
        "padding" => {
            style.layout.padding = parse_px_value(value);
        }
        "margin" => {
            style.layout.margin = parse_px_value(value);
        }
        "border" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if !parts.is_empty() {
                style.border_width = parse_px_value(parts[0]);
            }
            if parts.len() >= 3 {
                style.border_color = Some(parse_color(parts[2]));
            }
        }
        "width" => {
            style.layout.width = SizeValue::Px(parse_px_value(value));
        }
        "height" => {
            style.layout.height = SizeValue::Px(parse_px_value(value));
        }
        _ => {}
    }
}

/// 解析颜色字符串为 Color
///
/// 支持命名颜色（red, green, blue, white, black, transparent）
/// 和十六进制颜色（#RGB, #RRGGBB, #RRGGBBAA）。
pub fn parse_color(value: &str) -> Color {
    match value.trim().to_lowercase().as_str() {
        "red" => Color::RED,
        "green" => Color::GREEN,
        "blue" => Color::BLUE,
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "transparent" => Color::TRANSPARENT,
        hex if hex.starts_with('#') => parse_hex_color(hex),
        _ => Color::WHITE,
    }
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            Color::new(r, g, b, 1.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            Color::new(r, g, b, 1.0)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(0) as f32 / 255.0;
            Color::new(r, g, b, a)
        }
        _ => Color::WHITE,
    }
}

fn parse_px_value(value: &str) -> f32 {
    value.trim().trim_end_matches("px").trim().parse().unwrap_or(0.0)
}
