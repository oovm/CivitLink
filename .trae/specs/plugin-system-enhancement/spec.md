# 插件系统增强与官方插件推进 Spec

## Why
当前 `gg-core::plugin::Plugin` trait 仅包含 `name()`、`initialize()`、`shutdown()` 三个方法，无法向 World 注册组件、资源和系统，导致所有现有插件（dialogue、portrait、save、scene-transition）的 `initialize()` 均为空实现。对话系统插件虽有基础骨架，但命令处理器无法接收实际命令数据、打字机效果未集成、对话脚本无法从文件加载。瓦片地图和 Spine 动画插件尚未创建。按照 roadmap 插件组本月工作重点，需要增强插件系统通用接口、完善对话系统、规划瓦片地图和 Spine 动画插件。

## What Changes
- 增强 `gg-core::plugin::Plugin` trait：支持向 World 注册组件、资源、系统，支持插件依赖声明
- 新增 `PluginRegistrar` 结构：在 `initialize` 阶段收集插件注册信息，统一应用到 World
- 重构 `gg-plugin-dialogue`：命令处理器接收实际命令数据、集成打字机效果、支持对话脚本文件加载
- 新增 `gg-plugin-tilemap`：瓦片地图插件骨架，定义核心类型和基础系统
- 新增 `gg-plugin-spine`：Spine 动画插件骨架，定义核心类型和数据结构
- **BREAKING** `Plugin` trait 增加 `build` 方法，所有现有插件需适配新接口

## Impact
- Affected specs: 插件系统架构、对话系统、瓦片地图、Spine 动画
- Affected code: `gg-core`（Plugin trait 重构）、`gg-plugin-dialogue`（重构）、`gg-plugin-portrait`（适配）、`gg-plugin-save`（适配）、`gg-plugin-scene-transition`（适配）、新增 `gg-plugin-tilemap`、新增 `gg-plugin-spine`

## ADDED Requirements

### Requirement: 增强的 Plugin trait
系统 SHALL 提供增强的 `Plugin` trait，支持插件向 World 注册组件、资源和系统。

#### Scenario: 插件注册系统和资源
- **WHEN** 插件的 `build` 方法被调用
- **THEN** 插件通过 `PluginRegistrar` 注册所需的系统、资源和初始实体
- **AND** `PluginRegistrar` 将注册信息收集后统一应用到 World

#### Scenario: 插件依赖声明
- **WHEN** 插件声明对其他插件的依赖
- **THEN** 引擎在初始化时检查依赖是否满足
- **AND** 依赖插件先于当前插件初始化

#### Scenario: 插件生命周期
- **WHEN** 引擎启动
- **THEN** 按 `Plugin::build` → `Plugin::initialize` → 运行 → `Plugin::shutdown` 顺序管理插件生命周期
- **AND** `build` 阶段仅收集注册信息，不修改 World

### Requirement: PluginRegistrar 注册器
系统 SHALL 提供 `PluginRegistrar` 结构，在插件 `build` 阶段收集注册信息。

#### Scenario: 注册系统
- **WHEN** 插件调用 `registrar.register_system(system)`
- **THEN** 系统被添加到待注册队列
- **AND** 在 `apply` 阶段统一注册到 World

#### Scenario: 注册资源
- **WHEN** 插件调用 `registrar.insert_resource(resource)`
- **THEN** 资源被添加到待注册队列
- **AND** 在 `apply` 阶段统一插入到 World

#### Scenario: 声明依赖
- **WHEN** 插件调用 `registrar.add_dependency("other_plugin")`
- **THEN** 依赖关系被记录
- **AND** 在初始化时验证依赖是否满足

### Requirement: 对话系统命令数据传递
系统 SHALL 使命令处理器接收实际的 `DialogueCommand` 数据，而非仅操作 World。

#### Scenario: 命令处理器接收命令数据
- **WHEN** `CommandDispatcher::dispatch` 被调用
- **THEN** 对应的 `CommandHandler::execute` 接收 `DialogueCommand` 引用和 `World` 可变引用
- **AND** 处理器根据命令数据执行具体操作

#### Scenario: PlayBgm 命令处理
- **WHEN** `PlayBgm { asset_path, volume, fade_in_secs }` 命令被分发
- **THEN** 处理器将 `asset_path`、`volume`、`fade_in_secs` 写入 `AudioControl` 组件

#### Scenario: ShowPortrait 命令处理
- **WHEN** `ShowPortrait { character_id, expression, position, transition }` 命令被分发
- **THEN** 处理器创建 `PortraitState` 实体，设置 `character_id`、`current_expression`、`position`

#### Scenario: SetVariable 命令处理
- **WHEN** `SetVariable { name, value }` 命令被分发
- **THEN** 处理器将 `name` 和 `value` 写入 `GameVariables` 资源

#### Scenario: Wait 命令处理
- **WHEN** `Wait { duration_secs }` 命令被分发
- **THEN** 处理器设置等待计时器资源 `WaitTimer`

### Requirement: 打字机效果集成
系统 SHALL 将 `TypewriterState` 集成到对话系统中，实现逐字显示对话文本。

#### Scenario: 对话节点开始时创建打字机状态
- **WHEN** `DialogueSystem` 处理新的对话节点
- **THEN** 使用节点文本创建 `TypewriterState` 并插入 World

#### Scenario: 打字机效果逐帧更新
- **WHEN** 每帧执行打字机系统
- **THEN** 根据帧间隔时间更新 `TypewriterState` 的 `current_position`
- **AND** 打字机完成前阻止对话推进

#### Scenario: 跳过打字机效果
- **WHEN** 玩家触发跳过操作
- **THEN** 调用 `TypewriterState::skip()` 立即显示全部文本
- **AND** 允许对话推进到下一节点

### Requirement: 对话脚本文件加载
系统 SHALL 支持从文件加载对话脚本数据，初始化对话节点和角色定义。

#### Scenario: 加载对话脚本
- **WHEN** 引擎加载 `.json` 格式的对话脚本文件
- **THEN** 解析文件内容为 `DialogueNode` 和 `CharacterDef` 实体
- **AND** 将实体注册到 World 中

#### Scenario: 对话脚本格式
- **WHEN** 对话脚本文件包含 `nodes`、`characters`、`variables` 字段
- **THEN** `nodes` 解析为 `DialogueNode` 实体列表
- **AND** `characters` 解析为 `CharacterDef` 实体列表
- **AND** `variables` 解析为 `GameVariables` 初始值

### Requirement: 瓦片地图插件
系统 SHALL 提供 `gg-plugin-tilemap` 插件，定义瓦片地图的核心类型和基础系统。

#### Scenario: 瓦片地图核心类型
- **WHEN** 插件初始化
- **THEN** 定义 `Tilemap` 组件（地图尺寸、瓦片尺寸、图层列表）
- **AND** 定义 `Tile` 组件（全局坐标、瓦片类型、碰撞标记）
- **AND** 定义 `Tileset` 资源（纹理 ID、瓦片尺寸、行列数）

#### Scenario: 瓦片地图渲染系统
- **WHEN** `TilemapRenderSystem` 执行
- **THEN** 遍历所有 `Tilemap` 和 `Tile` 实体
- **AND** 根据 `Tileset` 资源计算瓦片在纹理图集中的位置
- **AND** 提交 `DrawCommand::Sprite` 渲染指令

#### Scenario: 瓦片地图碰撞系统
- **WHEN** `TileCollisionSystem` 执行
- **THEN** 检查实体与碰撞瓦片的重叠
- **AND** 将碰撞信息写入 `TileCollisionState` 资源

### Requirement: Spine 动画插件
系统 SHALL 提供 `gg-plugin-spine` 插件，定义 Spine 骨骼动画的核心类型和数据结构。

#### Scenario: Spine 核心类型
- **WHEN** 插件初始化
- **THEN** 定义 `SpineSkeleton` 组件（骨骼数据引用、当前动画、混合时间）
- **AND** 定义 `SpineAnimationState` 组件（当前轨道动画、时间、循环标记）
- **AND** 定义 `SpineData` 资源（骨骼定义、动画列表、附件映射）

#### Scenario: Spine 动画更新系统
- **WHEN** `SpineAnimationSystem` 执行
- **THEN** 更新所有 `SpineAnimationState` 的动画时间
- **AND** 计算骨骼变换矩阵
- **AND** 更新 `SpineSkeleton` 的世界变换

#### Scenario: Spine 渲染系统
- **WHEN** `SpineRenderSystem` 执行
- **THEN** 根据 `SpineSkeleton` 和 `SpineData` 计算顶点位置
- **AND** 提交 `DrawCommand::Sprite` 渲染指令（按插槽顺序）

## MODIFIED Requirements

### Requirement: Plugin trait
`Plugin` trait SHALL 增加 `build` 方法用于通过 `PluginRegistrar` 注册组件、资源和系统，以及 `dependencies` 方法声明插件依赖。原有的 `initialize` 和 `shutdown` 方法保留不变。

### Requirement: 现有插件适配
所有现有插件（`DialoguePlugin`、`PortraitPlugin`、`SavePlugin`、`SceneTransitionPlugin`）SHALL 实现新的 `Plugin` trait 接口，在 `build` 方法中注册各自的系统和资源。

## REMOVED Requirements

### Requirement: 命令处理器不接收命令数据
**Reason**: 当前 `CommandHandler::execute` 仅接收 `&mut World`，无法获取具体命令数据，导致所有处理器都是占位实现
**Migration**: 修改 `CommandHandler::execute` 签名为 `execute(&self, command: &DialogueCommand, world: &mut World)`
