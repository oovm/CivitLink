# WGPU 渲染后端与 UI 系统实现 Spec

## Why
当前 gg-render 仅为占位实现（只打印信息，无实际图形渲染），gg-ui 完全不存在，整个引擎没有可用的 GUI 渲染后端。按照架构设计文档，引擎需要基于 WGPU 实现跨平台渲染，并提供 UI 组件库支撑游戏界面和编辑器面板。

## What Changes
- 重构 `gg-render`：从占位实现改为渲染 HAL（硬件抽象层），定义 Renderer trait、DrawCommand、Texture、Surface 等核心抽象
- 新增 `gg-render-wgpu`：基于 WGPU 实现渲染后端，提供窗口管理、2D 精灵渲染、文本渲染、转场效果
- 新增 `gg-ui`：UI 组件库，提供布局系统、基础控件（按钮、文本框、面板）、事件系统
- **BREAKING** 重写 `gg-render` 的公共 API：原有 `RenderComponent`/`RenderSystem` 将被新的渲染抽象替代
- 更新 `gg-galgame-schema` 中的组件类型，使其适配新的渲染管线
- 更新 `gg-plugin-portrait`/`gg-plugin-scene-transition` 等插件，对接新的渲染接口

## Impact
- Affected specs: 渲染抽象层（HAL）、平台抽象层、UI 系统
- Affected code: `gg-render`（重写）、`gg-plugin-portrait`（适配）、`gg-plugin-scene-transition`（适配）、`gg-galgame-schema`（适配）、`gg-editor-shell`（适配）、新增 `gg-render-wgpu`、新增 `gg-ui`

## ADDED Requirements

### Requirement: 渲染硬件抽象层（HAL）
系统 SHALL 在 `gg-render` 中定义渲染硬件抽象层，包含核心 trait 和类型，使上层代码与具体 GPU API 解耦。

#### Scenario: 定义 Renderer trait
- **WHEN** 系统初始化渲染模块
- **THEN** `gg-render` 提供 `Renderer` trait，包含方法：`begin_frame`、`end_frame`、`draw_sprites`、`draw_text`、`present`
- **AND** `Renderer` trait 不依赖任何具体 GPU 库

#### Scenario: 定义 DrawCommand
- **WHEN** 游戏逻辑需要提交绘制指令
- **THEN** 系统提供 `DrawCommand` 枚举，包含变体：`Sprite`、`Text`、`Rect`、`Transition`
- **AND** 每个 `DrawCommand` 携带完整的绘制参数（位置、尺寸、颜色、纹理句柄等）

#### Scenario: 定义 Texture 抽象
- **WHEN** 系统需要加载和引用图像资源
- **THEN** `gg-render` 提供 `TextureId` 标识已加载纹理
- **AND** 提供 `TextureDescriptor` 描述纹理元数据（宽度、高度、格式）

#### Scenario: 定义 Surface 和 Window 抽象
- **WHEN** 系统需要创建渲染窗口
- **THEN** `gg-render` 提供 `SurfaceInfo` 结构描述窗口属性（宽度、高度、标题、全屏模式）
- **AND** 提供 `WindowEvent` 枚举描述窗口事件（Resize、Close、FocusChange）

### Requirement: WGPU 渲染后端
系统 SHALL 提供 `gg-render-wgpu` crate，基于 WGPU 实现 `gg-render` 定义的渲染 HAL。

#### Scenario: 初始化 WGPU 设备和表面
- **WHEN** 应用启动并请求创建渲染器
- **THEN** `WgpuRenderer` 创建 winit 窗口、请求 wgpu 适配器、设备和队列
- **AND** 配置 surface 格式为 `Bgra8UnormSrgb`，启用 alpha 混合

#### Scenario: 窗口事件处理
- **WHEN** 用户调整窗口大小或关闭窗口
- **THEN** `WgpuRenderer` 处理 winit 事件，更新 surface 配置
- **AND** 通知上层通过 `WindowEvent` 回调

#### Scenario: 2D 精灵渲染
- **WHEN** 游戏提交 `DrawCommand::Sprite` 指令
- **THEN** `WgpuRenderer` 使用纹理四边形渲染精灵
- **AND** 支持 Z 轴排序、透明度混合、缩放和位置变换

#### Scenario: 文本渲染
- **WHEN** 游戏提交 `DrawCommand::Text` 指令
- **THEN** `WgpuRenderer` 使用 `ab_glyph` 进行字形光栅化
- **AND** 将字形缓存为纹理图集，渲染到指定位置
- **AND** 支持字体大小、颜色、对齐方式配置

#### Scenario: 转场效果渲染
- **WHEN** 游戏提交 `DrawCommand::Transition` 指令
- **THEN** `WgpuRenderer` 使用混合着色器渲染淡入淡出、交叉溶解效果
- **AND** 通过 uniform 传递进度参数控制转场动画

#### Scenario: 纹理加载
- **WHEN** 游戏请求加载图像文件（PNG/JPEG）
- **THEN** `WgpuRenderer` 使用 `image` crate 解码图像数据
- **AND** 创建 wgpu 纹理并上传到 GPU
- **AND** 返回 `TextureId` 供后续绘制引用

#### Scenario: 主循环集成
- **WHEN** 游戏引擎运行主循环
- **THEN** `WgpuRenderer` 在每帧执行：处理窗口事件 → 清除帧缓冲 → 执行 DrawCommand 队列 → 提交渲染指令 → 呈现帧
- **AND** 帧率由垂直同步控制

### Requirement: UI 组件库
系统 SHALL 提供 `gg-ui` crate，实现声明式 UI 组件库，用于游戏界面和编辑器面板。

#### Scenario: UI 节点树
- **WHEN** 开发者构建 UI 界面
- **THEN** 系统提供 `UiNode` 树结构，支持嵌套组合
- **AND** 每个节点包含：布局属性、样式属性、事件处理器

#### Scenario: 布局系统
- **WHEN** UI 节点需要计算位置和尺寸
- **THEN** 系统实现弹性布局（Flexbox），支持主轴/交叉轴对齐、换行、间距
- **AND** 布局计算结果缓存，仅在节点属性变化时重新计算

#### Scenario: 基础控件 - 按钮
- **WHEN** 开发者创建按钮控件
- **THEN** 系统提供 `Button` 组件，支持文本标签、点击回调、悬停/按下状态样式
- **AND** 按钮自动处理鼠标事件并切换视觉状态

#### Scenario: 基础控件 - 文本框
- **WHEN** 开发者创建文本框控件
- **THEN** 系统提供 `TextBox` 组件，支持多行文本、自动换行、滚动
- **AND** 支持文本样式（字体、大小、颜色、行间距）

#### Scenario: 基础控件 - 面板
- **WHEN** 开发者创建面板控件
- **THEN** 系统提供 `Panel` 组件，作为容器组织子控件
- **AND** 支持背景色、边框、圆角、内边距

#### Scenario: 事件系统
- **WHEN** 用户与 UI 交互（点击、悬停、输入）
- **THEN** 系统将输入事件路由到目标 UI 节点
- **AND** 支持事件冒泡和拦截
- **AND** 事件处理器可修改 UI 状态触发重绘

#### Scenario: UI 渲染集成
- **WHEN** UI 需要渲染到屏幕
- **THEN** `gg-ui` 将 UI 节点树转换为 `DrawCommand` 序列
- **AND** 通过 `gg-render` 的 `Renderer` trait 提交渲染
- **AND** UI 层渲染在游戏场景层之上

### Requirement: Galgame 游戏界面集成
系统 SHALL 将渲染后端和 UI 系统集成到 Galgame 引擎，实现可运行的游戏画面。

#### Scenario: 对话框 UI
- **WHEN** Galgame 引擎显示对话内容
- **THEN** 使用 `gg-ui` 的 `TextBox` 和 `Panel` 渲染对话框
- **AND** 角色名字、对话文本、选项按钮正确显示

#### Scenario: 立绘渲染
- **WHEN** Galgame 引擎显示角色立绘
- **THEN** 使用 `gg-render-wgpu` 渲染立绘精灵
- **AND** 支持立绘位置（左/中/右）、透明度、Z 排序
- **AND** 说话角色高亮，非说话角色降低透明度

#### Scenario: 背景转场
- **WHEN** Galgame 引擎切换场景背景
- **THEN** 使用 `gg-render-wgpu` 的转场效果渲染
- **AND** 支持淡入淡出、交叉溶解转场动画

## MODIFIED Requirements

### Requirement: gg-render 模块
`gg-render` SHALL 从占位实现重构为渲染硬件抽象层。原有的 `RenderComponent` 和 `RenderSystem` 将被移除，替换为 `Renderer` trait、`DrawCommand` 枚举、`TextureId`、`SurfaceInfo` 等渲染抽象类型。

### Requirement: gg-plugin-portrait 系统
立绘渲染系统 SHALL 使用 `gg-render` 的 `Renderer` trait 提交精灵绘制指令，而非仅修改 ECS 组件状态。`PortraitRenderSystem` 需要访问 `Renderer` 资源来执行实际渲染。

### Requirement: gg-plugin-scene-transition 系统
转场系统 SHALL 使用 `gg-render` 的 `DrawCommand::Transition` 提交转场渲染指令，而非仅计算进度值。

### Requirement: gg-editor-shell 面板
编辑器面板 SHALL 使用 `gg-ui` 组件库渲染，而非空壳的 `EditorPanel` trait。`EditorPanel::render` 方法将操作 `UiNode` 树而非直接操作 `PanelContext`。

## REMOVED Requirements

### Requirement: 占位渲染实现
**Reason**: gg-render 当前仅打印信息的占位实现需要被实际的 WGPU 渲染后端替代
**Migration**: 所有依赖 `RenderComponent`/`RenderSystem` 的代码迁移到使用新的 `Renderer` trait 和 `DrawCommand`
