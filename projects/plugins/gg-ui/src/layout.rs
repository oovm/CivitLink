use crate::{
    node::{UiNodeId, UiTree},
    style::{FlexAlign, FlexDirection, SizeValue},
};

/// 布局计算结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutResult {
    /// 计算后的位置 X
    pub x: f32,
    /// 计算后的位置 Y
    pub y: f32,
    /// 计算后的宽度
    pub width: f32,
    /// 计算后的高度
    pub height: f32,
}

impl LayoutResult {
    /// 创建布局结果
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// 布局引擎
///
/// 实现简化的 Flexbox 布局算法，支持行/列方向、对齐、换行等功能。
pub struct LayoutEngine;

impl LayoutEngine {
    /// 计算整棵树的布局
    ///
    /// 从根节点开始，递归计算每个节点的位置和尺寸。
    pub fn compute(tree: &mut UiTree, available_width: f32, available_height: f32) {
        if let Some(root_id) = tree.root() {
            Self::compute_node(tree, root_id, 0.0, 0.0, available_width, available_height);
        }
    }

    /// 计算子树布局
    ///
    /// 从指定节点开始，递归计算该节点及其所有后代的布局。
    /// 节点的位置从 (0, 0) 开始计算。
    ///
    /// # 参数
    ///
    /// - `tree` - UI 节点树
    /// - `node_id` - 起始节点 ID
    /// - `available_width` - 可用宽度
    /// - `available_height` - 可用高度
    pub fn compute_subtree(tree: &mut UiTree, node_id: UiNodeId, available_width: f32, available_height: f32) {
        Self::compute_node(tree, node_id, 0.0, 0.0, available_width, available_height);
    }

    /// 计算单个节点的布局（递归）
    fn compute_node(tree: &mut UiTree, node_id: UiNodeId, x: f32, y: f32, available_width: f32, available_height: f32) {
        let (style, children, data) = {
            let node = tree.get(node_id);
            match node {
                Some(n) => (n.style.clone(), n.children.clone(), n.data.clone()),
                None => return,
            }
        };

        let layout = &style.layout;
        let padding = layout.padding;
        let margin = layout.margin;
        let gap = layout.gap;

        let content_x = x + margin + padding;
        let content_y = y + margin + padding;
        let content_available_w = available_width - 2.0 * margin - 2.0 * padding;
        let content_available_h = available_height - 2.0 * margin - 2.0 * padding;

        let node_width = Self::resolve_size(layout.width, content_available_w, available_width);
        let node_height = Self::resolve_size(layout.height, content_available_h, available_height);

        let min_w = match layout.min_width {
            SizeValue::Px(v) => v,
            SizeValue::Percent(p) => p * available_width,
            SizeValue::Auto => 0.0,
        };
        let min_h = match layout.min_height {
            SizeValue::Px(v) => v,
            SizeValue::Percent(p) => p * available_height,
            SizeValue::Auto => 0.0,
        };

        let final_w = node_width.max(min_w);
        let final_h = node_height.max(min_h);

        let inner_w = final_w - 2.0 * padding;
        let inner_h = final_h - 2.0 * padding;

        let children = children;
        let is_row = layout.direction == FlexDirection::Row;

        let mut cursor_main;
        let mut cursor_cross = 0.0f32;
        let mut line_max_cross = 0.0f32;
        let mut line_children_count = 0usize;
        let mut total_main = 0.0f32;

        let child_sizes: Vec<(f32, f32)> = children
            .iter()
            .map(|&child_id| {
                let child_node = tree.get(child_id);
                match child_node {
                    Some(cn) => {
                        let cw = Self::resolve_size(cn.style.layout.width, inner_w, inner_w);
                        let ch = Self::resolve_size(cn.style.layout.height, inner_h, inner_h);
                        (cw, ch)
                    }
                    None => (0.0, 0.0),
                }
            })
            .collect();

        for (cw, ch) in &child_sizes {
            let main = if is_row { *cw } else { *ch };
            total_main += main;
        }
        if children.len() > 1 {
            total_main += gap * (children.len() - 1) as f32;
        }

        let justify_offset = Self::compute_justify_offset(
            layout.justify_content,
            if is_row { inner_w } else { inner_h },
            total_main,
            children.len(),
        );

        cursor_main = justify_offset;

        let mut line_start_idx = 0usize;

        for (i, &child_id) in children.iter().enumerate() {
            let (cw, ch) = child_sizes[i];
            let child_main = if is_row { cw } else { ch };
            let child_cross = if is_row { ch } else { cw };

            let needs_wrap = layout.wrap && i > 0 && cursor_main + child_main > if is_row { inner_w } else { inner_h };

            if needs_wrap {
                Self::apply_align_for_line(
                    tree,
                    &children,
                    line_start_idx,
                    i,
                    is_row,
                    layout.align_items,
                    line_max_cross,
                    cursor_cross,
                    content_x,
                    content_y,
                    inner_w,
                    inner_h,
                );
                cursor_cross += line_max_cross + gap;
                line_max_cross = 0.0;
                cursor_main = justify_offset;
                line_start_idx = i;
                line_children_count = 0;
            }

            let child_x = if is_row { content_x + cursor_main } else { content_x };
            let child_y = if is_row { content_y + cursor_cross } else { content_y + cursor_main };

            let child_available_w = if is_row { child_main } else { inner_w };
            let child_available_h = if is_row { inner_h } else { child_main };

            Self::compute_node(tree, child_id, child_x, child_y, child_available_w, child_available_h);

            cursor_main += child_main + gap;
            line_max_cross = line_max_cross.max(child_cross);
            line_children_count += 1;
        }

        if line_children_count > 0 {
            Self::apply_align_for_line(
                tree,
                &children,
                line_start_idx,
                children.len(),
                is_row,
                layout.align_items,
                line_max_cross,
                cursor_cross,
                content_x,
                content_y,
                inner_w,
                inner_h,
            );
        }

        let auto_w = matches!(layout.width, SizeValue::Auto);
        let auto_h = matches!(layout.height, SizeValue::Auto);

        let computed_w = if auto_w {
            let content_size = Self::measure_content_width(tree, &children, is_row, gap);
            content_size + 2.0 * padding
        }
        else {
            final_w
        };

        let computed_h = if auto_h {
            let content_size = Self::measure_content_height(tree, &children, is_row, gap);
            content_size + 2.0 * padding
        }
        else {
            final_h
        };

        if let Some(node) = tree.get_mut(node_id) {
            node.layout_result = Some(LayoutResult::new(x, y, computed_w, computed_h));
        }

        let _ = data;
    }

    /// 解析尺寸值
    fn resolve_size(value: SizeValue, available: f32, parent_available: f32) -> f32 {
        match value {
            SizeValue::Auto => available.max(0.0),
            SizeValue::Px(v) => v,
            SizeValue::Percent(p) => (p * parent_available).max(0.0),
        }
    }

    /// 计算主轴对齐偏移
    fn compute_justify_offset(align: FlexAlign, container_size: f32, content_size: f32, count: usize) -> f32 {
        match align {
            FlexAlign::Start => 0.0,
            FlexAlign::Center => (container_size - content_size).max(0.0) / 2.0,
            FlexAlign::End => (container_size - content_size).max(0.0),
            FlexAlign::SpaceBetween => {
                if count <= 1 {
                    0.0
                }
                else {
                    0.0
                }
            }
        }
    }

    /// 为一行子节点应用交叉轴对齐
    #[allow(clippy::too_many_arguments)]
    fn apply_align_for_line(
        tree: &mut UiTree,
        children: &[UiNodeId],
        start: usize,
        end: usize,
        is_row: bool,
        align: FlexAlign,
        line_cross_size: f32,
        line_cross_offset: f32,
        _content_x: f32,
        _content_y: f32,
        _inner_w: f32,
        inner_h: f32,
    ) {
        let line_height = line_cross_size;
        for &child_id in &children[start..end] {
            let child_result = tree.get(child_id).and_then(|n| n.layout_result);
            if let Some(result) = child_result {
                let child_cross = if is_row { result.height } else { result.width };
                let cross_offset = match align {
                    FlexAlign::Start => 0.0,
                    FlexAlign::Center => (line_height - child_cross) / 2.0,
                    FlexAlign::End => line_height - child_cross,
                    FlexAlign::SpaceBetween => 0.0,
                };
                if let Some(node) = tree.get_mut(child_id) {
                    if let Some(ref mut lr) = node.layout_result {
                        if is_row {
                            lr.y += line_cross_offset + cross_offset;
                        }
                        else {
                            lr.x += line_cross_offset + cross_offset;
                        }
                    }
                }
            }
        }
        let _ = inner_h;
    }

    /// 测量子节点在主轴方向上的总宽度
    fn measure_content_width(tree: &UiTree, children: &[UiNodeId], is_row: bool, gap: f32) -> f32 {
        if is_row {
            let mut total = 0.0f32;
            for (i, &child_id) in children.iter().enumerate() {
                let w = tree.get(child_id).and_then(|n| n.layout_result).map(|r| r.width).unwrap_or(0.0);
                total += w;
                if i > 0 {
                    total += gap;
                }
            }
            total
        }
        else {
            children
                .iter()
                .filter_map(|&child_id| tree.get(child_id).and_then(|n| n.layout_result).map(|r| r.width))
                .fold(0.0f32, f32::max)
        }
    }

    /// 测量子节点在主轴方向上的总高度
    fn measure_content_height(tree: &UiTree, children: &[UiNodeId], is_row: bool, gap: f32) -> f32 {
        if !is_row {
            let mut total = 0.0f32;
            for (i, &child_id) in children.iter().enumerate() {
                let h = tree.get(child_id).and_then(|n| n.layout_result).map(|r| r.height).unwrap_or(0.0);
                total += h;
                if i > 0 {
                    total += gap;
                }
            }
            total
        }
        else {
            children
                .iter()
                .filter_map(|&child_id| tree.get(child_id).and_then(|n| n.layout_result).map(|r| r.height))
                .fold(0.0f32, f32::max)
        }
    }

    /// 估算文本节点的固有尺寸
    pub fn measure_text(text: &str, font_size: f32, line_height: f32, max_width: Option<f32>) -> (f32, f32) {
        let char_width = font_size * 0.6;
        if let Some(max_w) = max_width {
            let chars_per_line = (max_w / char_width).max(1.0) as usize;
            let lines = (text.len().max(1) + chars_per_line - 1) / chars_per_line;
            let w = text.len() as f32 * char_width;
            let h = lines as f32 * font_size * line_height;
            (w.min(max_w), h)
        }
        else {
            let w = text.len() as f32 * char_width;
            let h = font_size * line_height;
            (w, h)
        }
    }
}
