# Tasks

- [ ] Task 1: 重构 gg-editor-shell 为微内核架构
  - [ ] SubTask 1.1: 定义 `ServiceRegistry` 结构：`register<T: Any + Send + Sync>(&mut self, service: T)`、`get<T: Any>(&self) -> Option<&T>`、`get_mut<T: Any>(&mut self) -> Option<&mut T>`
  - [ ] SubTask 1.2: 定义 `Command` trait：`execute(&mut self, context: &mut EditorContext) -> GResult<()>`、`undo(&mut self, context: &mut EditorContext) -> GResult<()>`、`description(&self) -> &str`
  - [ ] SubTask 1.3: 定义 `CommandManager` 结构：`execute(&mut self, command: Box<dyn Command>, context: &mut EditorContext)`、`undo(&mut self, context: &mut EditorContext) -> GResult<()>`、`redo(&mut self, context: &mut EditorContext) -> GResult<()>`、`can_undo/can_redo`
  - [ ] SubTask 1.4: 定义 `EditorEvent` 枚举：`EntitySelected`、`EntityDeselected`、`PropertyChanged`、`FileChanged`、`SceneLoaded`、`SceneUnloaded`、`Custom(String, Box<dyn Any + Send + Sync>)`
  - [ ] SubTask 1.5: 定义 `EventBus` 结构：`subscribe(&mut self, handler: Box<dyn FnMut(&EditorEvent)>)`、`publish(&mut self, event: EditorEvent)`、`process_pending(&mut self)`
  - [ ] SubTask 1.6: 定义 `EditorContext` 结构：包含 `services: &mut ServiceRegistry`、`commands: &mut CommandManager`、`events: &mut EventBus`
  - [ ] SubTask 1.7: 定义 `EditorPlugin` trait：`name(&self) -> &str`、`initialize(&mut self, context: &mut EditorContext)`、`shutdown(&mut self, context: &mut EditorContext)`
  - [ ] SubTask 1.8: 重构 `EditorShell`：新增 `services: ServiceRegistry`、`commands: CommandManager`、`events: EventBus`、`plugins: Vec<Box<dyn EditorPlugin>>` 字段；`run()` 启动时调用所有插件的 `initialize`，关闭时调用 `shutdown`
  - [ ] SubTask 1.9: 重构 `EditorPanel` trait：新增 `on_register(&mut self, context: &mut EditorContext)`、`on_unregister(&mut self, context: &mut EditorContext)`；`render` 签名改为 `render(&mut self, context: &mut EditorContext) -> GResult<()>`
  - [ ] SubTask 1.10: 定义 `PanelLayoutHint` 结构：`position: PanelPosition`（Left/Right/Center/Bottom/Floating）、`preferred_size: Option<(f32, f32)>`、`min_size: Option<(f32, f32)>`；`EditorPanel` trait 新增 `layout_hint(&self) -> PanelLayoutHint`
  - [ ] SubTask 1.11: 移除 `PanelContext` 和 `PanelData`，所有现有面板迁移到使用 `EditorContext`
  - [ ] SubTask 1.12: 更新 `gg-editor-shell/Cargo.toml` 依赖

- [ ] Task 2: 增强属性检查器框架
  - [ ] SubTask 2.1: 定义 `PropertyType` 枚举：`String`、`Int`、`Float`、`Bool`、`Enum(Vec<String>)`、`Color`、`AssetPath(String)`、`Vec2`、`Custom(String)`
  - [ ] SubTask 2.2: 定义 `PropertyDescriptor` 结构：`name: String`、`display_name: String`、`property_type: PropertyType`、`default_value: Option<String>`、`constraints: Option<PropertyConstraints>`（包含 min/max/range 等）
  - [ ] SubTask 2.3: 定义 `ComponentDescriptor` 结构：`type_name: String`、`display_name: String`、`properties: Vec<PropertyDescriptor>`
  - [ ] SubTask 2.4: 定义 `DescriptorRegistry` 结构：`register_component(&mut self, descriptor: ComponentDescriptor)`、`get_component(&self, type_name: &str) -> Option<&ComponentDescriptor>`、`component_descriptors(&self) -> &[ComponentDescriptor]`
  - [ ] SubTask 2.5: 定义 `PropertyEditorFactory` trait：`can_edit(&self, property_type: &PropertyType) -> bool`、`create_editor(&self) -> Box<dyn PropertyEditorWidget>`
  - [ ] SubTask 2.6: 定义 `PropertyEditorWidget` trait：`set_value(&mut self, value: &str)`、`get_value(&self) -> String`、`is_modified(&self) -> bool`
  - [ ] SubTask 2.7: 定义 `PropertyEditorRegistry` 结构：`register_factory(&mut self, factory: Box<dyn PropertyEditorFactory>)`、`create_editor(&self, property_type: &PropertyType) -> Option<Box<dyn PropertyEditorWidget>>`
  - [ ] SubTask 2.8: 实现内置属性编辑器工厂：`StringEditorFactory`、`NumericEditorFactory`（Int/Float）、`BoolEditorFactory`、`EnumEditorFactory`、`ColorEditorFactory`、`AssetPathEditorFactory`
  - [ ] SubTask 2.9: 定义 `PropertyBinding` trait：`read(&self, world: &mut World, entity: Entity) -> Option<String>`、`write(&self, world: &mut World, entity: Entity, value: &str) -> GResult<()>`
  - [ ] SubTask 2.10: 定义 `SetPropertyCommand` 结构，实现 `Command` trait：执行时写入属性值，撤销时恢复旧值
  - [ ] SubTask 2.11: 重构 `InspectorPanel`：使用 `DescriptorRegistry` 驱动属性展示，使用 `PropertyEditorRegistry` 创建编辑器，使用 `CommandManager` 管理撤销/重做
  - [ ] SubTask 2.12: 注册 Galgame 组件描述符：为 `DialogueNode`、`PortraitState`、`AudioControl`、`SceneBackground` 创建 `ComponentDescriptor` 并注册
  - [ ] SubTask 2.13: 移除旧的 `PropertyEditor` trait 和硬编码编辑器（`DialogueNodeEditor` 等），替换为新的属性编辑器注册表

- [ ] Task 3: 创建 gg-editor-scene 通用场景视图基类
  - [ ] SubTask 3.1: 创建 `projects/editor/gg-editor-scene` 目录和 `Cargo.toml`，依赖 `gg-core`、`gg-ecs`、`gg-editor-shell`
  - [ ] SubTask 3.2: 定义 `SceneView` trait：继承 `EditorPanel`，新增 `on_scene_load(&mut self, context: &mut EditorContext)`、`on_scene_unload(&mut self, context: &mut EditorContext)`、`on_entity_selected(&mut self, entity: Entity, context: &mut EditorContext)`、`on_entity_moved(&mut self, entity: Entity, delta: (f32, f32), context: &mut EditorContext)`、`render_overlay(&mut self, context: &mut EditorContext) -> GResult<()>`
  - [ ] SubTask 3.3: 定义 `ViewportState` 结构：`offset: (f32, f32)`、`zoom: f32`、`size: (f32, f32)`；提供 `world_to_screen`、`screen_to_world` 坐标转换
  - [ ] SubTask 3.4: 实现 `BaseSceneView` 结构：包含 `ViewportState`、`selected_entities: Vec<Entity>`、`grid_visible: bool`；实现视口平移/缩放、实体选择、网格渲染等基础功能
  - [ ] SubTask 3.5: 在根 `Cargo.toml` 中添加 `gg-editor-scene` 到 workspace

- [ ] Task 4: 创建 gg-editor-lsp LSP 客户端接口
  - [ ] SubTask 4.1: 创建 `projects/editor/gg-editor-lsp` 目录和 `Cargo.toml`，依赖 `gg-core`、`serde`、`serde_json`
  - [ ] SubTask 4.2: 定义 LSP 基础类型：`Position`、`Range`、`Location`、`DiagnosticSeverity`、`Diagnostic`、`CompletionItem`、`Hover`、`TextDocumentIdentifier`、`VersionedTextDocumentIdentifier`
  - [ ] SubTask 4.3: 定义 `LspTransport` trait：`send_request(&mut self, method: &str, params: serde_json::Value) -> GResult<serde_json::Value>`、`send_notification(&mut self, method: &str, params: serde_json::Value) -> GResult<()>`
  - [ ] SubTask 4.4: 定义 `LspClient` 结构：封装 `LspTransport`，提供 `initialize`、`shutdown`、`did_open`、`did_change`、`did_close`、`completion`、`hover`、`goto_definition` 方法
  - [ ] SubTask 4.5: 定义 `DiagnosticCollector` 结构：`add_diagnostics(&mut self, uri: &str, diagnostics: Vec<Diagnostic>)`、`clear_diagnostics(&mut self, uri: &str)`、`get_diagnostics(&self, uri: &str) -> &[Diagnostic]`、`get_all_diagnostics(&self) -> &HashMap<String, Vec<Diagnostic>>`
  - [ ] SubTask 4.6: 在根 `Cargo.toml` 中添加 `gg-editor-lsp` 到 workspace

- [ ] Task 5: 迁移现有编辑器面板适配新架构
  - [ ] SubTask 5.1: 迁移 `gg-editor-asset-browser/AssetBrowserPanel`：`render` 签名改为 `&mut EditorContext`，移除 `PanelData` 依赖，通过 `EditorContext` 访问所需服务
  - [ ] SubTask 5.2: 迁移 `gg-editor-character/CharacterManagerPanel`：同上
  - [ ] SubTask 5.3: 迁移 `gg-editor-scene-galgame/GalgameSceneEditorPanel`：同上
  - [ ] SubTask 5.4: 迁移 `gg-editor-script/ScriptEditorPanel`：同上
  - [ ] SubTask 5.5: 迁移 `gg-editor-preview/PreviewPanel`：同上，`PreviewDebugger` 和 `HmrWatcher` 改为通过 `EditorContext` 获取
  - [ ] SubTask 5.6: 更新所有编辑器 crate 的 `Cargo.toml` 依赖

# Task Dependencies
- Task 2 depends on Task 1（属性检查器框架需要 `EditorContext`、`Command` trait 和 `CommandManager`）
- Task 3 depends on Task 1（场景视图基类需要 `EditorPanel` 新 trait 和 `EditorContext`）
- Task 5 depends on Task 1（面板迁移需要 `EditorContext` 和新的 `EditorPanel` trait）
- Task 4 无依赖，可与 Task 1 并行
