# Tasks

- [x] Task 1: 重构 gg-render 为渲染 HAL 抽象层
  - [x] SubTask 1.1: 定义核心类型：`TextureId`、`TextureDescriptor`、`SurfaceInfo`、`WindowEvent`、`Color`、`Rect`、`Transform`
  - [x] SubTask 1.2: 定义 `DrawCommand` 枚举：`Sprite`、`Text`、`Rect`、`Transition`，每个变体携带完整绘制参数
  - [x] SubTask 1.3: 定义 `Renderer` trait：`begin_frame`、`end_frame`、`draw`、`present`、`load_texture`、`resize`
  - [x] SubTask 1.4: 定义 `RenderContext` 结构，封装帧状态（DrawCommand 队列、当前帧尺寸等）
  - [x] SubTask 1.5: 移除原有占位实现 `RenderComponent`/`RenderSystem`
  - [x] SubTask 1.6: 更新 `gg-render/Cargo.toml` 依赖（移除 gg-ecs 依赖，渲染抽象不应依赖 ECS）

- [x] Task 2: 创建 gg-render-wgpu crate，实现 WGPU 渲染后端
  - [x] SubTask 2.1: 创建 `projects/runtime/gg-render-wgpu` 目录和 `Cargo.toml`，添加 wgpu、winit、image、ab_glyph 依赖
  - [x] SubTask 2.2: 实现 `WgpuRenderer` 结构：初始化 winit 窗口、wgpu 实例/适配器/设备/队列/surface
  - [x] SubTask 2.3: 实现窗口事件循环：处理 Resize、Close、FocusChange 事件，更新 surface 配置
  - [x] SubTask 2.4: 实现纹理加载：使用 image crate 解码 PNG/JPEG，创建 wgpu 纹理，返回 TextureId
  - [x] SubTask 2.5: 实现精灵渲染管线：创建顶点/索引缓冲、着色器、渲染管线，支持纹理采样和 alpha 混合
  - [x] SubTask 2.6: 实现文本渲染：使用 ab_glyph 光栅化字形，维护字形纹理图集缓存，渲染文本
  - [x] SubTask 2.7: 实现转场效果渲染：创建混合着色器，通过 uniform 传递进度参数
  - [x] SubTask 2.8: 实现 `Renderer` trait：`begin_frame`/`end_frame`/`draw`/`present`/`load_texture`/`resize`
  - [x] SubTask 2.9: 实现主循环：事件处理 → 清除帧缓冲 → 执行 DrawCommand 队列 → 提交渲染 → 呈现
  - [x] SubTask 2.10: 在根 `Cargo.toml` 中添加 `gg-render-wgpu` 到 workspace

- [x] Task 3: 创建 gg-ui crate，实现 UI 组件库
  - [x] SubTask 3.1: 创建 `projects/plugins/gg-ui` 目录和 `Cargo.toml`，依赖 gg-render
  - [x] SubTask 3.2: 定义 UI 核心类型：`UiNode`、`UiNodeId`、`UiTree`、`Style`、`LayoutResult`
  - [x] SubTask 3.3: 实现弹性布局引擎（Flexbox）：计算节点位置和尺寸，支持主轴/交叉轴对齐、换行、间距
  - [x] SubTask 3.4: 实现基础控件：`Button`（文本标签、点击回调、悬停/按下状态）
  - [x] SubTask 3.5: 实现基础控件：`TextBox`（多行文本、自动换行、滚动）
  - [x] SubTask 3.6: 实现基础控件：`Panel`（容器、背景色、边框、圆角、内边距）
  - [x] SubTask 3.7: 实现事件系统：输入事件路由、事件冒泡和拦截、事件处理器注册
  - [x] SubTask 3.8: 实现 UI 渲染：将 UiNode 树转换为 DrawCommand 序列，通过 Renderer trait 提交
  - [x] SubTask 3.9: 在根 `Cargo.toml` 中添加 `gg-ui` 到 workspace

- [ ] Task 4: 适配现有插件对接新渲染接口
  - [ ] SubTask 4.1: 更新 `gg-plugin-portrait`：`PortraitRenderSystem` 使用 `Renderer` trait 提交精灵绘制指令
  - [ ] SubTask 4.2: 更新 `gg-plugin-scene-transition`：转场系统使用 `DrawCommand::Transition` 提交渲染指令
  - [ ] SubTask 4.3: 更新 `gg-galgame-schema`：确保组件类型与新的渲染管线参数兼容
  - [ ] SubTask 4.4: 更新 `gg-editor-shell`：`EditorPanel::render` 操作 `UiNode` 树

- [ ] Task 5: 集成到 Galgame 引擎并验证
  - [ ] SubTask 5.1: 更新 `gg-galgame` 的 `Cargo.toml` 依赖，添加 `gg-render-wgpu` 和 `gg-ui`
  - [ ] SubTask 5.2: 重写 `GalgameEngine::initialize`：创建 `WgpuRenderer`，初始化窗口
  - [ ] SubTask 5.3: 重写 `GalgameEngine::run`：实现完整主循环（事件 → 逻辑 → 渲染 → 呈现）
  - [ ] SubTask 5.4: 实现对话框 UI：使用 gg-ui 的 TextBox 和 Panel 渲染对话内容
  - [ ] SubTask 5.5: 验证：运行 Galgame 引擎，确认窗口打开、背景/立绘/对话/转场正确渲染

# Task Dependencies
- Task 2 depends on Task 1（gg-render-wgpu 需要实现 gg-render 定义的 Renderer trait）
- Task 3 depends on Task 1（gg-ui 需要使用 gg-render 的 DrawCommand 和 Renderer trait）
- Task 4 depends on Task 1, Task 2（插件适配需要新的渲染抽象和 WGPU 实现）
- Task 5 depends on Task 2, Task 3, Task 4（集成需要所有模块就绪）
