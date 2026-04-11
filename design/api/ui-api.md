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
    
    /// 判断是否有脏标记
    fn is_dirty(&self) -> bool;
    
    /// 清除所有脏标记
    fn clear_dirty(&mut self);
    
    /// 获取当前脏标记
    fn get_dirty_flags(&self) -> DirtyFlag;
    
    /// 标记指定脏标记
    fn mark_dirty(&mut self, flag: DirtyFlag);
    
    /// 获取 UsageHints
    fn usage_hints(&self) -> UsageHints;
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
    
    /// 接收外部事件并标记脏节点
    pub fn process_event(&mut self, event: &GuiEvent);
    
    /// 仅在存在脏节点时执行重绘
    pub fn commit_render(&mut self);
    
    /// 手动标记节点为脏
    pub fn mark_dirty(&mut self, component_id: &str, flag: DirtyFlag);
    
    /// 检查是否有脏组件
    pub fn has_dirty_components(&self) -> bool;
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

### 1.6 DirtyFlag 系统

`DirtyFlag` 是编辑器 UI 的位标志系统，用于追踪组件的变化状态，实现按需更新。

**DirtyFlag 常量：**

| 常量 | 值 | 描述 |
|------|---|------|
| `NONE` | `0b00000` | 无脏标记 |
| `LAYOUT` | `0b00001` | 布局脏，需要重新计算布局 |
| `STYLE` | `0b00010` | 样式脏，需要重新应用样式 |
| `CONTENT` | `0b00100` | 内容脏，需要重新生成网格 |
| `TRANSFORM` | `0b01000` | 变换脏，需要重新计算变换矩阵 |
| `ALL` | `0b01111` | 所有脏标记 |

**位运算支持：**

```rust
bitflags! {
    pub struct DirtyFlag: u8 {
        const NONE     = 0b00000;
        const LAYOUT   = 0b00001;
        const STYLE    = 0b00010;
        const CONTENT  = 0b00100;
        const TRANSFORM = 0b01000;
        const ALL      = 0b01111;
    }
}

impl DirtyFlag {
    /// 判断是否包含指定标记
    pub fn contains(&self, flag: DirtyFlag) -> bool;

    /// 插入指定标记
    pub fn insert(&mut self, flag: DirtyFlag);

    /// 移除指定标记
    pub fn remove(&mut self, flag: DirtyFlag);

    /// 判断是否为空
    pub fn is_empty(&self) -> bool;

    /// 判断是否有任何标记
    pub fn is_any(&self) -> bool;
}
```

**使用示例：**

```rust
let mut flag = DirtyFlag::NONE;
flag.insert(DirtyFlag::LAYOUT);
flag.insert(DirtyFlag::CONTENT);

if flag.contains(DirtyFlag::LAYOUT) {
    // 重新计算布局
}

flag.remove(DirtyFlag::LAYOUT);
flag.clear_dirty();
```

### 1.7 UsageHints 系统

`UsageHints` 系统允许 UI 元素声明需要 GPU 驱动的变换类型，通过 uniform 传入着色器执行变换，避免 CPU 侧重建顶点数据。

**UsageHint 枚举：**

| 枚举值 | 描述 | GPU 变换方式 |
|-------|------|------------|
| `TransformOffset` | 位移偏移 | 顶点着色器中叠加偏移向量 |
| `ColorTint` | 颜色调色 | 片段着色器中乘以调色值 |
| `Opacity` | 透明度 | 片段着色器中乘以透明度系数 |
| `ScaleTransform` | 缩放变换 | 顶点着色器中应用缩放矩阵 |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsageHint {
    TransformOffset,
    ColorTint,
    Opacity,
    ScaleTransform,
}

pub struct UsageHints {
    hints: HashSet<UsageHint>,
}

impl UsageHints {
    /// 创建空的 UsageHints
    pub fn new() -> Self;

    /// 添加 hint
    pub fn add(&mut self, hint: UsageHint);

    /// 移除 hint
    pub fn remove(&mut self, hint: UsageHint);

    /// 判断是否包含指定 hint
    pub fn contains(&self, hint: UsageHint) -> bool;

    /// 获取所有 hints
    pub fn iter(&self) -> impl Iterator<Item = &UsageHint>;
}
```

**GpuTransformUniform 结构体：**

```rust
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GpuTransformUniform {
    /// 位移偏移 (UsageHint::TransformOffset)
    pub transform_offset: Vec2,
    /// 颜色调色 (UsageHint::ColorTint)
    pub color_tint: Vec4,
    /// 透明度 (UsageHint::Opacity)
    pub opacity: f32,
    /// 缩放变换 (UsageHint::ScaleTransform)
    pub scale_transform: Vec2,
}
```

**UsageHintsManager 管理器：**

```rust
pub struct UsageHintsManager {
    /// 元素 ID 到 UsageHints 的映射
    hints_map: HashMap<String, UsageHints>,
    /// GPU uniform 数据
    uniform_data: Vec<GpuTransformUniform>,
    /// 是否有脏数据需要更新
    dirty: bool,
}

impl UsageHintsManager {
    /// 创建新的管理器
    pub fn new() -> Self;

    /// 注册元素的 UsageHints
    pub fn register(&mut self, element_id: &str, hints: UsageHints);

    /// 注销元素的 UsageHints
    pub fn unregister(&mut self, element_id: &str);

    /// 更新元素的变换数据
    pub fn update_transform(&mut self, element_id: &str, uniform: GpuTransformUniform);

    /// 获取所有 uniform 数据
    pub fn uniform_data(&self) -> &[GpuTransformUniform];

    /// 判断是否有脏数据
    pub fn is_dirty(&self) -> bool;

    /// 清除脏标记
    pub fn clear_dirty(&mut self);
}
```

### 1.8 Uber-Shader 系统

`EditorUiUberShader` 是编辑器 UI 的超级着色器系统，将所有 UI 元素类型合并为单次 Draw Call 提交。

**UberShaderMode 枚举：**

| 枚举值 | 值 | 描述 |
|-------|---|------|
| `Rect` | `0` | 矩形填充，用于面板、按钮背景等 |
| `Text` | `1` | SDF 文字渲染，用于文本元素 |
| `Icon` | `2` | 图标渲染，用于矢量图标 |
| `Image` | `3` | 图片渲染，用于位图元素 |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum UberShaderMode {
    Rect = 0,
    Text = 1,
    Icon = 2,
    Image = 3,
}
```

**TextureAtlas 纹理图集：**

```rust
pub struct TextureAtlas {
    /// 图集纹理
    texture: Handle<TextureAsset>,
    /// 图集尺寸
    size: (u32, u32),
    /// 区域映射：名称 → (UV 偏移, UV 尺寸)
    regions: HashMap<String, (Vec2, Vec2)>,
}

impl TextureAtlas {
    /// 创建新的纹理图集
    pub fn new(size: (u32, u32)) -> Self;

    /// 添加区域
    pub fn add_region(&mut self, name: &str, offset: Vec2, size: Vec2);

    /// 获取区域的 UV 坐标
    pub fn get_uv(&self, name: &str) -> Option<(Vec2, Vec2)>;

    /// 获取图集纹理
    pub fn texture(&self) -> &Handle<TextureAsset>;
}
```

**EditorUiUberShader 管理器：**

```rust
pub struct EditorUiUberShader {
    /// 着色器程序
    shader: Handle<ShaderAsset>,
    /// 纹理图集
    atlas: TextureAtlas,
    /// 当前 Draw Call 的 uniform 缓冲区
    uniform_buffer: Vec<u8>,
}

impl EditorUiUberShader {
    /// 创建新的 Uber-Shader
    pub fn new(shader: Handle<ShaderAsset>, atlas_size: (u32, u32)) -> Self;

    /// 添加矩形绘制命令
    pub fn draw_rect(&mut self, rect: &Rect, color: Vec4);

    /// 添加文字绘制命令
    pub fn draw_text(&mut self, text: &TextInfo, atlas_region: &str);

    /// 添加图标绘制命令
    pub fn draw_icon(&mut self, icon: &IconInfo, atlas_region: &str);

    /// 添加图片绘制命令
    pub fn draw_image(&mut self, image: &ImageInfo, atlas_region: &str);

    /// 提交所有绘制命令为单次 Draw Call
    pub fn commit(&mut self, renderer: &mut dyn Renderer);

    /// 重置绘制命令
    pub fn reset(&mut self);
}
```

### 1.9 Flexbox 布局引擎

`FlexLayoutEngine` 是编辑器 UI 的 Flexbox 布局引擎，自行实现 CSS Flexbox 核心算法，不依赖外部 yoga crate。

**LayoutStyle 结构体：**

```rust
#[derive(Debug, Clone)]
pub struct LayoutStyle {
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_self: Option<AlignSelf>,
    pub align_content: AlignContent,
    pub flex_wrap: FlexWrap,
    pub gap: (f32, f32),
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Length,
    pub padding: (f32, f32, f32, f32),
    pub margin: (f32, f32, f32, f32),
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub position_type: PositionType,
    pub left: Length,
    pub right: Length,
    pub top: Length,
    pub bottom: Length,
}
```

**LayoutResult 结构体：**

```rust
#[derive(Debug, Clone, Copy)]
pub struct LayoutResult {
    /// 元素的最终 X 坐标
    pub x: f32,
    /// 元素的最终 Y 坐标
    pub y: f32,
    /// 元素的最终宽度
    pub width: f32,
    /// 元素的最终高度
    pub height: f32,
}
```

**LayoutNode 结构体：**

```rust
#[derive(Debug, Clone)]
pub struct LayoutNode {
    /// 节点的布局样式
    pub style: LayoutStyle,
    /// 节点的布局结果
    pub result: LayoutResult,
    /// 子节点 ID 列表
    pub children: Vec<String>,
    /// 父节点 ID
    pub parent: Option<String>,
    /// 是否为脏节点
    pub dirty: bool,
}
```

**FlexLayoutEngine 引擎：**

```rust
pub struct FlexLayoutEngine {
    /// 节点映射
    nodes: HashMap<String, LayoutNode>,
    /// 脏节点集合
    dirty_nodes: HashSet<String>,
}

impl FlexLayoutEngine {
    /// 创建新的布局引擎
    pub fn new() -> Self;

    /// 添加节点
    pub fn add_node(&mut self, id: &str, style: LayoutStyle, parent: Option<&str>);

    /// 移除节点
    pub fn remove_node(&mut self, id: &str);

    /// 更新节点样式
    pub fn update_style(&mut self, id: &str, style: LayoutStyle);

    /// 标记节点为脏
    pub fn mark_dirty(&mut self, id: &str);

    /// 执行布局计算（仅重算脏节点）
    pub fn compute(&mut self, available_width: f32, available_height: f32);

    /// 获取节点的布局结果
    pub fn get_result(&self, id: &str) -> Option<&LayoutResult>;

    /// 判断是否有脏节点
    pub fn has_dirty_nodes(&self) -> bool;
}
```

**UssStyleMapper 映射器：**

```rust
pub struct UssStyleMapper;

impl UssStyleMapper {
    /// 将 USS 样式属性映射为 LayoutStyle
    pub fn map(style: &Style) -> LayoutStyle;

    /// 将单个 USS 属性映射为布局属性
    pub fn map_property(property: &str, value: &StyleValue) -> Option<LayoutProperty>;
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

### 2.6 Canvas 合并-重建模式

Game UI 的 Canvas 采用合并-重建（Merge-Rebuild）渲染模式，每帧将 Canvas 下的 UI 网格动态合并后提交 GPU 渲染。

**CanvasDirtyFlag 位标志：**

| 标志 | 值 | 描述 |
|------|---|------|
| `VERTICES` | `0b001` | 顶点数据脏，需要重建网格 |
| `MATERIAL` | `0b010` | 材质数据脏，需要重新分组 |
| `LAYOUT` | `0b100` | 布局数据脏，需要重新计算布局 |

```rust
bitflags! {
    pub struct CanvasDirtyFlag: u8 {
        const VERTICES = 0b001;
        const MATERIAL = 0b010;
        const LAYOUT   = 0b100;
    }
}
```

**CanvasRenderMode 枚举：**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasRenderMode {
    /// 屏幕空间覆盖模式
    ScreenSpaceOverlay,
    /// 屏幕空间相机模式
    ScreenSpaceCamera,
    /// 世界空间模式
    WorldSpace,
}
```

**CanvasElement 结构体：**

```rust
#[derive(Debug, Clone)]
pub struct CanvasElement {
    /// 元素 ID
    pub id: String,
    /// 元素的网格数据
    pub mesh: MeshData,
    /// 元素的材质
    pub material: Handle<MaterialAsset>,
    /// 元素的纹理
    pub texture: Handle<TextureAsset>,
    /// 元素的脏标记
    pub dirty_flags: CanvasDirtyFlag,
    /// 元素的排序顺序
    pub sort_order: i32,
}
```

**CanvasBatch 结构体：**

```rust
#[derive(Debug, Clone)]
pub struct CanvasBatch {
    /// 批次中的元素 ID 列表
    pub element_ids: Vec<String>,
    /// 合并后的网格数据
    pub merged_mesh: MeshData,
    /// 批次使用的材质
    pub material: Handle<MaterialAsset>,
    /// 批次使用的纹理
    pub texture: Handle<TextureAsset>,
    /// 批次的顶点数量
    pub vertex_count: u32,
    /// 批次的索引数量
    pub index_count: u32,
}
```

**CanvasRebuilder 结构体：**

```rust
pub struct CanvasRebuilder {
    /// 当前帧的批次列表
    batches: Vec<CanvasBatch>,
    /// 是否需要重建
    needs_rebuild: bool,
}

impl CanvasRebuilder {
    /// 创建新的重建器
    pub fn new() -> Self;

    /// 标记需要重建
    pub fn mark_for_rebuild(&mut self, flag: CanvasDirtyFlag);

    /// 执行网格重建
    pub fn rebuild(&mut self, elements: &[CanvasElement]) -> &[CanvasBatch];

    /// 动静分离：将频繁变化的元素与静态元素分离
    pub fn separate_dynamic_elements(
        &mut self,
        canvas: &Canvas,
        threshold: f32,
    ) -> SeparationResult;

    /// 判断是否需要重建
    pub fn needs_rebuild(&self) -> bool;

    /// 获取当前批次
    pub fn batches(&self) -> &[CanvasBatch];
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