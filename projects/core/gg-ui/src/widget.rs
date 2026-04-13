use std::{
    any::Any,
    sync::{Arc, RwLock},
};

use crate::{
    gui_event::{EventContext, GuiEvent},
    node::{UiNodeId, UiTree},
};
use gg_core::GResult;

/// 单个 UsageHint 标志，指示哪种属性变化由 GPU 处理
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageHint {
    /// 位移变化通过 uniform 传入着色器，GPU 执行变换
    TransformOffset,
    /// 颜色变化通过 uniform 传入着色器，GPU 执行颜色混合
    ColorTint,
    /// 透明度变化通过 uniform 传入着色器
    Opacity,
    /// 缩放变化通过 uniform 传入着色器
    ScaleTransform,
}

impl UsageHint {
    /// 获取对应位标志值
    fn bit(self) -> u8 {
        match self {
            UsageHint::TransformOffset => 1,
            UsageHint::ColorTint => 2,
            UsageHint::Opacity => 4,
            UsageHint::ScaleTransform => 8,
        }
    }
}

/// UsageHint 集合，基于 u8 位标志实现
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageHints {
    /// 内部位标志
    flags: u8,
}

impl UsageHints {
    /// 无任何 hint
    pub const NONE: UsageHints = UsageHints { flags: 0 };
    /// TransformOffset hint 标志
    pub const TRANSFORM_OFFSET: UsageHints = UsageHints { flags: 1 };
    /// ColorTint hint 标志
    pub const COLOR_TINT: UsageHints = UsageHints { flags: 2 };
    /// Opacity hint 标志
    pub const OPACITY: UsageHints = UsageHints { flags: 4 };
    /// ScaleTransform hint 标志
    pub const SCALE_TRANSFORM: UsageHints = UsageHints { flags: 8 };

    /// 判断是否包含指定 hint
    pub fn contains(&self, hint: UsageHint) -> bool {
        self.flags & hint.bit() != 0
    }

    /// 插入指定 hint
    pub fn insert(&mut self, hint: UsageHint) {
        self.flags |= hint.bit();
    }

    /// 移除指定 hint
    pub fn remove(&mut self, hint: UsageHint) {
        self.flags &= !hint.bit();
    }

    /// 判断是否没有任何 hint
    pub fn is_empty(&self) -> bool {
        self.flags == 0
    }
}

impl Default for UsageHints {
    fn default() -> Self {
        UsageHints::NONE
    }
}

/// 组件生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetLifecycle {
    /// 已创建但未挂载
    Created,
    /// 已挂载到 DOM 树
    Mounted,
    /// 已更新
    Updated,
    /// 已卸载
    Unmounted,
}

/// 脏标记位标志，用于标识组件需要更新的类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyFlag(u32);

impl DirtyFlag {
    /// 无脏标记
    pub const NONE: DirtyFlag = DirtyFlag(0);
    /// 布局脏标记，表示需要重新计算布局
    pub const LAYOUT: DirtyFlag = DirtyFlag(1 << 0);
    /// 样式脏标记，表示需要重新应用样式
    pub const STYLE: DirtyFlag = DirtyFlag(1 << 1);
    /// 内容脏标记，表示需要重新渲染内容
    pub const CONTENT: DirtyFlag = DirtyFlag(1 << 2);
    /// 变换脏标记，表示需要重新计算变换矩阵
    pub const TRANSFORM: DirtyFlag = DirtyFlag(1 << 3);
    /// 全部脏标记，表示所有类型都需要更新
    pub const ALL: DirtyFlag = DirtyFlag(Self::LAYOUT.0 | Self::STYLE.0 | Self::CONTENT.0 | Self::TRANSFORM.0);

    /// 判断是否包含指定脏标记
    pub fn contains(self, other: DirtyFlag) -> bool {
        self.0 & other.0 != 0
    }

    /// 判断是否没有任何脏标记
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// 获取内部位值
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl Default for DirtyFlag {
    fn default() -> Self {
        DirtyFlag::NONE
    }
}

impl std::ops::BitOr for DirtyFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        DirtyFlag(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for DirtyFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for DirtyFlag {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        DirtyFlag(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for DirtyFlag {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::Not for DirtyFlag {
    type Output = Self;

    fn not(self) -> Self::Output {
        DirtyFlag(!self.0 & DirtyFlag::ALL.0)
    }
}

/// 控件 trait
///
/// 所有 UI 控件必须实现此 trait，对齐 *.widget 文件格式的三段式结构
/// （template/script/style），定义控件的渲染、脚本、样式和事件处理接口。
pub trait Widget: Any + Send + Sync {
    /// 渲染模板，返回 UI 节点描述
    fn render_template(&self) -> oak_voc::TemplateNode;

    /// 脚本初始化，设置响应式状态和事件处理
    fn script_setup(&mut self);

    /// 获取样式定义
    fn get_style(&self) -> Option<&str> {
        None
    }

    /// 获取组件 ID
    fn get_id(&self) -> &str;

    /// 处理 GUI 事件
    fn handle_event(&mut self, event: &GuiEvent, ctx: &mut EventContext);

    /// 获取子组件列表
    fn children(&self) -> Vec<Arc<RwLock<dyn Widget>>> {
        Vec::new()
    }

    /// 组件挂载时调用
    fn on_mount(&mut self) {}

    /// 组件更新时调用
    fn on_update(&mut self) {}

    /// 组件卸载时调用
    fn on_cleanup(&mut self) {}

    /// 判断组件是否有脏标记
    fn is_dirty(&self) -> bool {
        false
    }

    /// 清除所有脏标记
    fn clear_dirty(&mut self) {}

    /// 获取当前脏标记
    fn get_dirty_flags(&self) -> DirtyFlag {
        DirtyFlag::NONE
    }

    /// 标记指定脏标记
    fn mark_dirty(&mut self, _flag: DirtyFlag) {}

    /// 获取组件的使用提示
    fn usage_hints(&self) -> UsageHints {
        UsageHints::default()
    }

    /// 构建控件节点树，存储根节点 ID 并返回
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId>;

    /// 更新控件状态
    fn update(&self, tree: &mut UiTree);

    /// 获取控件根节点 ID
    fn node_id(&self) -> Option<UiNodeId>;
}

/// 动态控件，由 VxDocument 转换而来
pub struct DynamicWidget {
    /// 组件 ID
    id: String,
    /// 模板节点
    template: Option<oak_voc::TemplateNode>,
    /// 格式化的样式字符串
    style_string: Option<String>,
    /// 脚本源码
    script_source: Option<String>,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 脏标记位标志
    dirty_flags: DirtyFlag,
    /// 使用提示
    hints: UsageHints,
    /// 节点 ID
    node_id: Option<UiNodeId>,
}

impl DynamicWidget {
    /// 创建新的动态控件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            template: None,
            style_string: None,
            script_source: None,
            lifecycle: WidgetLifecycle::Created,
            dirty_flags: DirtyFlag::NONE,
            hints: UsageHints::NONE,
            node_id: None,
        }
    }

    /// 从文档数据创建动态控件
    pub fn from_document(
        id: &str,
        template: Option<oak_voc::TemplateNode>,
        style_string: Option<String>,
        script_source: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            template,
            style_string,
            script_source,
            lifecycle: WidgetLifecycle::Created,
            dirty_flags: DirtyFlag::NONE,
            hints: UsageHints::NONE,
            node_id: None,
        }
    }

    /// 获取脚本源码引用
    pub fn script_source(&self) -> Option<&str> {
        self.script_source.as_deref()
    }

    /// 获取生命周期状态
    pub fn lifecycle(&self) -> WidgetLifecycle {
        self.lifecycle
    }
}

impl Widget for DynamicWidget {
    fn render_template(&self) -> oak_voc::TemplateNode {
        match &self.template {
            Some(node) => node.clone(),
            None => oak_voc::TemplateNode::text(String::new()),
        }
    }

    fn script_setup(&mut self) {
        let _ = &self.script_source;
    }

    fn get_style(&self) -> Option<&str> {
        self.style_string.as_ref().and_then(|s| if s.is_empty() { None } else { Some(s.as_str()) })
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, _event: &GuiEvent, _ctx: &mut EventContext) {}

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn is_dirty(&self) -> bool {
        !self.dirty_flags.is_empty()
    }

    fn clear_dirty(&mut self) {
        self.dirty_flags = DirtyFlag::NONE;
    }

    fn get_dirty_flags(&self) -> DirtyFlag {
        self.dirty_flags
    }

    fn mark_dirty(&mut self, flag: DirtyFlag) {
        self.dirty_flags |= flag;
    }

    fn usage_hints(&self) -> UsageHints {
        self.hints
    }

    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut UiTree) {}

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
