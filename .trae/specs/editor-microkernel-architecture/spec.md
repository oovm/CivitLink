# 编辑器微内核架构与核心框架 Spec

## Why
当前编辑器模块（`gg-editor-shell`、`gg-editor-inspector` 等）仅是占位实现，`EditorShell` 只是一个简单的面板容器，缺少微内核架构所需的命令系统、服务注册、事件总线和插件加载机制。按照 roadmap 编辑器组本月工作重点，需要设计编辑器微内核架构、实现基本窗口和命令系统、开始属性检查器框架设计、规划场景视图实现、调研 LSP 集成方案。

## What Changes
- 重构 `gg-editor-shell`：从简单面板容器升级为编辑器微内核，新增服务注册表、命令系统、事件总线、插件加载机制
- 新增 `gg-editor-lsp` crate：定义 LSP 客户端接口、文档同步和诊断处理
- 增强 `gg-editor-inspector`：新增属性描述符系统、属性编辑器注册表、属性绑定机制
- 新增 `gg-editor-scene` crate：定义通用场景视图基类和场景视图 trait
- **BREAKING** 重写 `EditorShell` 的公共 API：从简单的面板注册改为微内核服务架构
- **BREAKING** 重写 `EditorPanel` trait：增加生命周期方法和命令处理能力

## Impact
- Affected specs: 编辑器微内核架构、属性检查器框架、场景视图基类、LSP 集成
- Affected code: `gg-editor-shell`（重写核心）、`gg-editor-inspector`（增强框架）、新增 `gg-editor-lsp`、新增 `gg-editor-scene`、所有现有面板需适配新 `EditorPanel` trait

## ADDED Requirements

### Requirement: 编辑器微内核架构
系统 SHALL 在 `gg-editor-shell` 中实现微内核架构，提供服务注册、命令分发、事件通信和插件加载能力。

#### Scenario: 服务注册与发现
- **WHEN** 编辑器插件需要注册或使用某个服务（如 AssetDatabase、CommandManager）
- **THEN** 通过 `ServiceRegistry` 注册服务实例
- **AND** 其他插件通过 `ServiceRegistry` 按类型查找已注册的服务

#### Scenario: 命令系统
- **WHEN** 用户执行编辑器操作（如保存、撤销、创建实体）
- **THEN** 操作封装为 `Command` 对象，通过 `CommandManager` 执行
- **AND** `CommandManager` 维护撤销/重做栈，支持 `undo`/`redo` 操作
- **AND** 命令可通过命令 ID 注册和查找

#### Scenario: 事件总线
- **WHEN** 编辑器内部某个组件产生事件（如实体选中、属性变更、文件修改）
- **THEN** 通过 `EventBus` 发布事件
- **AND** 已订阅该事件类型的处理器收到通知

#### Scenario: 插件加载
- **WHEN** 编辑器启动并加载引擎清单中指定的编辑器插件
- **THEN** `EditorPlugin` trait 的 `initialize` 方法被调用，插件可注册面板、命令和服务
- **AND** 编辑器关闭时 `shutdown` 方法被调用，插件可清理资源

### Requirement: 窗口和面板布局管理
系统 SHALL 在 `gg-editor-shell` 中提供窗口和面板布局管理能力。

#### Scenario: 面板注册与布局
- **WHEN** 编辑器插件注册面板
- **THEN** 面板携带布局提示（位置、尺寸、可停靠区域）
- **AND** `EditorShell` 维护面板布局状态

#### Scenario: 面板可见性切换
- **WHEN** 用户通过命令或菜单切换面板可见性
- **THEN** 面板的 `set_visible` 被调用
- **AND** 布局自动调整

### Requirement: 属性检查器框架
系统 SHALL 在 `gg-editor-inspector` 中提供属性检查器框架，支持基于属性描述符的动态属性编辑。

#### Scenario: 属性描述符
- **WHEN** 组件类型需要被属性检查器展示
- **THEN** 通过 `PropertyDescriptor` 描述组件的每个可编辑属性（名称、类型、默认值、范围约束等）
- **AND** 属性描述符可手动注册或通过反射系统自动生成

#### Scenario: 属性编辑器注册表
- **WHEN** 属性检查器需要渲染某个类型的属性
- **THEN** 通过 `PropertyEditorRegistry` 查找该类型对应的编辑器
- **AND** 内置基本类型编辑器（字符串、数值、布尔、枚举、颜色、资源路径）
- **AND** 支持注册自定义类型编辑器

#### Scenario: 属性绑定
- **WHEN** 属性检查器需要读取或修改选中实体的组件属性
- **THEN** 通过 `PropertyBinding` 从 ECS World 读取组件字段值
- **AND** 修改属性时生成 `Command` 对象，通过命令系统执行以支持撤销/重做

#### Scenario: 检查器面板集成
- **WHEN** 用户选中一个实体
- **THEN** 属性检查器自动显示该实体所有组件的可编辑属性
- **AND** 修改属性后立即反映到 ECS World

### Requirement: 通用场景视图基类
系统 SHALL 在 `gg-editor-scene` 中定义通用场景视图 trait 和基类，为不同游戏类型提供场景编辑基础。

#### Scenario: 场景视图 trait
- **WHEN** 编辑器需要展示场景编辑视图
- **THEN** 实现 `SceneView` trait，包含方法：`on_scene_load`、`on_scene_unload`、`on_entity_selected`、`on_entity_moved`、`render_overlay`
- **AND** `SceneView` 继承 `EditorPanel` trait

#### Scenario: 场景视图基类
- **WHEN** 插件开发者需要创建特定游戏类型的场景视图
- **THEN** 继承 `BaseSceneView`，获得视口管理、实体选择、缩放平移等基础功能
- **AND** 只需实现游戏类型特定的渲染和交互逻辑

### Requirement: LSP 客户端接口
系统 SHALL 在 `gg-editor-lsp` 中定义 LSP 客户端接口，为脚本编辑器提供代码智能提示基础。

#### Scenario: LSP 客户端 trait
- **WHEN** 编辑器需要与语言服务器通信
- **THEN** 实现 `LspClient` trait，包含方法：`initialize`、`did_open`、`did_change`、`completion`、`hover`、`goto_definition`
- **AND** LSP 客户端不依赖具体传输实现（可使用 stdio、TCP 等）

#### Scenario: 文档同步
- **WHEN** 用户在脚本编辑器中编辑文件
- **THEN** LSP 客户端发送 `textDocument/didOpen` 和 `textDocument/didChange` 通知
- **AND** 接收语言服务器返回的诊断信息

#### Scenario: 诊断信息处理
- **WHEN** 语言服务器返回诊断信息（错误、警告）
- **THEN** `DiagnosticCollector` 收集并分类诊断信息
- **AND** 提供按文件和严重级别查询诊断的能力

## MODIFIED Requirements

### Requirement: EditorShell
`EditorShell` SHALL 从简单面板容器升级为微内核架构。原有 `EditorShell::new()`、`register_panel()`、`tick()`、`run()` 接口保留但内部实现重构，新增 `service_registry()`、`command_manager()`、`event_bus()` 访问器。`EditorShell` 在 `run()` 启动时初始化所有已注册插件。

### Requirement: EditorPanel trait
`EditorPanel` trait SHALL 增加生命周期方法。新增 `on_register(&mut self, context: &mut EditorContext)` 和 `on_unregister(&mut self, context: &mut EditorContext)` 方法。`render` 方法签名改为接收 `&mut EditorContext` 而非 `&mut PanelContext`，以访问服务注册表和命令管理器。

### Requirement: InspectorPanel
`InspectorPanel` SHALL 使用新的属性描述符系统和属性编辑器注册表来驱动属性展示和编辑，而非硬编码的 `PropertyEditor` 实现。撤销/重做功能通过 `CommandManager` 统一管理。

## REMOVED Requirements

### Requirement: PanelContext 和 PanelData
**Reason**: 微内核架构中，面板通过 `EditorContext` 访问服务注册表获取所需服务，不再需要 `PanelContext`/`PanelData` 作为中间层。World 访问、选中实体、项目路径等通过服务注册表获取。
**Migration**: 所有使用 `PanelContext`/`PanelData` 的代码迁移到使用 `EditorContext` 和服务注册表。
