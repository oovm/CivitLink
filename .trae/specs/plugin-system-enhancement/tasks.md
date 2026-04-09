# Tasks

- [ ] Task 1: 增强 Plugin trait 和 PluginRegistrar
  - [ ] SubTask 1.1: 在 `gg-core::plugin` 中定义 `PluginRegistrar` 结构，包含待注册系统队列、待注册资源队列、依赖列表
  - [ ] SubTask 1.2: 为 `PluginRegistrar` 实现 `register_system`、`insert_resource`、`add_dependency` 方法
  - [ ] SubTask 1.3: 为 `PluginRegistrar` 实现 `apply` 方法，将收集的注册信息统一应用到 World
  - [ ] SubTask 1.4: 修改 `Plugin` trait，增加 `build(&self, registrar: &mut PluginRegistrar)` 方法（提供默认空实现）
  - [ ] SubTask 1.5: 修改 `Plugin` trait，增加 `dependencies(&self) -> Vec<&str>` 方法（提供默认空实现）
  - [ ] SubTask 1.6: 在 `gg-core::plugin` 中实现 `PluginManager`，管理插件加载、依赖检查、生命周期

- [ ] Task 2: 适配现有插件到新 Plugin trait
  - [ ] SubTask 2.1: 更新 `DialoguePlugin`：在 `build` 中注册 `DialogueSystem`、`ChoiceSystem`、`TypewriterSystem` 和 `DialogueHistory`、`GameVariables`、`ChoiceState` 资源
  - [ ] SubTask 2.2: 更新 `PortraitPlugin`：在 `build` 中注册 `PortraitRenderSystem`、`PortraitAnimationSystem`
  - [ ] SubTask 2.3: 更新 `SavePlugin`：在 `build` 中注册存档相关系统和资源
  - [ ] SubTask 2.4: 更新 `SceneTransitionPlugin`：在 `build` 中注册 `TransitionSystem`

- [ ] Task 3: 重构对话系统命令处理器
  - [ ] SubTask 3.1: 修改 `CommandHandler` trait 签名为 `execute(&self, command: &DialogueCommand, world: &mut World) -> GResult<()>`
  - [ ] SubTask 3.2: 重写 `PlayBgmHandler`：从 `DialogueCommand::PlayBgm` 提取 `asset_path`、`volume`、`fade_in_secs` 写入 `AudioControl`
  - [ ] SubTask 3.3: 重写 `StopBgmHandler`：从 `DialogueCommand::StopBgm` 提取 `fade_out_secs` 更新 `AudioControl`
  - [ ] SubTask 3.4: 重写 `PlaySeHandler`：从 `DialogueCommand::PlaySe` 提取 `asset_path`、`volume` 添加到 `AudioControl::pending_se`
  - [ ] SubTask 3.5: 重写 `ShowPortraitHandler`：从 `DialogueCommand::ShowPortrait` 提取 `character_id`、`expression`、`position` 创建 `PortraitState`
  - [ ] SubTask 3.6: 重写 `HidePortraitHandler`：从 `DialogueCommand::HidePortrait` 提取 `character_id` 移除对应 `PortraitState`
  - [ ] SubTask 3.7: 重写 `ChangeBackgroundHandler`：从 `DialogueCommand::ChangeBackground` 提取 `asset_path`、`transition` 更新 `SceneBackground`
  - [ ] SubTask 3.8: 重写 `SetVariableHandler`：从 `DialogueCommand::SetVariable` 提取 `name`、`value` 写入 `GameVariables`
  - [ ] SubTask 3.9: 重写 `WaitHandler`：从 `DialogueCommand::Wait` 提取 `duration_secs` 设置 `WaitTimer` 资源
  - [ ] SubTask 3.10: 新增 `WaitTimer` 资源类型到 `gg-galgame-schema::resources`
  - [ ] SubTask 3.11: 更新 `CommandDispatcher::dispatch` 传递 `DialogueCommand` 引用到处理器

- [ ] Task 4: 集成打字机效果到对话系统
  - [ ] SubTask 4.1: 将 `TypewriterState` 实现 `Component` trait
  - [ ] SubTask 4.2: 在 `DialogueSystem::execute` 中，处理新节点时创建 `TypewriterState` 并插入 World
  - [ ] SubTask 4.3: 新增 `TypewriterSystem`，每帧更新 `TypewriterState`，推进 `current_position`
  - [ ] SubTask 4.4: 在 `DialogueSystem` 中检查 `TypewriterState::is_complete`，未完成时不推进到下一节点
  - [ ] SubTask 4.5: 新增 `WaitSystem`，每帧更新 `WaitTimer`，倒计时完成后移除 `WaitTimer`

- [ ] Task 5: 实现对话脚本文件加载
  - [ ] SubTask 5.1: 在 `gg-galgame-schema` 中定义 `DialogueScript` 结构（nodes、characters、variables 字段）
  - [ ] SubTask 5.2: 实现 `DialogueScript::from_json` 方法，从 JSON 字符串解析对话脚本
  - [ ] SubTask 5.3: 实现 `DialogueScriptLoader`，从文件路径加载对话脚本并创建 World 实体
  - [ ] SubTask 5.4: 在 `DialoguePlugin::build` 中注册 `DialogueScriptLoader` 资源

- [ ] Task 6: 创建 gg-plugin-tilemap 瓦片地图插件
  - [ ] SubTask 6.1: 创建 `projects/plugins/gg-plugin-tilemap` 目录和 `Cargo.toml`
  - [ ] SubTask 6.2: 定义核心组件：`Tilemap`（地图尺寸、瓦片尺寸、图层列表）、`Tile`（全局坐标、瓦片类型、碰撞标记）
  - [ ] SubTask 6.3: 定义核心资源：`Tileset`（纹理 ID、瓦片尺寸、行列数）、`TileCollisionState`（碰撞信息）
  - [ ] SubTask 6.4: 实现 `TilemapRenderSystem`：遍历瓦片实体，计算纹理坐标，提交 DrawCommand
  - [ ] SubTask 6.5: 实现 `TileCollisionSystem`：检查实体与碰撞瓦片重叠，更新碰撞状态
  - [ ] SubTask 6.6: 实现 `TilemapPlugin`：在 `build` 中注册组件、资源和系统
  - [ ] SubTask 6.7: 在根 `Cargo.toml` 中添加 `gg-plugin-tilemap` 到 workspace

- [ ] Task 7: 创建 gg-plugin-spine Spine 动画插件
  - [ ] SubTask 7.1: 创建 `projects/plugins/gg-plugin-spine` 目录和 `Cargo.toml`
  - [ ] SubTask 7.2: 定义核心组件：`SpineSkeleton`（骨骼数据引用、当前动画、混合时间）、`SpineAnimationState`（当前轨道动画、时间、循环标记）
  - [ ] SubTask 7.3: 定义核心资源：`SpineData`（骨骼定义、动画列表、附件映射）
  - [ ] SubTask 7.4: 实现 `SpineAnimationSystem`：更新动画时间，计算骨骼变换
  - [ ] SubTask 7.5: 实现 `SpineRenderSystem`：计算顶点位置，提交 DrawCommand
  - [ ] SubTask 7.6: 实现 `SpinePlugin`：在 `build` 中注册组件、资源和系统
  - [ ] SubTask 7.7: 在根 `Cargo.toml` 中添加 `gg-plugin-spine` 到 workspace

# Task Dependencies
- Task 2 depends on Task 1（现有插件适配需要新的 Plugin trait 和 PluginRegistrar）
- Task 3 depends on Task 1（命令处理器重构需要新的 Plugin 接口）
- Task 4 depends on Task 3（打字机集成需要命令处理器能正确传递数据）
- Task 5 depends on Task 4（脚本加载需要完整的对话系统）
- Task 6 depends on Task 1（瓦片地图插件需要新的 Plugin trait）
- Task 7 depends on Task 1（Spine 插件需要新的 Plugin trait）
- Task 6 和 Task 7 可并行执行
