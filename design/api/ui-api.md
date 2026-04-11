# GG 引擎 UI API 文档

## 1. 编辑器 UI API

### 1.1 核心模块

#### 1.1.1 `gg_editor_ui::VxComponent` 特质

```rust
pub trait VxComponent: Send + Sync + 'static {
    /// 创建组件
    fn create(world: &mut World, entity: Entity) -> Self;
    
    /// 更新组件
    fn update(&mut self, world: &mut World, entity: Entity);
    
    /// 渲染组件
    fn render(&self, world: &World, entity: Entity, renderer: &mut dyn Renderer);
    
    /// 处理事件
    fn handle_event(&mut self, world: &mut World, entity: Entity, event: &Event);
    
    /// 销毁组件
    fn destroy(&mut self, world: &mut World, entity: Entity);
}
```

#### 1.1.2 `gg_editor_ui::GuiRuntime` 结构体

```rust
pub struct GuiRuntime {
    // 内部字段
}

impl GuiRuntime {
    /// 创建新的 GUI 运行时
    pub fn new() -> Self;
    
    /// 初始化 GUI 运行时
    pub fn initialize(&mut self, world: &mut World) -> GResult<()>;
    
    /// 加载 .vx 文件
    pub fn load_vx_file(&mut self, path: &Path) -> GResult<Entity>;
    
    /// 更新 GUI
    pub fn update(&mut self, world: &mut World) -> GResult<()>;
    
    /// 渲染 GUI
    pub fn render(&mut self, world: &World, renderer: &mut dyn Renderer) -> GResult<()>;
    
    /// 处理事件
    pub fn handle_event(&mut self, world: &mut World, event: &Event) -> GResult<()>;
}
```

### 1.2 组件 API

#### 1.2.1 `gg_editor_ui::components::Layout` 组件

```rust
#[derive(Debug, Clone)]
pub struct Layout {
    pub id: String,
    pub class: String,
    pub style: Style,
    pub children: Vec<Entity>,
    pub layout_type: LayoutType,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub flex_wrap: FlexWrap,
    pub gap: (f32, f32),
    pub padding: (f32, f32, f32, f32),
    pub margin: (f32, f32, f32, f32),
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
}

impl VxComponent for Layout {
    // 实现 VxComponent 特质
}
```

#### 1.2.2 `gg_editor_ui::components::Button` 组件

```rust
#[derive(Debug, Clone)]
pub struct Button {
    pub id: String,
    pub class: String,
    pub style: Style,
    pub text: String,
    pub is_clicked: bool,
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub onclick: Option<Box<dyn Fn(&mut World, Entity) + Send + Sync>>,
}

impl VxComponent for Button {
    // 实现 VxComponent 特质
}
```

#### 1.2.3 `gg_editor_ui::components::Text` 组件

```rust
#[derive(Debug, Clone)]
pub struct Text {
    pub id: String,
    pub class: String,
    pub style: Style,
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub color: Color,
    pub alignment: TextAlignment,
    pub word_wrap: bool,
}

impl VxComponent for Text {
    // 实现 VxComponent 特质
}
```

#### 1.2.4 `gg_editor_ui::components::Image` 组件

```rust
#[derive(Debug, Clone)]
pub struct Image {
    pub id: String,
    pub class: String,
    pub style: Style,
    pub texture: Option<Handle<TextureAsset>>,
    pub color: Color,
    pub scale_mode: ScaleMode,
}

impl VxComponent for Image {
    // 实现 VxComponent 特质
}
```

### 1.3 事件系统 API

#### 1.3.1 `gg_editor_ui::events::Event` 枚举

```rust
#[derive(Debug, Clone)]
pub enum Event {
    /// 鼠标事件
    MouseEvent(MouseEvent),
    /// 键盘事件
    KeyboardEvent(KeyboardEvent),
    /// 触摸事件
    TouchEvent(TouchEvent),
    /// 自定义事件
    CustomEvent(String, serde_json::Value),
}
```

#### 1.3.2 `gg_editor_ui::events::MouseEvent` 结构体

```rust
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub position: (f32, f32),
    pub button: MouseButton,
    pub modifiers: Modifiers,
}
```

#### 1.3.3 `gg_editor_ui::events::KeyboardEvent` 结构体

```rust
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub event_type: KeyboardEventType,
    pub key: Key,
    pub modifiers: Modifiers,
}
```

### 1.4 状态管理 API

#### 1.4.1 `gg_editor_ui::state::Signal` 结构体

```rust
pub struct Signal<T: Send + Sync + 'static> {
    // 内部字段
}

impl<T: Send + Sync + 'static> Signal<T> {
    /// 创建新的信号
    pub fn new(value: T) -> Self;
    
    /// 获取信号值
    pub fn get(&self) -> T;
    
    /// 设置信号值
    pub fn set(&mut self, value: T);
    
    /// 订阅信号变化
    pub fn subscribe<F: Fn(&T) + Send + Sync + 'static>(&mut self, callback: F);
    
    /// 取消订阅
    pub fn unsubscribe(&mut self, subscription_id: usize);
}
```

### 1.5 样式系统 API

#### 1.5.1 `gg_editor_ui::styles::Style` 结构体

```rust
#[derive(Debug, Clone)]
pub struct Style {
    pub properties: HashMap<String, StyleValue>,
}

impl Style {
    /// 创建新的样式
    pub fn new() -> Self;
    
    /// 设置样式属性
    pub fn set(&mut self, property: &str, value: StyleValue);
    
    /// 获取样式属性
    pub fn get(&self, property: &str) -> Option<&StyleValue>;
    
    /// 合并样式
    pub fn merge(&mut self, other: &Style);
}
```

## 2. 游戏 UI API

### 2.1 核心模块

#### 2.1.1 `gg_game_ui::Canvas` 组件

```rust
#[derive(Debug, Clone)]
pub struct Canvas {
    pub render_mode: RenderMode,
    pub sort_order: i32,
    pub pixel_perfect: bool,
    pub override_sorting: bool,
    pub sorting_layer_id: i32,
    pub sorting_order: i32,
}

impl Component for Canvas {
    // 实现 Component 特质
}
```

#### 2.1.2 `gg_game_ui::RectTransform` 组件

```rust
#[derive(Debug, Clone)]
pub struct RectTransform {
    pub anchor_min: (f32, f32),
    pub anchor_max: (f32, f32),
    pub pivot: (f32, f32),
    pub anchored_position: (f32, f32),
    pub size_delta: (f32, f32),
    pub local_scale: (f32, f32, f32),
    pub rotation: Quaternion,
}

impl Component for RectTransform {
    // 实现 Component 特质
}
```

### 2.2 组件 API

#### 2.2.1 `gg_game_ui::components::Image` 组件

```rust
#[derive(Debug, Clone)]
pub struct Image {
    pub sprite: Option<Handle<TextureAsset>>,
    pub color: Color,
    pub material: Option<Handle<MaterialAsset>>,
    pub raycast_target: bool,
    pub maskable: bool,
}

impl Component for Image {
    // 实现 Component 特质
}
```

#### 2.2.2 `gg_game_ui::components::Text` 组件

```rust
#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub font: Handle<FontAsset>,
    pub font_size: f32,
    pub color: Color,
    pub alignment: TextAlignment,
    pub line_spacing: f32,
    pub word_wrap: bool,
    pub raycast_target: bool,
}

impl Component for Text {
    // 实现 Component 特质
}
```

#### 2.2.3 `gg_game_ui::components::Button` 组件

```rust
#[derive(Debug, Clone)]
pub struct Button {
    pub interactable: bool,
    pub transition: ButtonTransition,
    pub onclick: Option<Box<dyn Fn(&mut World, Entity) + Send + Sync>>,
}

impl Component for Button {
    // 实现 Component 特质
}
```

#### 2.2.4 `gg_game_ui::components::InputField` 组件

```rust