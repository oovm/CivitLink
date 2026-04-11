//! 基础UI组件实现
//! 
//! 提供编辑器UI系统的基础组件，如Layout、Button、Text等

use std::sync::{Arc, RwLock};
use crate::{VxComponent, FlexDirection, ComponentLifecycle, events::GuiEvent, events::EventContext};
use oak_voc::TemplateNode;

/// 布局组件，支持 flex 布局
pub struct Layout {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn VxComponent>>>,
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
    lifecycle: ComponentLifecycle,
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
            lifecycle: ComponentLifecycle::Created,
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
    pub fn add_child(&mut self, child: Arc<RwLock<dyn VxComponent>>) {
        self.children.push(child);
    }
}

impl VxComponent for Layout {
    fn render_template(&self) -> TemplateNode {
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
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { TemplateNode::Text(String::new()) })
            .collect();
        TemplateNode::Element { tag: "Layout".to_string(), attributes: attrs, children }
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

    fn children(&self) -> Vec<Arc<RwLock<dyn VxComponent>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
    }
}

/// 栈式布局组件，支持垂直/水平排列
pub struct Stack {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn VxComponent>>>,
    /// 方向：vertical 或 horizontal
    orientation: FlexDirection,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: ComponentLifecycle,
}

impl Stack {
    /// 创建新的栈式布局组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            children: Vec::new(),
            orientation: FlexDirection::Column,
            style: String::new(),
            lifecycle: ComponentLifecycle::Created,
        }
    }

    /// 设置方向
    pub fn set_orientation(&mut self, orientation: FlexDirection) {
        self.orientation = orientation;
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn VxComponent>>) {
        self.children.push(child);
    }
}

impl VxComponent for Stack {
    fn render_template(&self) -> TemplateNode {
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
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { TemplateNode::Text(String::new()) })
            .collect();
        TemplateNode::Element { tag: "Stack".to_string(), attributes: attrs, children }
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

    fn children(&self) -> Vec<Arc<RwLock<dyn VxComponent>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
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
    lifecycle: ComponentLifecycle,
}

impl Button {
    /// 创建新的按钮组件
    pub fn new(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            onclick: None,
            style: String::new(),
            lifecycle: ComponentLifecycle::Created,
        }
    }

    /// 设置点击回调
    pub fn set_onclick<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
        self.onclick = Some(Box::new(callback));
    }
}

impl VxComponent for Button {
    fn render_template(&self) -> TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        TemplateNode::Element {
            tag: "Button".to_string(),
            attributes: attrs,
            children: vec![TemplateNode::Text(self.text.clone())],
        }
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
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
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
    lifecycle: ComponentLifecycle,
}

impl Text {
    /// 创建新的文本组件
    pub fn new(id: &str, value: &str) -> Self {
        Self { id: id.to_string(), value: value.to_string(), style: String::new(), lifecycle: ComponentLifecycle::Created }
    }
}

impl VxComponent for Text {
    fn render_template(&self) -> TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        TemplateNode::Element {
            tag: "Text".to_string(),
            attributes: attrs,
            children: vec![TemplateNode::Text(self.value.clone())],
        }
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
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
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
    lifecycle: ComponentLifecycle,
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
            lifecycle: ComponentLifecycle::Created,
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

impl VxComponent for Input {
    fn render_template(&self) -> TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.placeholder.is_empty() {
            attrs.push(("placeholder".to_string(), self.placeholder.clone()));
        }
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let display_text = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            format!("{}|", self.value)
        };
        TemplateNode::Element {
            tag: "Input".to_string(),
            attributes: attrs,
            children: vec![TemplateNode::Text(display_text)],
        }
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
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
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
    lifecycle: ComponentLifecycle,
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
            lifecycle: ComponentLifecycle::Created,
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

impl VxComponent for Image {
    fn render_template(&self) -> TemplateNode {
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
        TemplateNode::Element { tag: "Image".to_string(), attributes: attrs, children: vec![] }
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
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
    }
}

/// 面板容器组件
pub struct Panel {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn VxComponent>>>,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: ComponentLifecycle,
}

impl Panel {
    /// 创建新的面板组件
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string(), children: Vec::new(), style: String::new(), lifecycle: ComponentLifecycle::Created }
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn VxComponent>>) {
        self.children.push(child);
    }
}

impl VxComponent for Panel {
    fn render_template(&self) -> TemplateNode {
        let mut attrs = vec![("id".to_string(), self.id.clone())];
        if !self.style.is_empty() {
            attrs.push(("style".to_string(), self.style.clone()));
        }
        let children = self
            .children
            .iter()
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { TemplateNode::Text(String::new()) })
            .collect();
        TemplateNode::Element { tag: "Panel".to_string(), attributes: attrs, children }
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

    fn children(&self) -> Vec<Arc<RwLock<dyn VxComponent>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
    }
}

/// 滚动视图组件
pub struct ScrollView {
    /// 组件 ID
    id: String,
    /// 子组件列表
    children: Vec<Arc<RwLock<dyn VxComponent>>>,
    /// 滚动方向
    direction: FlexDirection,
    /// 样式字符串
    style: String,
    /// 生命周期状态
    lifecycle: ComponentLifecycle,
}

impl ScrollView {
    /// 创建新的滚动视图组件
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            children: Vec::new(),
            direction: FlexDirection::Column,
            style: String::new(),
            lifecycle: ComponentLifecycle::Created,
        }
    }

    /// 设置滚动方向
    pub fn set_direction(&mut self, direction: FlexDirection) {
        self.direction = direction;
    }

    /// 添加子组件
    pub fn add_child(&mut self, child: Arc<RwLock<dyn VxComponent>>) {
        self.children.push(child);
    }
}

impl VxComponent for ScrollView {
    fn render_template(&self) -> TemplateNode {
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
            .map(|c| if let Ok(comp) = c.read() { comp.render_template() } else { TemplateNode::Text(String::new()) })
            .collect();
        TemplateNode::Element { tag: "ScrollView".to_string(), attributes: attrs, children }
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

    fn children(&self) -> Vec<Arc<RwLock<dyn VxComponent>>> {
        self.children.clone()
    }

    fn on_mount(&mut self) {
        self.lifecycle = ComponentLifecycle::Mounted;
    }

    fn on_update(&mut self) {
        self.lifecycle = ComponentLifecycle::Updated;
    }

    fn on_cleanup(&mut self) {
        self.lifecycle = ComponentLifecycle::Unmounted;
    }
}
