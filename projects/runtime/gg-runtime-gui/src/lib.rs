//! GG Editor 跨平台 GUI 运行时
//!
//! 提供 VX 组件体系、响应式状态管理、生命周期钩子和基础组件实现，
//! 对齐 *.vx 文件格式的 template/script/style 三段式结构。

#![warn(missing_docs)]

use std::{
    any::Any,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

/// 模板节点，描述 UI 结构
#[derive(Debug, Clone)]
pub enum TemplateNode {
    /// 元素节点
    Element {
        /// 标签名
        tag: String,
        /// 属性列表
        attributes: Vec<(String, String)>,
        /// 子节点
        children: Vec<TemplateNode>,
    },
    /// 文本节点
    Text(String),
}

/// Flex 布局方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    /// 水平方向排列
    Row,
    /// 垂直方向排列
    Column,
}

/// 组件生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentLifecycle {
    /// 已创建但未挂载
    Created,
    /// 已挂载到 DOM 树
    Mounted,
    /// 已更新
    Updated,
    /// 已卸载
    Unmounted,
}

/// VX 组件 trait，对齐 *.vx 文件格式的三段式结构
pub trait VxComponent: Any + Send + Sync {
    /// 渲染模板，返回 UI 节点描述
    fn render_template(&self) -> TemplateNode;

    /// 脚本初始化，设置响应式状态和事件处理
    fn script_setup(&mut self);

    /// 获取样式定义
    fn get_style(&self) -> Option<&str> {
        None
    }

    /// 获取组件 ID
    fn get_id(&self) -> &str;

    /// 处理 GUI 事件
    fn handle_event(&mut self, event: &GuiEvent);

    /// 组件挂载时调用
    fn on_mount(&mut self) {}

    /// 组件更新时调用
    fn on_update(&mut self) {}

    /// 组件卸载时调用
    fn on_cleanup(&mut self) {}
}

/// GUI 组件特质
#[deprecated(note = "使用 VxComponent 替代")]
pub trait GuiComponent: Any + Send + Sync {
    /// 渲染组件
    fn render(&self);

    /// 更新组件
    fn update(&mut self);

    /// 处理事件
    fn handle_event(&mut self, event: &GuiEvent);

    /// 获取组件ID
    fn get_id(&self) -> &str;

    /// 设置组件属性
    fn set_property(&mut self, name: &str, value: PropertyValue);

    /// 获取组件属性
    fn get_property(&self, name: &str) -> Option<PropertyValue>;
}

/// GUI 事件
#[derive(Debug)]
pub enum GuiEvent {
    /// 鼠标点击事件
    MouseClick {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
        /// 鼠标按钮
        button: MouseButton,
    },
    /// 鼠标移动事件
    MouseMove {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
    /// 键盘按键事件
    KeyPress {
        /// 按键
        key: Key,
        /// 修饰键
        modifiers: KeyModifiers,
    },
    /// 文本输入事件
    TextInput {
        /// 输入的文本
        text: String,
    },
    /// 组件特定事件
    Custom {
        /// 事件名称
        name: String,
        /// 事件数据
        data: Box<dyn Any>,
    },
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 其他按钮
    Other(u32),
}

/// 键盘按键
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A 键
    A,
    /// B 键
    B,
    /// C 键
    C,
    /// D 键
    D,
    /// E 键
    E,
    /// F 功能键
    FKey(u32),
    /// G 键
    G,
    /// H 键
    H,
    /// I 键
    I,
    /// J 键
    J,
    /// K 键
    K,
    /// L 键
    L,
    /// M 键
    M,
    /// N 键
    N,
    /// O 键
    O,
    /// P 键
    P,
    /// Q 键
    Q,
    /// R 键
    R,
    /// S 键
    S,
    /// T 键
    T,
    /// U 键
    U,
    /// V 键
    V,
    /// W 键
    W,
    /// X 键
    X,
    /// Y 键
    Y,
    /// Z 键
    Z,
    /// 数字键
    Number(u32),
    /// 空格键
    Space,
    /// 回车键
    Enter,
    /// ESC 键
    Escape,
    /// 退格键
    Backspace,
    /// Tab 键
    Tab,
    /// Shift 键
    Shift,
    /// Control 键
    Control,
    /// Alt 键
    Alt,
    /// 其他键
    Other(String),
}

/// 键盘修饰键
#[derive(Debug, Clone, Default)]
pub struct KeyModifiers {
    /// Shift 是否按下
    pub shift: bool,
    /// Control 是否按下
    pub control: bool,
    /// Alt 是否按下
    pub alt: bool,
    /// Meta 是否按下
    pub meta: bool,
}

/// 可克隆的对象 trait
pub trait CloneAny: Any + Send + Sync {
    /// 克隆为 Box<dyn CloneAny>
    fn clone_boxed(&self) -> Box<dyn CloneAny>;
}

impl<T: Any + Send + Sync + Clone + 'static> CloneAny for T {
    fn clone_boxed(&self) -> Box<dyn CloneAny> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CloneAny> {
    fn clone(&self) -> Self {
        (**self).clone_boxed()
    }
}

/// 属性值
#[derive(Clone)]
pub enum PropertyValue {
    /// 字符串值
    String(String),
    /// 数值
    Number(f64),
    /// 布尔值
    Boolean(bool),
    /// 对象值
    Object(Box<dyn CloneAny>),
    /// 数组值
    Array(Vec<PropertyValue>),
}

/// 响应式信号，支持 get/set/subscribe 操作
pub struct Signal<T> {
    /// 当前值
    value: T,
    /// 订阅者列表
    subscribers: Vec<Box<dyn Fn(&T)>>,
}

impl<T> Signal<T> {
    /// 创建新的响应式信号
    pub fn new(value: T) -> Self {
        Self { value, subscribers: Vec::new() }
    }

    /// 获取当前值的引用
    pub fn get(&self) -> &T {
        &self.value
    }

    /// 订阅值变化
    pub fn subscribe(&mut self, handler: Box<dyn Fn(&T)>) {
        self.subscribers.push(handler);
    }
}

impl<T: Clone> Signal<T> {
    /// 设置新值并通知订阅者
    pub fn set(&mut self, value: T) {
        self.value = value;
        for handler in &self.subscribers {
            handler(&self.value);
        }
    }
}

/// GUI 渲染器特质
pub trait GuiRenderer: Send + Sync {
    /// 渲染组件树
    fn render(&self, component: Arc<dyn VxComponent>);

    /// 处理事件
    fn process_events(&mut self);

    /// 更新渲染器
    fn update(&mut self);

    /// 设置视口大小
    fn set_viewport_size(&mut self, width: u32, height: u32);
}

/// GUI 运行时
pub struct GuiRuntime {
    /// 渲染器
    renderer: Arc<RwLock<dyn GuiRenderer>>,
    /// 根组件
    root_component: Option<Arc<RwLock<dyn VxComponent>>>,
    /// 是否正在运行
    is_running: bool,
    /// 目标帧率
    target_fps: Option<u32>,
}

impl GuiRuntime {
    /// 创建新的 GUI 运行时
    pub fn new<T: GuiRenderer + 'static>(renderer: T) -> Self {
        Self { renderer: Arc::new(RwLock::new(renderer)), root_component: None, is_running: false, target_fps: None }
    }

    /// 设置根组件
    pub fn set_root_component(&mut self, component: Arc<RwLock<dyn VxComponent>>) {
        self.root_component = Some(component);
    }

    /// 设置目标帧率
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = Some(fps);
    }

    /// 运行 GUI 循环
    pub fn run(&mut self) {
        self.is_running = true;
        while self.is_running {
            let frame_start = Instant::now();
            self.update();
            if let Some(fps) = self.target_fps {
                let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
                let elapsed = frame_start.elapsed();
                if elapsed < frame_duration {
                    std::thread::sleep(frame_duration - elapsed);
                }
            }
        }
    }

    /// 停止 GUI 循环
    pub fn shutdown(&mut self) {
        self.is_running = false;
    }

    /// 处理单个帧
    pub fn update(&mut self) {
        if let Ok(mut renderer) = self.renderer.write() {
            renderer.process_events();
        }

        if let Some(component) = &self.root_component {
            if let Ok(mut comp) = component.write() {
                comp.on_update();
            }
        }

        if let Some(component) = &self.root_component {
            if let Ok(_comp) = component.read() {
                if let Ok(renderer) = self.renderer.read() {
                    let arc_comp: Arc<dyn VxComponent> = Arc::new(ComponentWrapper { inner: Arc::clone(component) });
                    renderer.render(arc_comp);
                }
            }
        }

        if let Ok(mut renderer) = self.renderer.write() {
            renderer.update();
        }
    }
}

/// 组件包装器，用于将 RwLock 包装的组件转换为 Arc<dyn VxComponent>
struct ComponentWrapper {
    /// 内部组件
    inner: Arc<RwLock<dyn VxComponent>>,
}

impl VxComponent for ComponentWrapper {
    fn render_template(&self) -> TemplateNode {
        if let Ok(comp) = self.inner.read() { comp.render_template() } else { TemplateNode::Text(String::new()) }
    }

    fn script_setup(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.script_setup();
        }
    }

    fn get_style(&self) -> Option<&str> {
        None
    }

    fn get_id(&self) -> &str {
        if let Ok(comp) = self.inner.read() { comp.get_id().to_string().leak() } else { "" }
    }

    fn handle_event(&mut self, event: &GuiEvent) {
        if let Ok(mut comp) = self.inner.write() {
            comp.handle_event(event);
        }
    }

    fn on_mount(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_mount();
        }
    }

    fn on_update(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_update();
        }
    }

    fn on_cleanup(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_cleanup();
        }
    }
}

/// 平台特定的 GUI 工厂
pub trait GuiFactory {
    /// 创建 GUI 渲染器
    fn create_renderer(&self) -> Arc<dyn GuiRenderer>;

    /// 创建基础组件
    fn create_component(&self, component_type: &str) -> Arc<RwLock<dyn VxComponent>>;
}

/// 基础组件实现
pub mod components {
    use super::*;

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

        fn handle_event(&mut self, event: &GuiEvent) {
            for child in &self.children {
                if let Ok(mut comp) = child.write() {
                    comp.handle_event(event);
                }
            }
            let _ = event;
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

        fn handle_event(&mut self, event: &GuiEvent) {
            for child in &self.children {
                if let Ok(mut comp) = child.write() {
                    comp.handle_event(event);
                }
            }
            let _ = event;
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
            let mut attrs = vec![("id".to_string(), self.id.clone()), ("text".to_string(), self.text.clone())];
            if !self.style.is_empty() {
                attrs.push(("style".to_string(), self.style.clone()));
            }
            TemplateNode::Element { tag: "Button".to_string(), attributes: attrs, children: vec![] }
        }

        fn script_setup(&mut self) {}

        fn get_style(&self) -> Option<&str> {
            if self.style.is_empty() { None } else { Some(&self.style) }
        }

        fn get_id(&self) -> &str {
            &self.id
        }

        fn handle_event(&mut self, event: &GuiEvent) {
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

        fn handle_event(&mut self, _event: &GuiEvent) {}

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
            let mut attrs = vec![("id".to_string(), self.id.clone()), ("value".to_string(), self.value.clone())];
            if !self.placeholder.is_empty() {
                attrs.push(("placeholder".to_string(), self.placeholder.clone()));
            }
            if !self.style.is_empty() {
                attrs.push(("style".to_string(), self.style.clone()));
            }
            TemplateNode::Element { tag: "Input".to_string(), attributes: attrs, children: vec![] }
        }

        fn script_setup(&mut self) {}

        fn get_style(&self) -> Option<&str> {
            if self.style.is_empty() { None } else { Some(&self.style) }
        }

        fn get_id(&self) -> &str {
            &self.id
        }

        fn handle_event(&mut self, event: &GuiEvent) {
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

        fn handle_event(&mut self, _event: &GuiEvent) {}

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

        fn handle_event(&mut self, event: &GuiEvent) {
            for child in &self.children {
                if let Ok(mut comp) = child.write() {
                    comp.handle_event(event);
                }
            }
            let _ = event;
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

        fn handle_event(&mut self, event: &GuiEvent) {
            for child in &self.children {
                if let Ok(mut comp) = child.write() {
                    comp.handle_event(event);
                }
            }
            let _ = event;
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
}

#[allow(deprecated)]
mod legacy {
    use super::*;

    /// 布局组件（旧版 GuiComponent 实现）
    pub struct LegacyLayout {
        /// 组件 ID
        id: String,
        /// 子组件列表
        children: Vec<Arc<dyn GuiComponent>>,
        /// 样式
        style: PropertyValue,
        /// 类名
        class: String,
    }

    impl LegacyLayout {
        /// 创建新的布局组件
        pub fn new(id: &str) -> Self {
            Self { id: id.to_string(), children: Vec::new(), style: PropertyValue::String(String::new()), class: String::new() }
        }

        /// 添加子组件
        pub fn add_child(&mut self, child: Arc<dyn GuiComponent>) {
            self.children.push(child);
        }
    }

    #[allow(deprecated)]
    impl GuiComponent for LegacyLayout {
        fn render(&self) {
            for child in &self.children {
                child.render();
            }
        }

        fn update(&mut self) {}

        fn handle_event(&mut self, _event: &GuiEvent) {}

        fn get_id(&self) -> &str {
            &self.id
        }

        fn set_property(&mut self, name: &str, value: PropertyValue) {
            match name {
                "style" => self.style = value,
                "class" => {
                    if let PropertyValue::String(class) = value {
                        self.class = class;
                    }
                }
                _ => {}
            }
        }

        fn get_property(&self, name: &str) -> Option<PropertyValue> {
            match name {
                "style" => Some(self.style.clone()),
                "class" => Some(PropertyValue::String(self.class.clone())),
                _ => None,
            }
        }
    }

    /// 文本组件（旧版 GuiComponent 实现）
    pub struct LegacyText {
        /// 组件 ID
        id: String,
        /// 文本内容
        value: String,
        /// 样式
        style: PropertyValue,
        /// 类名
        class: String,
    }

    impl LegacyText {
        /// 创建新的文本组件
        pub fn new(id: &str, value: &str) -> Self {
            Self {
                id: id.to_string(),
                value: value.to_string(),
                style: PropertyValue::String(String::new()),
                class: String::new(),
            }
        }
    }

    #[allow(deprecated)]
    impl GuiComponent for LegacyText {
        fn render(&self) {}

        fn update(&mut self) {}

        fn handle_event(&mut self, _event: &GuiEvent) {}

        fn get_id(&self) -> &str {
            &self.id
        }

        fn set_property(&mut self, name: &str, value: PropertyValue) {
            match name {
                "value" => {
                    if let PropertyValue::String(v) = value {
                        self.value = v;
                    }
                }
                "style" => self.style = value,
                "class" => {
                    if let PropertyValue::String(class) = value {
                        self.class = class;
                    }
                }
                _ => {}
            }
        }

        fn get_property(&self, name: &str) -> Option<PropertyValue> {
            match name {
                "value" => Some(PropertyValue::String(self.value.clone())),
                "style" => Some(self.style.clone()),
                "class" => Some(PropertyValue::String(self.class.clone())),
                _ => None,
            }
        }
    }

    /// 按钮组件（旧版 GuiComponent 实现）
    pub struct LegacyButton {
        /// 组件 ID
        id: String,
        /// 按钮文本
        text: String,
        /// 样式
        style: PropertyValue,
        /// 类名
        class: String,
        /// 点击回调
        onclick: Option<Box<dyn Fn() + Send + Sync>>,
    }

    impl LegacyButton {
        /// 创建新的按钮组件
        pub fn new(id: &str, text: &str) -> Self {
            Self {
                id: id.to_string(),
                text: text.to_string(),
                style: PropertyValue::String(String::new()),
                class: String::new(),
                onclick: None,
            }
        }

        /// 设置点击回调
        pub fn set_onclick<F: Fn() + Send + Sync + 'static>(&mut self, callback: F) {
            self.onclick = Some(Box::new(callback));
        }
    }

    #[allow(deprecated)]
    impl GuiComponent for LegacyButton {
        fn render(&self) {}

        fn update(&mut self) {}

        fn handle_event(&mut self, event: &GuiEvent) {
            if let GuiEvent::MouseClick { .. } = event {
                if let Some(onclick) = &self.onclick {
                    onclick();
                }
            }
        }

        fn get_id(&self) -> &str {
            &self.id
        }

        fn set_property(&mut self, name: &str, value: PropertyValue) {
            match name {
                "text" => {
                    if let PropertyValue::String(text) = value {
                        self.text = text;
                    }
                }
                "style" => self.style = value,
                "class" => {
                    if let PropertyValue::String(class) = value {
                        self.class = class;
                    }
                }
                _ => {}
            }
        }

        fn get_property(&self, name: &str) -> Option<PropertyValue> {
            match name {
                "text" => Some(PropertyValue::String(self.text.clone())),
                "style" => Some(self.style.clone()),
                "class" => Some(PropertyValue::String(self.class.clone())),
                _ => None,
            }
        }
    }
}

#[allow(deprecated)]
pub use legacy::{LegacyButton, LegacyLayout, LegacyText};
