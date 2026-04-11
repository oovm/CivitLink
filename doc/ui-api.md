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
#[derive(Debug, Clone)]
pub struct InputField {
    pub text: String,
    pub placeholder: String,
    pub font: Handle<FontAsset>,
    pub font_size: f32,
    pub color: Color,
    pub placeholder_color: Color,
    pub caret_color: Color,
    pub selection_color: Color,
    pub interactable: bool,
    pub read_only: bool,
    pub character_limit: i32,
    pub on_value_changed: Option<Box<dyn Fn(&mut World, Entity, &str) + Send + Sync>>,
    pub on_end_edit: Option<Box<dyn Fn(&mut World, Entity, &str) + Send + Sync>>,
}

impl Component for InputField {
    // 实现 Component 特质
}
```

### 2.3 布局系统 API

#### 2.3.1 `gg_game_ui::layout::LayoutGroup` 组件

```rust
#[derive(Debug, Clone)]
pub struct LayoutGroup {
    pub padding: (f32, f32, f32, f32),
    pub spacing: f32,
    pub child_alignment: TextAlignment,
}

impl Component for LayoutGroup {
    // 实现 Component 特质
}
```

#### 2.3.2 `gg_game_ui::layout::HorizontalLayoutGroup` 组件

```rust
#[derive(Debug, Clone)]
pub struct HorizontalLayoutGroup {
    pub layout_group: LayoutGroup,
    pub child_control_width: bool,
    pub child_control_height: bool,
    pub child_force_expand_width: bool,
    pub child_force_expand_height: bool,
}

impl Component for HorizontalLayoutGroup {
    // 实现 Component 特质
}
```

#### 2.3.3 `gg_game_ui::layout::VerticalLayoutGroup` 组件

```rust
#[derive(Debug, Clone)]
pub struct VerticalLayoutGroup {
    pub layout_group: LayoutGroup,
    pub child_control_width: bool,
    pub child_control_height: bool,
    pub child_force_expand_width: bool,
    pub child_force_expand_height: bool,
}

impl Component for VerticalLayoutGroup {
    // 实现 Component 特质
}
```

#### 2.3.4 `gg_game_ui::layout::GridLayoutGroup` 组件

```rust
#[derive(Debug, Clone)]
pub struct GridLayoutGroup {
    pub layout_group: LayoutGroup,
    pub cell_size: (f32, f32),
    pub spacing: (f32, f32),
    pub start_corner: GridCorner,
    pub start_axis: GridAxis,
    pub constraint: GridConstraint,
    pub constraint_count: i32,
}

impl Component for GridLayoutGroup {
    // 实现 Component 特质
}
```

### 2.4 事件系统 API

#### 2.4.1 `gg_game_ui::events::EventSystem` 结构体

```rust
pub struct EventSystem {
    // 内部字段
}

impl EventSystem {
    /// 创建新的事件系统
    pub fn new() -> Self;
    
    /// 处理事件
    pub fn process_events(&mut self, world: &mut World) -> GResult<()>;
    
    /// 注册事件处理器
    pub fn register_handler<F: Fn(&mut World, Entity, &Event) + Send + Sync + 'static>(&mut self, handler: F);
}

impl System for EventSystem {
    // 实现 System 特质
}
```

### 2.5 动画系统 API

#### 2.5.1 `gg_game_ui::animation::Animation` 组件

```rust
#[derive(Debug, Clone)]
pub struct Animation {
    pub clips: Vec<AnimationClip>,
    pub current_clip: Option<String>,
    pub state: AnimationState,
    pub time: f32,
    pub speed: f32,
    pub looped: bool,
}

impl Component for Animation {
    // 实现 Component 特质
}
```

#### 2.5.2 `gg_game_ui::animation::AnimationClip` 结构体

```rust
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub curves: Vec<AnimationCurve>,
    pub length: f32,
    pub looped: bool,
}
```

#### 2.5.3 `gg_game_ui::animation::AnimationCurve` 结构体

```rust
#[derive(Debug, Clone)]
pub struct AnimationCurve {
    pub keyframes: Vec<Keyframe>,
    pub looped: bool,
    pub speed: f32,
}
```

## 3. 资源管理 API

### 3.1 核心模块

#### 3.1.1 `gg_asset::AssetServer` 结构体

```rust
pub struct AssetServer {
    // 内部字段
}

impl AssetServer {
    /// 创建新的资源服务器
    pub fn new() -> Self;
    
    /// 加载资源
    pub fn load<T: Asset>(&mut self, path: &Path, is_editor_ui: bool) -> GResult<Handle<T>>;
    
    /// 获取资源
    pub fn get<T: Asset>(&self, handle: &Handle<T>) -> Option<&T>;
    
    /// 获取资源可变引用
    pub fn get_mut<T: Asset>(&mut self, handle: &Handle<T>) -> Option<&mut T>;
    
    /// 释放资源
    pub fn release<T: Asset>(&mut self, handle: Handle<T>);
    
    /// 启用热更新
    pub fn enable_hot_reload(&mut self, enabled: bool);
    
    /// 检查热更新
    pub fn check_hot_reload(&mut self) -> GResult<Vec<u64>>;
    
    /// 清理未使用的资源
    pub fn cleanup_unused(&mut self, max_age: Duration);
}
```

### 3.2 资源类型

#### 3.2.1 `gg_asset::Asset` 特质

```rust
pub trait Asset: Send + Sync + 'static {
    /// 获取资源类型
    fn asset_type() -> AssetType;
    
    /// 加载资源
    fn load(path: &Path) -> GResult<Self>;
    
    /// 卸载资源
    fn unload(&mut self);
    
    /// 获取资源大小
    fn size(&self) -> usize;
}
```

#### 3.2.2 `gg_asset::TextureAsset` 结构体

```rust
#[derive(Debug, Clone)]
pub struct TextureAsset {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Asset for TextureAsset {
    // 实现 Asset 特质
}
```

#### 3.2.3 `gg_asset::FontAsset` 结构体

```rust
#[derive(Debug, Clone)]
pub struct FontAsset {
    pub data: Vec<u8>,
}

impl Asset for FontAsset {
    // 实现 Asset 特质
}
```

#### 3.2.4 `gg_asset::MaterialAsset` 结构体

```rust
#[derive(Debug, Clone)]
pub struct MaterialAsset {
    pub data: Vec<u8>,
}

impl Asset for MaterialAsset {
    // 实现 Asset 特质
}
```

## 4. 性能优化 API

### 4.1 核心模块

#### 4.1.1 `gg_ui_perf::PerformanceOptimizer` 结构体

```rust
pub struct PerformanceOptimizer {
    // 内部字段
}

impl PerformanceOptimizer {
    /// 创建新的性能优化器
    pub fn new() -> Self;
    
    /// 启用优化策略
    pub fn enable_strategy(&mut self, strategy: OptimizationStrategy);
    
    /// 禁用优化策略
    pub fn disable_strategy(&mut self, strategy: OptimizationStrategy);
    
    /// 执行批处理优化
    pub fn optimize_batching(&mut self, commands: Vec<RenderCommand>) -> GResult<()>;
    
    /// 执行缓存优化
    pub fn optimize_caching(&mut self, key: &str, data: Vec<u8>) -> GResult<Arc<Mutex<CacheItem<Vec<u8>>>>;
    
    /// 执行多线程优化
    pub fn optimize_multi_threading(&self, tasks: Vec<Box<dyn FnOnce() + Send + 'static>>);
    
    /// 执行内存池优化
    pub fn optimize_memory_pooling(&mut self) -> Vec<f32>;
    
    /// 释放内存池对象
    pub fn release_memory_pool_object(&mut self, obj: Vec<f32>);
    
    /// 优化编辑器UI性能
    pub fn optimize_editor_ui(&mut self) -> GResult<()>;
    
    /// 优化游戏UI性能
    pub fn optimize_game_ui(&mut self) -> GResult<()>;
}
```

### 4.2 性能分析 API

#### 4.2.1 `gg_ui_perf::PerformanceAnalyzer` 结构体

```rust
pub struct PerformanceAnalyzer {
    // 内部字段
}

impl PerformanceAnalyzer {
    /// 创建新的性能分析工具
    pub fn new() -> Self;
    
    /// 开始分析
    pub fn start_analyzing(&mut self);
    
    /// 停止分析
    pub fn stop_analyzing(&mut self);
    
    /// 记录绘制调用
    pub fn record_draw_call(&mut self);
    
    /// 记录三角形数量
    pub fn record_triangles(&mut self, count: u32);
    
    /// 记录顶点数量
    pub fn record_vertices(&mut self, count: u32);
    
    /// 记录渲染时间
    pub fn record_render_time(&mut self, time: f32);
    
    /// 记录布局计算时间
    pub fn record_layout_time(&mut self, time: f32);
    
    /// 记录事件处理时间
    pub fn record_event_time(&mut self, time: f32);
    
    /// 记录内存使用
    pub fn record_memory_usage(&mut self, usage: usize);
    
    /// 记录帧率
    pub fn record_frame_rate(&mut self, frame_rate: f32);
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &PerformanceStats;
    
    /// 生成性能报告
    pub fn generate_report(&self) -> String;
    
    /// 分析性能瓶颈
    pub fn analyze_bottlenecks(&self) -> Vec<String>;
}
```

## 5. 工具 API

### 5.1 编辑器 UI 工具

#### 5.1.1 `gg_ui_tools::EditorUiEditor` 结构体

```rust
pub struct EditorUiEditor {
    // 内部字段
}

impl EditorUiEditor {
    /// 创建新的编辑器UI编辑器
    pub fn new() -> GResult<Self>;
    
    /// 启动编辑器
    pub fn start(&mut self) -> GResult<()>;
    
    /// 停止编辑器
    pub fn stop(&mut self) -> GResult<()>;
    
    /// 打开文件
    pub fn open_file(&mut self, path: &PathBuf) -> GResult<()>;
    
    /// 保存文件
    pub fn save_file(&mut self, path: &PathBuf) -> GResult<()>;
    
    /// 预览UI
    pub fn preview(&self) -> GResult<()>;
    
    /// 导出UI
    pub fn export(&self, path: &PathBuf) -> GResult<()>;
    
    /// 编辑组件
    pub fn edit_component(&mut self, component_id: &str, properties: &HashMap<String, String>) -> GResult<()>;
    
    /// 添加组件
    pub fn add_component(&mut self, component_type: &str, parent_id: &str) -> GResult<String>;
    
    /// 删除组件
    pub fn remove_component(&mut self, component_id: &str) -> GResult<()>;
}
```

### 5.2 游戏 UI 工具

#### 5.2.1 `gg_ui_tools::GameUiEditor` 结构体

```rust
pub struct GameUiEditor {
    // 内部字段
}

impl GameUiEditor {
    /// 创建新的游戏UI编辑器
    pub fn new() -> GResult<Self>;
    
    /// 启动编辑器
    pub fn start(&mut self) -> GResult<()>;
    
    /// 停止编辑器
    pub fn stop(&mut self) -> GResult<()>;
    
    /// 打开文件
    pub fn open_file(&mut self, path: &PathBuf) -> GResult<()>;
    
    /// 保存文件
    pub fn save_file(&mut self, path: &PathBuf) -> GResult<()>;
    
    /// 预览UI
    pub fn preview(&self) -> GResult<()>;
    
    /// 导出UI
    pub fn export(&self, path: &PathBuf) -> GResult<()>;
    
    /// 添加UI元素
    pub fn add_ui_element(&mut self, element_type: &str, parent_id: &str) -> GResult<String>;
    
    /// 删除UI元素
    pub fn remove_ui_element(&mut self, element_id: &str) -> GResult<()>;
    
    /// 设置UI元素属性
    pub fn set_ui_element_property(&mut self, element_id: &str, property: &str, value: &str) -> GResult<()>;
    
    /// 预览游戏UI
    pub fn preview_game_ui(&self, scene_path: &PathBuf) -> GResult<()>;
}
```

### 5.3 UI 性能分析工具

#### 5.3.1 `gg_ui_tools::UiProfiler` 结构体

```rust
pub struct UiProfiler {
    // 内部字段
}

impl UiProfiler {
    /// 创建新的UI性能分析器
    pub fn new() -> Self;
    
    /// 开始分析
    pub fn start_profiling(&mut self);
    
    /// 停止分析
    pub fn stop_profiling(&mut self);
    
    /// 记录绘制调用
    pub fn record_draw_call(&mut self);
    
    /// 记录三角形数量
    pub fn record_triangles(&mut self, count: u32);
    
    /// 记录顶点数量
    pub fn record_vertices(&mut self, count: u32);
    
    /// 记录渲染时间
    pub fn record_render_time(&mut self, time: f32);
    
    /// 记录布局计算时间
    pub fn record_layout_time(&mut self, time: f32);
    
    /// 记录事件处理时间
    pub fn record_event_time(&mut self, time: f32);
    
    /// 记录内存使用
    pub fn record_memory_usage(&mut self, usage: usize);
    
    /// 获取性能数据
    pub fn get_performance_data(&self) -> &PerformanceData;
    
    /// 导出性能报告
    pub fn export_report(&self, path: &PathBuf) -> GResult<()>;
    
    /// 分析性能瓶颈
    pub fn analyze_bottlenecks(&self) -> Vec<String>;
}
```

## 6. 使用示例

### 6.1 编辑器 UI 示例

```rust
use gg_editor_ui::{GuiRuntime, components::*};
use gg_ecs::World;

fn main() -> GResult<()> {
    let mut world = World::new();
    let mut gui_runtime = GuiRuntime::new();
    
    // 初始化 GUI 运行时
    gui_runtime.initialize(&mut world)?;
    
    // 加载 .vx 文件
    let ui_entity = gui_runtime.load_vx_file(Path::new("ui/main.vx"))?;
    
    // 主循环
    loop {
        // 更新 GUI
        gui_runtime.update(&mut world)?;
        
        // 渲染 GUI
        gui_runtime.render(&world, &mut renderer)?;
        
        // 处理事件
        if let Some(event) = get_event() {
            gui_runtime.handle_event(&mut world, &event)?;
        }
    }
}
```

### 6.2 游戏 UI 示例

```rust
use gg_game_ui::{Canvas, RectTransform, components::*};
use gg_ecs::World;

fn main() -> GResult<()> {
    let mut world = World::new();
    
    // 创建 Canvas
    let canvas_entity = world.create_entity();
    world.add_component(canvas_entity, Canvas {
        render_mode: RenderMode::ScreenSpaceOverlay,
        sort_order: 0,
        pixel_perfect: false,
        override_sorting: false,
        sorting_layer_id: 0,
        sorting_order: 0,
    });
    
    // 创建按钮
    let button_entity = world.create_entity();
    world.add_component(button_entity, RectTransform {
        anchor_min: (0.5, 0.5),
        anchor_max: (0.5, 0.5),
        pivot: (0.5, 0.5),
        anchored_position: (0.0, 0.0),
        size_delta: (200.0, 50.0),
        local_scale: (1.0, 1.0, 1.0),
        rotation: Quaternion::identity(),
    });
    world.add_component(button_entity, Image {
        sprite: None,
        color: Color::new(0.0, 0.5, 1.0, 1.0),
        material: None,
        raycast_target: true,
        maskable: true,
    });
    world.add_component(button_entity, Text {
        text: "Click Me".to_string(),
        font: font_handle,
        font_size: 20.0,
        color: Color::new(1.0, 1.0, 1.0, 1.0),
        alignment: TextAlignment::Center,
        line_spacing: 1.0,
        word_wrap: false,
        raycast_target: false,
    });
    world.add_component(button_entity, Button {
        interactable: true,
        transition: ButtonTransition::ColorTint,
        onclick: Some(Box::new(|world, entity| {
            println!("Button clicked!");
        })),
    });
    
    // 主循环
    loop {
        // 更新游戏
        world.execute_systems()?;
        
        // 渲染游戏
        render_game(&world)?;
    }
}
```

## 7. 结论

GG 引擎的 UI API 提供了丰富的功能和灵活的接口，支持编辑器 UI 和游戏 UI 的开发。通过分离这两个系统，我们能够为不同的使用场景提供优化的解决方案。编辑器 UI 提供了声明式的开发方式，适合创建复杂的编辑器界面；游戏 UI 提供了基于 GameObject 的开发方式，适合创建游戏内的 UI 元素。

同时，我们还提供了资源管理、性能优化和工具集成等功能，确保 UI 系统的高效运行和开发体验。未来，我们将继续扩展和优化 UI API，为开发者提供更好的工具和功能。
