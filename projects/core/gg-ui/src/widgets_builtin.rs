use std::sync::{Arc, RwLock};

use crate::{
    gui_event::{EventContext, GuiEvent},
    style::FlexDirection,
    widget::{Widget, WidgetLifecycle},
};

/// 布局组件，支持 flex 布局
pub struct Layout {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn Widget>>>,
    /// Flex 方向
    direction: FlexDirection,
    /// 对齐方式
    align: String,
    /// 主轴对齐
    justify: String,
    /// 间距
    gap: f32,
    /// 内边距
    padding: f32,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Layout {
    /// 创建新的布局组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            children: Vec::new(),
            direction: FlexDirection::Column,
            align: String::new(),
            justify: String::new(),
            gap: 0.0,
            padding: 0.0,
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 设置 Flex 方向
    pub fn set_direction(&mut self, direction: FlexDirection) {
        self.direction = direction;
    }

    /// 设置对齐方式
    pub fn set_align(&mut self, align: &str) {
        self.align = align.to_string();
    }

    /// 设置主轴对齐
    pub fn set_justify(&mut self, justify: &str) {
        self.justify = justify.to_string();
    }

    /// 设置间距
    pub fn set_gap(&mut self, gap: f32) {
        self.gap = gap;
    }

    /// 设置内边距
    pub fn set_padding(&mut self, padding: f32) {
        self.padding = padding;
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn Widget>>) {
        self.children.push(child);
    }
}

impl Widget for Layout {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![
            ("id".to_string(), self.id.clone()),
            (
                "direction".to_string(),
                match self.direction {
                    FlexDirection::Row => "row".to_string(),
                    FlexDirection::Column => "column".to_string(),
                },
            ),
        ];
        if !self.align.is_empty() {
            attrs.push(("align".to_string(), self.align.clone()));
        }
        if !self.justify.is_empty() {
            attrs.push(("justify".to_string(), self.justify.clone()));
        }
        if self.gap > 0.0 {
            attrs.push(("gap".to_string(), self.gap.to_string()));
        }
        if self.padding > 0.0 {
            attrs.push(("padding".to_string(), self.padding.to_string()));
        }
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let children = self
            .children
            .iter()
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { oak_voc::TemplateNode::text(String::new()) })
            .collect();
        oak_voc::TemplateNode::element("Layout", attrs, children)
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, event: &GuiEvent, ctx: &mut EventContext) {
        if ctx.stop_propagation {
            return;
        }
        for child in &self.children {
            if ctx.stop_propagation {
                return;
            }
            if let Ok(mut comp) = child.write() {
                comp.handle_event(event, ctx);
            }
        }
        let _ = event;
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Widget>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 栈式布局组件，支持垂直/水平排列
pub struct Stack {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn Widget>>>,
    /// 方向：vertical 或 horizontal
    orientation: FlexDirection,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Stack {
    /// 创建新的栈式布局组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            children: Vec::new(),
            orientation: FlexDirection::Column,
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 设置方向
    pub fn set_orientation(&mut self, orientation: FlexDirection) {
        self.orientation = orientation;
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn Widget>>) {
        self.children.push(child);
    }
}

impl Widget for Stack {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![
            ("id".to_string(), self.id.clone()),
            (
                "orientation".to_string(),
                match self.orientation {
                    FlexDirection::Row => "horizontal".to_string(),
                    FlexDirection::Column => "vertical".to_string(),
                },
            ),
        ];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let children = self
            .children
            .iter()
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { oak_voc::TemplateNode::text(String::new()) })
            .collect();
        oak_voc::TemplateNode::element("Stack", attrs, children)
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, event: &GuiEvent, ctx: &mut EventContext) {
        if ctx.stop_propagation {
            return;
        }
        for child in &self.children {
            if ctx.stop_propagation {
                return;
            }
            if let Ok(mut comp) = child.write() {
                comp.handle_event(event, ctx);
            }
        }
        let _ = event;
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Widget>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 按钮组件
pub struct Button {
    /// 组件 ID
    id: String,
    /// 按钮文本
    text: String,
    /// 点击回调
    onclick: Option<Box<dyn Fn() + Send + Sync>>,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Button {
    /// 创建新的按钮组件
    pub fn new(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            onclick: None,
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 设置点击回调
    pub fn set_onclick<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.onclick = Some(Box::new(callback));
    }
}

impl Widget for Button {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        oak_voc::TemplateNode::element("Button", attrs, vec![oak_voc::TemplateNode::text(self.text.clone())])
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, event: &GuiEvent, _ctx: &mut EventContext) {
        if let GuiEvent::MouseClick { .. } = event {
            if let Some(onclick) = &self.onclick {
                onclick();
            }
        }
    }

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 文本组件
pub struct Text {
    /// 组件 ID
    id: String,
    /// 文本内容
    value: String,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Text {
    /// 创建新的文本组件
    pub fn new(id: &str, value: &str) -> Self {
        Self {
            id: id.to_string(),
            value: value.to_string(),
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }
}

impl Widget for Text {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        oak_voc::TemplateNode::element("Text", attrs, vec![oak_voc::TemplateNode::text(self.value.clone())])
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
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

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 输入框组件
pub struct Input {
    /// 组件 ID
    id: String,
    /// 当前值
    value: String,
    /// 值变更回调
    onchange: Option<Box<dyn Fn(&str) + Send + Sync>>,
    /// 占位文本
    placeholder: String,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Input {
    /// 创建新的输入框组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            value: String::new(),
            onchange: None,
            placeholder: String::new(),
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 设置值变更回调
    pub fn set_onchange<F: Fn(&str) + Send + Sync + 'static>(&mut self, callback: F) {
        self.onchange = Some(Box::new(callback));
    }

    /// 设置占位文本
    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder = placeholder.to_string();
    }
}

impl Widget for Input {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.placeholder.is_empty() {
            attrs.push(("placeholder".to_string(), self.placeholder.clone()));
        }
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let display_text = if self.value.is_empty() { self.placeholder.clone() } else { format!("{}|", self.value) };
        oak_voc::TemplateNode::element("Input", attrs, vec![oak_voc::TemplateNode::text(display_text)])
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, event: &GuiEvent, _ctx: &mut EventContext) {
        if let GuiEvent::TextInput { text } = event {
            self.value = text.clone();
            if let Some(onchange) = &self.onchange {
                onchange(&self.value);
            }
        }
    }

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 图片组件
pub struct Image {
    /// 组件 ID
    id: String,
    /// 图片源路径
    src: String,
    /// 宽度
    width: Option<f32>,
    /// 高度
    height: Option<f32>,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Image {
    /// 创建新的图片组件
    pub fn new(id: &str, src: &str) -> Self {
        Self {
            id: id.to_string(),
            src: src.to_string(),
            width: None,
            height: None,
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 设置宽度
    pub fn set_width(&mut self, width: f32) {
        self.width = Some(width);
    }

    /// 设置高度
    pub fn set_height(&mut self, height: f32) {
        self.height = Some(height);
    }
}

impl Widget for Image {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone()), ("src".to_string(), self.src.clone())];
        if let Some(width) = self.width {
            attrs.push(("width".to_string(), width.to_string()));
        }
        if let Some(height) = self.height {
            attrs.push(("height".to_string(), height.to_string()));
        }
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        oak_voc::TemplateNode::element("Image", attrs, vec![])
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
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

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 面板容器组件
pub struct Panel {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn Widget>>>,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl Panel {
    /// 创建新的面板组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            children: Vec::new(),
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn Widget>>) {
        self.children.push(child);
    }
}

impl Widget for Panel {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let children = self
            .children
            .iter()
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { oak_voc::TemplateNode::text(String::new()) })
            .collect();
        oak_voc::TemplateNode::element("Panel", attrs, children)
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, event: &GuiEvent, ctx: &mut EventContext) {
        if ctx.stop_propagation {
            return;
        }
        for child in &self.children {
            if ctx.stop_propagation {
                return;
            }
            if let Ok(mut comp) = child.write() {
                comp.handle_event(event, ctx);
            }
        }
        let _ = event;
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Widget>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}

/// 滚动视图组件
pub struct ScrollView {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn Widget>>>,
    /// 滚动方向
    direction: FlexDirection,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: WidgetLifecycle,
    /// 节点 ID
    node_id: Option<crate::node::UiNodeId>,
}

impl ScrollView {
    /// 创建新的滚动视图组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            children: Vec::new(),
            direction: FlexDirection::Column,
            style: String::new(),
            lifecycle: WidgetLifecycle::Created,
            node_id: None,
        }
    }

    /// 设置滚动方向
    pub fn set_direction(&mut self, direction: FlexDirection) {
        self.direction = direction;
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn Widget>>) {
        self.children.push(child);
    }
}

impl Widget for ScrollView {
    fn render_template(&self) -> oak_voc::TemplateNode {
        let mut attrs = vec![
            ("id".to_string(), self.id.clone()),
            (
                "direction".to_string(),
                match self.direction {
                    FlexDirection::Row => "horizontal".to_string(),
                    FlexDirection::Column => "vertical".to_string(),
                },
            ),
        ];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let children = self
            .children
            .iter()
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { oak_voc::TemplateNode::text(String::new()) })
            .collect();
        oak_voc::TemplateNode::element("ScrollView", attrs, children)
    }

    fn script_setup(&mut self) {}

    fn get_style(&self) -> Option<&str> {
        if self.style.is_empty() { None } else { Some(&self.style) }
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn handle_event(&mut self, event: &GuiEvent, ctx: &mut EventContext) {
        if ctx.stop_propagation {
            return;
        }
        for child in &self.children {
            if ctx.stop_propagation {
                return;
            }
            if let Ok(mut comp) = child.write() {
                comp.handle_event(event, ctx);
            }
        }
        let _ = event;
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Widget>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = WidgetLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = WidgetLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = WidgetLifecycle::Unmounted;
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        let template = self.render_template();
        let converted = crate::template::template_node_to_ui_tree(&template);
        let root_id = converted.root().ok_or_else(|| gg_error::GError::new("Failed to get root node"))?;
        tree.merge_from(&converted);
        self.node_id = Some(root_id);
        Ok(root_id)
    }

    fn update(&self, _tree: &mut crate::node::UiTree) {}

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        self.node_id
    }
}
