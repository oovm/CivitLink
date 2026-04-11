//! Flexbox 布局引擎模块
//!
//! 实现核心 Flexbox 布局算法，支持主轴/交叉轴布局、flex-grow/shrink、
//! justify-content、align-items、flex-wrap、gap 间距、padding/margin 等特性。
//! 布局计算仅在脏标记触发时执行，静止时不会重算。

use std::collections::HashSet;

use gg_ui::DpiScale;

use crate::FlexDirection;

use super::{AlignItems, FlexWrap, JustifyContent, LayoutNode, LayoutResult, LayoutStyle};

/// Flexbox 布局引擎
pub struct FlexLayoutEngine {
    /// 根节点
    pub root: Option<LayoutNode>,
    /// 脏节点 ID 集合
    dirty_nodes: HashSet<String>,
}

impl FlexLayoutEngine {
    /// 创建新的布局引擎
    pub fn new() -> Self {
        Self { root: None, dirty_nodes: HashSet::new() }
    }

    /// 设置根节点，标记为脏
    pub fn set_root(&mut self, node: LayoutNode) {
        let mut node = node;
        node.mark_subtree_dirty();
        node.collect_dirty_ids(&mut self.dirty_nodes);
        self.root = Some(node);
    }

    /// 标记指定节点为脏，并向上传播到父级链
    pub fn mark_dirty(&mut self, node_id: &str) {
        self.dirty_nodes.insert(node_id.to_string());
        if let Some(ref mut root) = self.root {
            Self::mark_dirty_upward(root, node_id);
        }
    }

    /// 从目标节点向上传播脏标记到所有祖先节点
    fn mark_dirty_upward(node: &mut LayoutNode, target_id: &str) -> bool {
        let mut found = node.id == target_id;
        for child in &mut node.children {
            if Self::mark_dirty_upward(child, target_id) {
                found = true;
            }
        }
        if found {
            node.is_dirty = true;
        }
        found
    }

    /// 仅在存在脏节点时执行布局计算，计算完成后清除脏标记
    pub fn compute_layout(&mut self) {
        if !self.has_dirty_nodes() {
            return;
        }
        if let Some(ref mut root) = self.root {
            Self::compute_node_layout(root, 0.0, 0.0, None, None);
        }
        self.clear_all_dirty();
    }

    /// 带 DPI 缩放的布局计算
    ///
    /// 先执行标准布局计算，然后将所有布局结果从物理像素转换回逻辑像素
    pub fn compute_layout_with_dpi(&mut self, dpi: DpiScale) {
        if !self.has_dirty_nodes() {
            return;
        }
        self.compute_layout();
        if let Some(ref mut root) = self.root {
            Self::scale_layout_results(root, &dpi);
        }
    }

    /// 获取指定节点的布局结果
    pub fn get_layout(&self, node_id: &str) -> Option<&LayoutResult> {
        self.root.as_ref().and_then(|root| root.find_node(node_id)).map(|node| &node.computed)
    }

    /// 检查是否有脏节点
    pub fn has_dirty_nodes(&self) -> bool {
        if !self.dirty_nodes.is_empty() {
            return true;
        }
        self.root.as_ref().map_or(false, |root| Self::has_dirty_recursive(root))
    }

    /// 递归检查是否有脏节点
    fn has_dirty_recursive(node: &LayoutNode) -> bool {
        if node.is_dirty {
            return true;
        }
        node.children.iter().any(Self::has_dirty_recursive)
    }

    /// 清除所有脏标记
    pub fn clear_all_dirty(&mut self) {
        self.dirty_nodes.clear();
        if let Some(ref mut root) = self.root {
            root.clear_dirty_recursive();
        }
    }

    /// 递归缩放布局结果，将物理像素转换为逻辑像素
    fn scale_layout_results(node: &mut LayoutNode, dpi: &DpiScale) {
        node.computed.x = dpi.physical_to_logical(node.computed.x);
        node.computed.y = dpi.physical_to_logical(node.computed.y);
        node.computed.width = dpi.physical_to_logical(node.computed.width);
        node.computed.height = dpi.physical_to_logical(node.computed.height);
        for child in &mut node.children {
            Self::scale_layout_results(child, dpi);
        }
    }

    /// 计算单个节点的布局
    fn compute_node_layout(
        node: &mut LayoutNode,
        parent_x: f32,
        parent_y: f32,
        container_width: Option<f32>,
        container_height: Option<f32>,
    ) {
        let style = node.style.clone();

        let node_width =
            if style.is_width_auto() { container_width.unwrap_or(0.0) - style.margin.horizontal() } else { style.width };
        let node_width = style.clamp_width(node_width.max(0.0));

        let node_height =
            if style.is_height_auto() { container_height.unwrap_or(0.0) - style.margin.vertical() } else { style.height };
        let node_height = style.clamp_height(node_height.max(0.0));

        node.computed.x = parent_x + style.margin.left;
        node.computed.y = parent_y + style.margin.top;
        node.computed.width = node_width;
        node.computed.height = node_height;

        if node.children.is_empty() {
            return;
        }

        let inner_x = node.computed.x + style.padding.left;
        let inner_y = node.computed.y + style.padding.top;
        let inner_width = (node_width - style.padding.horizontal()).max(0.0);
        let inner_height = (node_height - style.padding.vertical()).max(0.0);

        match style.flex_wrap {
            FlexWrap::NoWrap => {
                Self::layout_single_line(&mut node.children, &style, inner_x, inner_y, inner_width, inner_height);
            }
            FlexWrap::Wrap | FlexWrap::WrapReverse => {
                Self::layout_wrap(&mut node.children, &style, inner_x, inner_y, inner_width, inner_height);
            }
        }
    }

    /// 单行布局（NoWrap 模式）
    fn layout_single_line(
        children: &mut [LayoutNode],
        style: &LayoutStyle,
        inner_x: f32,
        inner_y: f32,
        inner_width: f32,
        inner_height: f32,
    ) {
        let is_row = style.flex_direction == FlexDirection::Row;
        let main_size = if is_row { inner_width } else { inner_height };
        let cross_size = if is_row { inner_height } else { inner_width };

        let total_gap = if children.len() > 1 { style.gap * (children.len() - 1) as f32 } else { 0.0 };

        let mut base_main_sizes: Vec<f32> = Vec::with_capacity(children.len());
        let mut total_base: f32 = 0.0;
        let mut total_grow: f32 = 0.0;
        let mut total_shrink: f32 = 0.0;

        for child in children.iter() {
            let child_style = &child.style;
            let basis = if child_style.flex_basis > 0.0 {
                child_style.flex_basis
            }
            else if is_row {
                if child_style.is_width_auto() { 0.0 } else { child_style.width }
            }
            else {
                if child_style.is_height_auto() { 0.0 } else { child_style.height }
            };

            let margin_main = if is_row { child_style.margin.horizontal() } else { child_style.margin.vertical() };

            base_main_sizes.push(basis);
            total_base += basis + margin_main;

            if child_style.flex_grow > 0.0 {
                total_grow += child_style.flex_grow;
            }
            if child_style.flex_shrink > 0.0 {
                total_shrink += child_style.flex_shrink * basis;
            }
        }

        total_base += total_gap;

        let free_space = main_size - total_base;
        let final_main_sizes = Self::distribute_flex(children, &base_main_sizes, free_space, total_grow, total_shrink, is_row);

        Self::position_children(
            children,
            &final_main_sizes,
            style,
            inner_x,
            inner_y,
            inner_width,
            inner_height,
            main_size,
            cross_size,
            is_row,
        );
    }

    /// 换行布局（Wrap / WrapReverse 模式）
    fn layout_wrap(
        children: &mut [LayoutNode],
        style: &LayoutStyle,
        inner_x: f32,
        inner_y: f32,
        inner_width: f32,
        inner_height: f32,
    ) {
        let is_row = style.flex_direction == FlexDirection::Row;
        let main_size = if is_row { inner_width } else { inner_height };
        let cross_size = if is_row { inner_height } else { inner_width };

        let mut lines: Vec<Vec<usize>> = Vec::new();
        let mut current_line: Vec<usize> = Vec::new();
        let mut current_main: f32 = 0.0;

        for (i, child) in children.iter().enumerate() {
            let child_style = &child.style;
            let basis = if child_style.flex_basis > 0.0 {
                child_style.flex_basis
            }
            else if is_row {
                if child_style.is_width_auto() { 0.0 } else { child_style.width }
            }
            else {
                if child_style.is_height_auto() { 0.0 } else { child_style.height }
            };

            let margin_main = if is_row { child_style.margin.horizontal() } else { child_style.margin.vertical() };

            let item_size = basis + margin_main;
            let gap_if_not_first = if current_line.is_empty() { 0.0 } else { style.gap };

            if !current_line.is_empty() && current_main + gap_if_not_first + item_size > main_size {
                lines.push(current_line.clone());
                current_line.clear();
                current_main = 0.0;
            }

            if current_main > 0.0 {
                current_main += style.gap;
            }
            current_main += item_size;
            current_line.push(i);
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if style.flex_wrap == FlexWrap::WrapReverse {
            lines.reverse();
        }

        let num_lines = lines.len();
        let total_cross_gap = if num_lines > 1 { style.gap * (num_lines - 1) as f32 } else { 0.0 };

        let line_cross_total = if cross_size > 0.0 { (cross_size - total_cross_gap) / num_lines as f32 } else { 0.0 };

        let mut cross_offset: f32 = 0.0;

        for line_indices in &lines {
            let line_main_size = main_size;
            let line_cross_size = line_cross_total.max(0.0);

            let mut base_sizes: Vec<f32> = Vec::with_capacity(line_indices.len());
            let mut total_base: f32 = 0.0;
            let mut total_grow: f32 = 0.0;
            let mut total_shrink: f32 = 0.0;

            for &idx in line_indices {
                let child = &children[idx];
                let child_style = &child.style;
                let basis = if child_style.flex_basis > 0.0 {
                    child_style.flex_basis
                }
                else if is_row {
                    if child_style.is_width_auto() { 0.0 } else { child_style.width }
                }
                else {
                    if child_style.is_height_auto() { 0.0 } else { child_style.height }
                };

                let margin_main = if is_row { child_style.margin.horizontal() } else { child_style.margin.vertical() };

                base_sizes.push(basis);
                total_base += basis + margin_main;

                if child_style.flex_grow > 0.0 {
                    total_grow += child_style.flex_grow;
                }
                if child_style.flex_shrink > 0.0 {
                    total_shrink += child_style.flex_shrink * basis;
                }
            }

            let line_gap = if line_indices.len() > 1 { style.gap * (line_indices.len() - 1) as f32 } else { 0.0 };
            total_base += line_gap;

            let free_space = line_main_size - total_base;
            let final_sizes = Self::distribute_flex_by_indices(
                children,
                line_indices,
                &base_sizes,
                free_space,
                total_grow,
                total_shrink,
                is_row,
            );

            let line_inner_x = if is_row { inner_x } else { inner_x + cross_offset };
            let line_inner_y = if is_row { inner_y + cross_offset } else { inner_y };

            let line_main = if is_row { line_main_size } else { line_cross_size };
            let line_cross = if is_row { line_cross_size } else { line_main_size };

            Self::position_children_by_indices(
                children,
                line_indices,
                &final_sizes,
                style,
                line_inner_x,
                line_inner_y,
                if is_row { line_main_size } else { line_cross_size },
                if is_row { line_cross_size } else { line_main_size },
                line_main,
                line_cross,
                is_row,
            );

            cross_offset += line_cross_size + style.gap;
        }
    }

    /// 分配 flex-grow / flex-shrink 空间（单行模式）
    fn distribute_flex(
        children: &[LayoutNode],
        base_sizes: &[f32],
        free_space: f32,
        total_grow: f32,
        total_shrink: f32,
        is_row: bool,
    ) -> Vec<f32> {
        let mut result = Vec::with_capacity(children.len());

        for (i, child) in children.iter().enumerate() {
            let base = base_sizes[i];
            let mut size = base;

            if free_space > 0.0 && total_grow > 0.0 && child.style.flex_grow > 0.0 {
                size += (free_space * child.style.flex_grow / total_grow).max(0.0);
            }
            else if free_space < 0.0 && total_shrink > 0.0 && child.style.flex_shrink > 0.0 {
                let shrink_amount = (free_space.abs() * child.style.flex_shrink * base / total_shrink).max(0.0);
                size = (size - shrink_amount).max(0.0);
            }

            size = if is_row { child.style.clamp_width(size) } else { child.style.clamp_height(size) };

            result.push(size);
        }

        result
    }

    /// 分配 flex-grow / flex-shrink 空间（按索引，用于换行模式）
    fn distribute_flex_by_indices(
        children: &[LayoutNode],
        indices: &[usize],
        base_sizes: &[f32],
        free_space: f32,
        total_grow: f32,
        total_shrink: f32,
        is_row: bool,
    ) -> Vec<f32> {
        let mut result = Vec::with_capacity(indices.len());

        for (j, &idx) in indices.iter().enumerate() {
            let child = &children[idx];
            let base = base_sizes[j];
            let mut size = base;

            if free_space > 0.0 && total_grow > 0.0 && child.style.flex_grow > 0.0 {
                size += (free_space * child.style.flex_grow / total_grow).max(0.0);
            }
            else if free_space < 0.0 && total_shrink > 0.0 && child.style.flex_shrink > 0.0 {
                let shrink_amount = (free_space.abs() * child.style.flex_shrink * base / total_shrink).max(0.0);
                size = (size - shrink_amount).max(0.0);
            }

            size = if is_row { child.style.clamp_width(size) } else { child.style.clamp_height(size) };

            result.push(size);
        }

        result
    }

    /// 定位子节点（单行模式）
    fn position_children(
        children: &mut [LayoutNode],
        main_sizes: &[f32],
        style: &LayoutStyle,
        inner_x: f32,
        inner_y: f32,
        container_width: f32,
        container_height: f32,
        main_size: f32,
        cross_size: f32,
        is_row: bool,
    ) {
        let total_main: f32 = main_sizes.iter().copied().sum::<f32>()
            + children
                .iter()
                .map(|c| if is_row { c.style.margin.horizontal() } else { c.style.margin.vertical() })
                .sum::<f32>()
            + if children.len() > 1 { style.gap * (children.len() - 1) as f32 } else { 0.0 };

        let free_main = main_size - total_main;
        let (mut main_offset, item_gap) =
            Self::compute_justify_offset(style.justify_content, free_main, children.len(), style.gap);

        main_offset = if is_row { inner_x + main_offset } else { inner_y + main_offset };

        for (i, child) in children.iter_mut().enumerate() {
            let child_main = main_sizes[i];
            let child_margin_main_start = if is_row { child.style.margin.left } else { child.style.margin.top };
            let child_margin_main_end = if is_row { child.style.margin.right } else { child.style.margin.bottom };
            let child_margin_cross_start = if is_row { child.style.margin.top } else { child.style.margin.left };
            let child_margin_cross_end = if is_row { child.style.margin.bottom } else { child.style.margin.right };

            let child_cross = Self::compute_cross_size(child, cross_size, style.align_items, is_row);

            let cross_pos = Self::compute_align_offset(
                style.align_items,
                cross_size,
                child_cross,
                child_margin_cross_start,
                child_margin_cross_end,
            );

            let child_x = if is_row { main_offset + child_margin_main_start } else { inner_x + cross_pos };
            let child_y = if is_row { inner_y + cross_pos } else { main_offset + child_margin_main_start };

            child.computed.x = child_x;
            child.computed.y = child_y;
            child.computed.width = if is_row { child_main } else { child_cross };
            child.computed.height = if is_row { child_cross } else { child_main };

            let child_container_w = if is_row { child_main } else { container_width };
            let child_container_h = if is_row { container_height } else { child_main };

            Self::compute_node_layout(child, child_x, child_y, Some(child_container_w), Some(child_container_h));

            main_offset += child_main + child_margin_main_start + child_margin_main_end + item_gap;
        }
    }

    /// 定位子节点（按索引，用于换行模式）
    fn position_children_by_indices(
        children: &mut [LayoutNode],
        indices: &[usize],
        main_sizes: &[f32],
        style: &LayoutStyle,
        inner_x: f32,
        inner_y: f32,
        container_width: f32,
        container_height: f32,
        main_size: f32,
        cross_size: f32,
        is_row: bool,
    ) {
        let total_main: f32 = main_sizes.iter().copied().sum::<f32>()
            + indices
                .iter()
                .map(|&idx| {
                    let c = &children[idx];
                    if is_row { c.style.margin.horizontal() } else { c.style.margin.vertical() }
                })
                .sum::<f32>()
            + if indices.len() > 1 { style.gap * (indices.len() - 1) as f32 } else { 0.0 };

        let free_main = main_size - total_main;
        let (mut main_offset, item_gap) =
            Self::compute_justify_offset(style.justify_content, free_main, indices.len(), style.gap);

        main_offset = if is_row { inner_x + main_offset } else { inner_y + main_offset };

        for (j, &idx) in indices.iter().enumerate() {
            let child_main = main_sizes[j];
            let child = &mut children[idx];

            let child_margin_main_start = if is_row { child.style.margin.left } else { child.style.margin.top };
            let child_margin_main_end = if is_row { child.style.margin.right } else { child.style.margin.bottom };
            let child_margin_cross_start = if is_row { child.style.margin.top } else { child.style.margin.left };
            let child_margin_cross_end = if is_row { child.style.margin.bottom } else { child.style.margin.right };

            let child_cross = Self::compute_cross_size(child, cross_size, style.align_items, is_row);

            let cross_pos = Self::compute_align_offset(
                style.align_items,
                cross_size,
                child_cross,
                child_margin_cross_start,
                child_margin_cross_end,
            );

            let child_x = if is_row { main_offset + child_margin_main_start } else { inner_x + cross_pos };
            let child_y = if is_row { inner_y + cross_pos } else { main_offset + child_margin_main_start };

            child.computed.x = child_x;
            child.computed.y = child_y;
            child.computed.width = if is_row { child_main } else { child_cross };
            child.computed.height = if is_row { child_cross } else { child_main };

            let child_container_w = if is_row { child_main } else { container_width };
            let child_container_h = if is_row { container_height } else { child_main };

            Self::compute_node_layout(child, child_x, child_y, Some(child_container_w), Some(child_container_h));

            main_offset += child_main + child_margin_main_start + child_margin_main_end + item_gap;
        }
    }

    /// 计算 justify-content 的起始偏移和项目间距
    fn compute_justify_offset(justify: JustifyContent, free_space: f32, count: usize, gap: f32) -> (f32, f32) {
        if count == 0 {
            return (0.0, 0.0);
        }
        match justify {
            JustifyContent::FlexStart => (0.0, gap),
            JustifyContent::FlexEnd => (free_space.max(0.0), gap),
            JustifyContent::Center => (free_space.max(0.0) / 2.0, gap),
            JustifyContent::SpaceBetween => {
                if count <= 1 {
                    (0.0, 0.0)
                }
                else {
                    let spacing = free_space.max(0.0) / (count - 1) as f32;
                    (0.0, spacing)
                }
            }
            JustifyContent::SpaceAround => {
                if count == 0 {
                    (0.0, 0.0)
                }
                else {
                    let spacing = free_space.max(0.0) / count as f32;
                    (spacing / 2.0, spacing)
                }
            }
            JustifyContent::SpaceEvenly => {
                let spacing = free_space.max(0.0) / (count + 1) as f32;
                (spacing, spacing)
            }
        }
    }

    /// 计算子节点在交叉轴上的尺寸
    fn compute_cross_size(child: &LayoutNode, cross_available: f32, align: AlignItems, is_row: bool) -> f32 {
        let explicit = if is_row {
            if child.style.is_height_auto() { None } else { Some(child.style.height) }
        }
        else {
            if child.style.is_width_auto() { None } else { Some(child.style.width) }
        };

        match align {
            AlignItems::Stretch if explicit.is_none() => {
                let margin_cross = if is_row { child.style.margin.vertical() } else { child.style.margin.horizontal() };
                let stretched = (cross_available - margin_cross).max(0.0);
                if is_row { child.style.clamp_height(stretched) } else { child.style.clamp_width(stretched) }
            }
            _ => explicit.unwrap_or(0.0),
        }
    }

    /// 计算子节点在交叉轴上的偏移
    fn compute_align_offset(
        align: AlignItems,
        cross_size: f32,
        child_cross: f32,
        margin_cross_start: f32,
        margin_cross_end: f32,
    ) -> f32 {
        let available = cross_size - child_cross - margin_cross_start - margin_cross_end;
        match align {
            AlignItems::FlexStart | AlignItems::Stretch | AlignItems::Baseline => margin_cross_start,
            AlignItems::FlexEnd => available.max(0.0) + margin_cross_start,
            AlignItems::Center => available.max(0.0) / 2.0 + margin_cross_start,
        }
    }
}

impl Default for FlexLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
