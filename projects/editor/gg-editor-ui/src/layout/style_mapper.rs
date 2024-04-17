//! USS 样式属性映射模块
//!
//! 提供将 USS 样式属性解析为布局样式的功能。

use std::collections::HashMap;

use crate::FlexDirection;

use super::{AlignItems, FlexWrap, JustifyContent, LayoutEdge, LayoutStyle};

/// 将 USS 样式属性解析为 LayoutStyle
pub fn map_uss_to_layout_style(uss_properties: &HashMap<String, String>) -> LayoutStyle {
    let mut style = LayoutStyle::default();

    if let Some(v) = uss_properties.get("flex-direction") {
        style.flex_direction = parse_flex_direction(v);
    }
    if let Some(v) = uss_properties.get("justify-content") {
        style.justify_content = parse_justify_content(v);
    }
    if let Some(v) = uss_properties.get("align-items") {
        style.align_items = parse_align_items(v);
    }
    if let Some(v) = uss_properties.get("flex-wrap") {
        style.flex_wrap = parse_flex_wrap(v);
    }
    if let Some(v) = uss_properties.get("flex-grow") {
        style.flex_grow = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("flex-shrink") {
        style.flex_shrink = parse_f32(v).unwrap_or(1.0);
    }
    if let Some(v) = uss_properties.get("flex-basis") {
        style.flex_basis = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("gap") {
        style.gap = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("padding") {
        style.padding = parse_edge(v);
    }
    if let Some(v) = uss_properties.get("padding-left") {
        style.padding.left = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("padding-right") {
        style.padding.right = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("padding-top") {
        style.padding.top = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("padding-bottom") {
        style.padding.bottom = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("margin") {
        style.margin = parse_edge(v);
    }
    if let Some(v) = uss_properties.get("margin-left") {
        style.margin.left = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("margin-right") {
        style.margin.right = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("margin-top") {
        style.margin.top = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("margin-bottom") {
        style.margin.bottom = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("width") {
        style.width = parse_dimension(v);
    }
    if let Some(v) = uss_properties.get("height") {
        style.height = parse_dimension(v);
    }
    if let Some(v) = uss_properties.get("min-width") {
        style.min_width = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("min-height") {
        style.min_height = parse_f32(v).unwrap_or(0.0);
    }
    if let Some(v) = uss_properties.get("max-width") {
        style.max_width = parse_f32(v).unwrap_or(f32::MAX);
    }
    if let Some(v) = uss_properties.get("max-height") {
        style.max_height = parse_f32(v).unwrap_or(f32::MAX);
    }

    style
}

/// 解析 flex-direction 值
fn parse_flex_direction(value: &str) -> FlexDirection {
    match value.trim() {
        "column" => FlexDirection::Column,
        _ => FlexDirection::Row,
    }
}

/// 解析 justify-content 值
fn parse_justify_content(value: &str) -> JustifyContent {
    match value.trim() {
        "flex-end" => JustifyContent::FlexEnd,
        "center" => JustifyContent::Center,
        "space-between" => JustifyContent::SpaceBetween,
        "space-around" => JustifyContent::SpaceAround,
        "space-evenly" => JustifyContent::SpaceEvenly,
        _ => JustifyContent::FlexStart,
    }
}

/// 解析 align-items 值
fn parse_align_items(value: &str) -> AlignItems {
    match value.trim() {
        "flex-end" => AlignItems::FlexEnd,
        "center" => AlignItems::Center,
        "stretch" => AlignItems::Stretch,
        "baseline" => AlignItems::Baseline,
        _ => AlignItems::FlexStart,
    }
}

/// 解析 flex-wrap 值
fn parse_flex_wrap(value: &str) -> FlexWrap {
    match value.trim() {
        "wrap" => FlexWrap::Wrap,
        "wrap-reverse" => FlexWrap::WrapReverse,
        _ => FlexWrap::NoWrap,
    }
}

/// 解析 f32 值
fn parse_f32(value: &str) -> Option<f32> {
    value.trim().trim_end_matches("px").trim().parse().ok()
}

/// 解析尺寸值，auto 返回 -1
fn parse_dimension(value: &str) -> f32 {
    let trimmed = value.trim();
    if trimmed == "auto" { -1.0 } else { parse_f32(trimmed).unwrap_or(-1.0) }
}

/// 解析边距简写值（1~4 个值）
fn parse_edge(value: &str) -> LayoutEdge {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            let v = parse_f32(parts[0]).unwrap_or(0.0);
            LayoutEdge { left: v, right: v, top: v, bottom: v }
        }
        2 => {
            let v_tb = parse_f32(parts[0]).unwrap_or(0.0);
            let v_lr = parse_f32(parts[1]).unwrap_or(0.0);
            LayoutEdge { left: v_lr, right: v_lr, top: v_tb, bottom: v_tb }
        }
        3 => {
            let top = parse_f32(parts[0]).unwrap_or(0.0);
            let lr = parse_f32(parts[1]).unwrap_or(0.0);
            let bottom = parse_f32(parts[2]).unwrap_or(0.0);
            LayoutEdge { left: lr, right: lr, top, bottom }
        }
        4 => LayoutEdge {
            top: parse_f32(parts[0]).unwrap_or(0.0),
            right: parse_f32(parts[1]).unwrap_or(0.0),
            bottom: parse_f32(parts[2]).unwrap_or(0.0),
            left: parse_f32(parts[3]).unwrap_or(0.0),
        },
        _ => LayoutEdge::zero(),
    }
}
