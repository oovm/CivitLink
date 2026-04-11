//! 布局类型定义模块
//!
//! 定义布局引擎所需的核心数据类型，包括边距、对齐方式、布局样式和布局节点。

use std::collections::HashSet;

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
        Self { left: 0.0, right: 0.0, top: 0.0, bottom: 0.0 }
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
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
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
        Self { id: id.to_string(), style, children: Vec::new(), computed: LayoutResult::default(), is_dirty: true }
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
    pub fn collect_dirty_ids(&self, ids: &mut HashSet<String>) {
        if self.is_dirty {
            ids.insert(self.id.clone());
        }
        for child in &self.children {
            child.collect_dirty_ids(ids);
        }
    }

    /// 递归清除所有脏标记
    pub fn clear_dirty_recursive(&mut self) {
        self.is_dirty = false;
        for child in &mut self.children {
            child.clear_dirty_recursive();
        }
    }

    /// 递归标记自身及所有后代为脏
    pub fn mark_subtree_dirty(&mut self) {
        self.is_dirty = true;
        for child in &mut self.children {
            child.mark_subtree_dirty();
        }
    }
}
