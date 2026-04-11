//! Flexbox 布局引擎模块
//!
//! 自行实现核心 Flexbox 算法，不依赖外部 yoga crate。
//! 支持主轴/交叉轴布局、flex-grow/shrink、justify-content、align-items、
//! flex-wrap、gap 间距、padding/margin 等特性。
//! 布局计算仅在脏标记触发时执行，静止时不会重算。

use std::collections::{HashMap, HashSet};

use crate::FlexDirection;

/// 表示四个方向的边距值
#[derive(Debug, Clone, Copy)]
pub struct LayoutEdge {
    /// 左侧边距
    pub left: f32,
    /// 右侧边距
    pub right: f32,
    /// 顶部边距
    pub top: f32,
    /// 底部边距
    pub bottom: f32,
}

impl LayoutEdge {
    /// 创建零边距
    pub fn zero() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }

    /// 水平方向边距之和
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// 垂直方向边距之和
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for LayoutEdge {
    fn default() -> Self {
        Self::zero()
    }
}

/// 主轴对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    /// 从主轴起始位置对齐
    FlexStart,
    /// 从主轴末尾位置对齐
    FlexEnd,
    /// 居中对齐
    Center,
    /// 两端对齐，项目之间的间隔相等
    SpaceBetween,
    /// 每个项目两侧的间隔相等
    SpaceAround,
    /// 每个项目之间及两端间隔完全相等
    SpaceEvenly,
}

/// 交叉轴对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    /// 从交叉轴起始位置对齐
    FlexStart,
    /// 从交叉轴末尾位置对齐
    FlexEnd,
    /// 居中对齐
    Center,
    /// 拉伸填满交叉轴
    Stretch,
    /// 基线对齐
    Baseline,
}

/// 换行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    /// 不换行
    NoWrap,
    /// 正常换行
    Wrap,
    /// 反向换行
    WrapReverse,
}

/// 描述节点的布局样式
#[derive(Debug, Clone)]
pub struct LayoutStyle {
    /// 主轴方向
    pub flex_direction: FlexDirection,
    /// 主轴对齐方式
    pub justify_content: JustifyContent,
    /// 交叉轴对齐方式
    pub align_items: AlignItems,
    /// 换行模式
    pub flex_wrap: FlexWrap,
    /// 弹性增长因子
    pub flex_grow: f32,
    /// 弹性收缩因子
    pub flex_shrink: f32,
    /// 弹性基准值
    pub flex_basis: f32,
    /// 子元素间距
    pub gap: f32,
    /// 内边距
    pub padding: LayoutEdge,
    /// 外边距
    pub margin: LayoutEdge,
    /// 宽度，-1 表示 auto
    pub width: f32,
    /// 高度，-1 表示 auto
    pub height: f32,
    /// 最小宽度
    pub min_width: f32,
    /// 最小高度
    pub min_height: f32,
    /// 最大宽度
    pub max_width: f32,
    /// 最大高度
    pub max_height: f32,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            gap: 0.0,
            padding: LayoutEdge::zero(),
            margin: LayoutEdge::zero(),
            width: -1.0,
            height: -1.0,
            min_width: 0.0,
            min_height: 0.0,
            max_width: f32::MAX,
            max_height: f32::MAX,
        }
    }
}

impl LayoutStyle {
    /// 创建默认布局样式
    pub fn new() -> Self {
        Self::default()
    }

    /// 判断宽度是否为 auto
    pub fn is_width_auto(&self) -> bool {
        self.width < 0.0
    }

    /// 判断高度是否为 auto
    pub fn is_height_auto(&self) -> bool {
        self.height < 0.0
    }

    /// 将宽度值钳制在 min_width 和 max_width 之间
    pub fn clamp_width(&self, w: f32) -> f32 {
        w.max(self.min_width).min(self.max_width)
    }

    /// 将高度值钳制在 min_height 和 max_height 之间
    pub fn clamp_height(&self, h: f32) -> f32 {
        h.max(self.min_height).min(self.max_height)
    }
}

/// 布局计算结果
#[derive(Debug, Clone, Copy)]
pub struct LayoutResult {
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
    /// 宽度
    pub width: f32,
    /// 高度
    pub height: f32,
}

impl Default for LayoutResult {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

/// 布局节点
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// 节点唯一标识
    pub id: String,
    /// 布局样式
    pub style: LayoutStyle,
    /// 子节点列表
    pub children: Vec<LayoutNode>,
    /// 布局计算结果
    pub computed: LayoutResult,
    /// 脏标记，为 true 时表示需要重新计算布局
    pub is_dirty: bool,
}

impl LayoutNode {
    /// 创建新的布局节点
    pub fn new(id: &str, style: LayoutStyle) -> Self {
        Self {
            id: id.to_string(),
            style,
            children: Vec::new(),
            computed: LayoutResult::default(),
            is_dirty: true,
        }
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
        self.is_dirty = true;
    }

    /// 递归查找指定 ID 的节点
    pub fn find_node(&self, node_id: &str) -> Option<&LayoutNode> {
        if self.id == node_id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_node(node_id) {
                return Some(found);
            }
        }
        None
    }

    /// 递归查找指定 ID 的节点的可变引用
    pub fn find_node_mut(&mut self, node_id: &str) -> Option<&mut LayoutNode> {
        if self.id == node_id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_node_mut(node_id) {
                return Some(found);
            }
        }
        None
    }

    /// 收集所有脏节点的 ID
    fn collect_dirty_ids(&self, ids: &mut HashSet<String>) {
        if self.is_dirty {
            ids.insert(self.id.clone());
        }
        for child in &self.children {
            child.collect_dirty_ids(ids);
        }
    }

    /// 递归清除所有脏标记
    fn clear_dirty_recursive(&mut self) {
        self.is_dirty = false;
        for child in &mut self.children {
            child.clear_dirty_recursive();
        }
    }

    /// 递归标记自身及所有后代为脏
    fn mark_subtree_dirty(&mut self) {
        self.is_dirty = true;
        for child in &mut self.children {
            child.mark_subtree_dirty();
        }
    }
}

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
        Self {
            root: None,
            dirty_nodes: HashSet::new(),
        }
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

    /// 获取指定节点的布局结果
    pub fn get_layout(&self, node_id: &str) -> Option<&LayoutResult> {
        self.root
            .as_ref()
            .and_then(|root| root.find_node(node_id))
            .map(|node| &node.computed)
    }

    /// 检查是否有脏节点
    pub fn has_dirty_nodes(&self) -> bool {
        if !self.dirty_nodes.is_empty() {
            return true;
        }
        self.root
            .as_ref()
            .map_or(false, |root| Self::has_dirty_recursive(root))
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

    /// 计算单个节点的布局
    fn compute_node_layout(
        node: &mut LayoutNode,
        parent_x: f32,
        parent_y: f32,
        container_width: Option<f32>,
        container_height: Option<f32>,
    ) {
        let style = node.style.clone();

        let node_width = if style.is_width_auto() {
            container_width.unwrap_or(0.0) - style.margin.horizontal()
        } else {
            style.width
        };
        let node_width = style.clamp_width(node_width.max(0.0));

        let node_height = if style.is_height_auto() {
            container_height.unwrap_or(0.0) - style.margin.vertical()
        } else {
            style.height
        };
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
                Self::layout_single_line(
                    &mut node.children,
                    &style,
                    inner_x,
                    inner_y,
                    inner_width,
                    inner_height,
                );
            }
            FlexWrap::Wrap | FlexWrap::WrapReverse => {
                Self::layout_wrap(
                    &mut node.children,
                    &style,
                    inner_x,
                    inner_y,
                    inner_width,
                    inner_height,
                );
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

        let total_gap = if children.len() > 1 {
            style.gap * (children.len() - 1) as f32
        } else {
            0.0
        };

        let mut base_main_sizes: Vec<f32> = Vec::with_capacity(children.len());
        let mut total_base: f32 = 0.0;
        let mut total_grow: f32 = 0.0;
        let mut total_shrink: f32 = 0.0;

        for child in children.iter() {
            let child_style = &child.style;
            let basis = if child_style.flex_basis > 0.0 {
                child_style.flex_basis
            } else if is_row {
                if child_style.is_width_auto() { 0.0 } else { child_style.width }
            } else {
                if child_style.is_height_auto() { 0.0 } else { child_style.height }
            };

            let margin_main = if is_row {
                child_style.margin.horizontal()
            } else {
                child_style.margin.vertical()
            };

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
        let final_main_sizes = Self::distribute_flex(
            children,
            &base_main_sizes,
            free_space,
            total_grow,
            total_shrink,
            is_row,
        );

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
            } else if is_row {
                if child_style.is_width_auto() { 0.0 } else { child_style.width }
            } else {
                if child_style.is_height_auto() { 0.0 } else { child_style.height }
            };

            let margin_main = if is_row {
                child_style.margin.horizontal()
            } else {
                child_style.margin.vertical()
            };

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
        let total_cross_gap = if num_lines > 1 {
            style.gap * (num_lines - 1) as f32
        } else {
            0.0
        };

        let line_cross_total = if cross_size > 0.0 {
            (cross_size - total_cross_gap) / num_lines as f32
        } else {
            0.0
        };

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
                } else if is_row {
                    if child_style.is_width_auto() { 0.0 } else { child_style.width }
                } else {
                    if child_style.is_height_auto() { 0.0 } else { child_style.height }
                };

                let margin_main = if is_row {
                    child_style.margin.horizontal()
                } else {
                    child_style.margin.vertical()
                };

                base_sizes.push(basis);
                total_base += basis + margin_main;

                if child_style.flex_grow > 0.0 {
                    total_grow += child_style.flex_grow;
                }
                if child_style.flex_shrink > 0.0 {
                    total_shrink += child_style.flex_shrink * basis;
                }
            }

            let line_gap = if line_indices.len() > 1 {
                style.gap * (line_indices.len() - 1) as f32
            } else {
                0.0
            };
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
            } else if free_space < 0.0 && total_shrink > 0.0 && child.style.flex_shrink > 0.0 {
                let shrink_amount = (free_space.abs() * child.style.flex_shrink * base / total_shrink).max(0.0);
                size = (size - shrink_amount).max(0.0);
            }

            size = if is_row {
                child.style.clamp_width(size)
            } else {
                child.style.clamp_height(size)
            };

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
            } else if free_space < 0.0 && total_shrink > 0.0 && child.style.flex_shrink > 0.0 {
                let shrink_amount = (free_space.abs() * child.style.flex_shrink * base / total_shrink).max(0.0);
                size = (size - shrink_amount).max(0.0);
            }

            size = if is_row {
                child.style.clamp_width(size)
            } else {
                child.style.clamp_height(size)
            };

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
            + children.iter().map(|c| if is_row { c.style.margin.horizontal() } else { c.style.margin.vertical() }).sum::<f32>()
            + if children.len() > 1 { style.gap * (children.len() - 1) as f32 } else { 0.0 };

        let free_main = main_size - total_main;
        let (mut main_offset, item_gap) = Self::compute_justify_offset(style.justify_content, free_main, children.len(), style.gap);

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
            + indices.iter().map(|&idx| {
                let c = &children[idx];
                if is_row { c.style.margin.horizontal() } else { c.style.margin.vertical() }
            }).sum::<f32>()
            + if indices.len() > 1 { style.gap * (indices.len() - 1) as f32 } else { 0.0 };

        let free_main = main_size - total_main;
        let (mut main_offset, item_gap) = Self::compute_justify_offset(style.justify_content, free_main, indices.len(), style.gap);

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
                } else {
                    let spacing = free_space.max(0.0) / (count - 1) as f32;
                    (0.0, spacing)
                }
            }
            JustifyContent::SpaceAround => {
                if count == 0 {
                    (0.0, 0.0)
                } else {
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
        } else {
            if child.style.is_width_auto() { None } else { Some(child.style.width) }
        };

        match align {
            AlignItems::Stretch if explicit.is_none() => {
                let margin_cross = if is_row {
                    child.style.margin.vertical()
                } else {
                    child.style.margin.horizontal()
                };
                let stretched = (cross_available - margin_cross).max(0.0);
                if is_row {
                    child.style.clamp_height(stretched)
                } else {
                    child.style.clamp_width(stretched)
                }
            }
            _ => {
                explicit.unwrap_or(0.0)
            }
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
            AlignItems::FlexStart | AlignItems::Stretch | AlignItems::Baseline => {
                margin_cross_start
            }
            AlignItems::FlexEnd => {
                available.max(0.0) + margin_cross_start
            }
            AlignItems::Center => {
                available.max(0.0) / 2.0 + margin_cross_start
            }
        }
    }
}

impl Default for FlexLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// USS 样式属性映射模块
pub mod uss_style_mapper {
    use super::*;

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
        if trimmed == "auto" {
            -1.0
        } else {
            parse_f32(trimmed).unwrap_or(-1.0)
        }
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
            4 => {
                LayoutEdge {
                    top: parse_f32(parts[0]).unwrap_or(0.0),
                    right: parse_f32(parts[1]).unwrap_or(0.0),
                    bottom: parse_f32(parts[2]).unwrap_or(0.0),
                    left: parse_f32(parts[3]).unwrap_or(0.0),
                }
            }
            _ => LayoutEdge::zero(),
        }
    }
}
