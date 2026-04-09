# 核心基础组本月推进 Spec

## Why

GG 元游戏引擎的核心基础模块（ECS、资源系统、调度器、世界管理、反射系统）当前仅有骨架实现，与设计文档中定义的 GgWorld、GgSchedule、AssetServer、GameWorld、WorldManager、ReflectionRegistry 等核心类型差距较大。需要按照 roadmap 本月工作重点推进，使核心基础模块达到可用状态，为上层运行时、编辑器和插件系统提供坚实基础。

## What Changes

- 重构 `gg-ecs`：将现有 `World` 重命名为 `GgWorld`，增加 `Resource` 管理、`Query` 查询系统、实体构建器模式（`spawn().insert().id()`）
- 重构 `gg-asset`：引入 `AssetServer`、`AssetCache`（并发安全）、`Handle<T>` 类型安全句柄、`AssetError` 错误枚举，支持异步加载接口
- 新建 `gg-schedule` crate：实现 `ScheduleLabel`、标准调度标签（Startup/Update/FixedUpdate/PostUpdate/Render/Exit）、`SystemSet`（CoreSet）、`Schedule` 阶段管理、系统排序（before/after）
- 新建 `gg-world` crate：实现 `GameWorld`（整合 GgWorld + AssetServer + ReflectionRegistry）、`WorldManager`（多世界管理、活动世界切换）
- 新建 `gg-reflection` crate：实现 `ReflectionRegistry`（类型注册表）、`PropertyInfo`（属性信息）、`PropertyEditor` trait、`StructPropertyEditor<T>`

## Impact

- Affected specs: gg-ecs API（GgWorld 替代 World）、gg-asset API（AssetServer 替代 AssetManager）、gg-runtime-core（需适配新 ECS/调度器 API）
- Affected code:
  - `projects/core/gg-ecs/src/lib.rs` — 重构为 GgWorld + Query + Resource
  - `projects/core/gg-asset/src/lib.rs` — 重构为 AssetServer + Handle + AssetCache
  - `projects/core/gg-schedule/` — 新建 crate
  - `projects/core/gg-world/` — 新建 crate
  - `projects/core/gg-reflection/` — 新建 crate
  - `Cargo.toml` — workspace members 增加 3 个新 crate
  - `projects/runtime/gg-runtime-core/` — 适配新 API

## ADDED Requirements

### Requirement: GgWorld 实体重体系统

系统 SHALL 提供 `GgWorld` 作为 ECS 世界的核心容器，管理实体、组件和资源。

#### Scenario: 创建 GgWorld 并生成实体
- **WHEN** 调用 `GgWorld::new()` 创建世界
- **THEN** 返回一个空的 GgWorld 实例
- **WHEN** 调用 `world.spawn()` 生成实体
- **THEN** 返回 `EntityBuilder`，支持链式调用 `.insert(component)` 插入组件
- **AND** `.id()` 返回生成的实体 ID

#### Scenario: 实体组件管理
- **WHEN** 对实体调用 `insert_component<T>` / `remove_component<T>`
- **THEN** 组件正确添加到或从实体的组件存储中移除
- **AND** `get_component<T>` / `get_component_mut<T>` 可正确获取组件引用

#### Scenario: 全局资源管理
- **WHEN** 调用 `world.insert_resource(resource)` 插入全局资源
- **THEN** 资源按类型存储，可通过 `get_resource<T>` / `get_resource_mut<T>` 获取
- **AND** 同类型资源插入时覆盖旧值

### Requirement: Query 查询系统

系统 SHALL 提供 `Query` 类型，支持按组件类型查询实体集合。

#### Scenario: 基本组件查询
- **WHEN** 使用 `Query<(&Position, &Velocity)>` 查询
- **THEN** 返回所有同时拥有 Position 和 Velocity 组件的实体引用
- **AND** 支持可变引用 `Query<&mut Position>`

#### Scenario: 查询过滤
- **WHEN** 使用 `Query<&Position, With<Player>>` 查询
- **THEN** 仅返回拥有 Player 组件的实体的 Position
- **WHEN** 使用 `Query<&Position, Without<Enemy>>` 查询
- **THEN** 排除拥有 Enemy 组件的实体

### Requirement: AssetServer 异步资源管理

系统 SHALL 提供 `AssetServer` 作为资源加载和管理的核心，支持异步加载和并发安全缓存。

#### Scenario: 注册和加载资源
- **WHEN** 调用 `asset_server.add_asset(path, asset)` 添加资源
- **THEN** 返回类型安全的 `Handle<T>` 句柄
- **AND** 资源被存入 `AssetCache`

#### Scenario: 通过句柄获取资源
- **WHEN** 调用 `asset_server.cache().get(&handle)` 获取资源
- **THEN** 返回 `Option<Arc<T>>`，若资源存在则返回引用

#### Scenario: 异步加载接口
- **WHEN** `AssetLoader` trait 定义 `async fn load(&self, path: &Path) -> Result<T, AssetError>`
- **THEN** 加载器可在异步运行时中执行资源加载
- **AND** 加载完成后资源自动进入缓存

#### Scenario: 资源缓存管理
- **WHEN** 调用 `cache.contains(path)` / `cache.remove(path)` / `cache.clear()`
- **THEN** 缓存正确反映操作结果

### Requirement: Handle<T> 类型安全句柄

系统 SHALL 提供 `Handle<T>` 作为资源的类型安全引用。

#### Scenario: 创建和使用句柄
- **WHEN** 通过 `Handle::new(id, path)` 创建句柄
- **THEN** 句柄携带资源 ID 和路径信息
- **AND** `handle.id()` 返回资源 ID，`handle.path()` 返回资源路径

### Requirement: AssetError 资源错误枚举

系统 SHALL 提供 `AssetError` 枚举，统一资源加载错误类型。

#### Scenario: 错误分类
- **WHEN** 资源未找到时返回 `AssetError::NotFound(String)`
- **WHEN** 加载失败时返回 `AssetError::LoadError(String)`
- **WHEN** 类型不匹配时返回 `AssetError::TypeMismatch`

### Requirement: Schedule 调度器

系统 SHALL 提供 `gg-schedule` crate，实现系统调度和阶段管理。

#### Scenario: 标准调度标签
- **WHEN** 使用 `Startup`、`Update`、`FixedUpdate`、`PostUpdate`、`Render`、`Exit` 标签
- **THEN** 每个标签标识一个标准执行阶段
- **AND** 标签实现 `ScheduleLabel` trait，支持 Clone、Debug、PartialEq、Eq、Hash

#### Scenario: Schedule 创建和系统添加
- **WHEN** 调用 `Schedule::new(Update)` 创建调度器
- **THEN** 返回绑定到 Update 阶段的空调度器
- **WHEN** 调用 `schedule.add_systems(system_fn)` 添加系统
- **THEN** 系统被注册到该调度器中

#### Scenario: 系统集组织
- **WHEN** 调用 `schedule.add_systems(systems).in_set(CoreSet::Update)`
- **THEN** 系统被归入 CoreSet::Update 系统集
- **AND** CoreSet 枚举包含 Startup、First、PreUpdate、Update、PostUpdate、Last 变体

#### Scenario: 系统排序
- **WHEN** 调用 `.after(system_a)` 或 `.before(system_b)` 配置系统顺序
- **THEN** 调度器按指定顺序执行系统

#### Scenario: 调度器运行
- **WHEN** 调用 `schedule.run(&mut world)`
- **THEN** 按阶段和排序依次执行所有注册的系统

### Requirement: GameWorld 游戏世界

系统 SHALL 提供 `GameWorld`，整合 ECS 世界、资源服务器和反射注册表。

#### Scenario: 创建和操作游戏世界
- **WHEN** 调用 `GameWorld::new(name)` 创建世界
- **THEN** 返回包含 GgWorld、AssetServer、ReflectionRegistry 的整合实例
- **WHEN** 调用 `world.spawn().insert(component).id()` 生成实体
- **THEN** 实体和组件被添加到内部 GgWorld

#### Scenario: 资源和反射集成
- **WHEN** 调用 `world.insert_resource(resource)` 插入资源
- **THEN** 资源被添加到内部 GgWorld 的资源管理器
- **WHEN** 调用 `world.reflection_registry.register::<T>()` 注册类型
- **THEN** 类型信息被注册到内部 ReflectionRegistry

#### Scenario: 世界生命周期
- **WHEN** 调用 `world.destroy()` 销毁世界
- **THEN** `is_destroyed()` 返回 true
- **WHEN** 调用 `world.clear()` 清空世界
- **THEN** 所有实体和资源被移除

### Requirement: WorldManager 多世界管理

系统 SHALL 提供 `WorldManager`，管理多个游戏世界的创建、销毁和切换。

#### Scenario: 创建和销毁世界
- **WHEN** 调用 `manager.create_world(name)` 创建世界
- **THEN** 返回世界 ID，世界被添加到管理器
- **WHEN** 调用 `manager.destroy_world(id)` 销毁世界
- **THEN** 指定 ID 的世界被移除

#### Scenario: 活动世界切换
- **WHEN** 调用 `manager.set_active_world(id)` 设置活动世界
- **THEN** `manager.active_world()` / `active_world_mut()` 返回指定世界
- **AND** 同一时间只有一个活动世界

#### Scenario: 世界列表查询
- **WHEN** 调用 `manager.world_ids()` 获取世界 ID 列表
- **THEN** 返回所有未销毁世界的 ID

### Requirement: ReflectionRegistry 反射注册表

系统 SHALL 提供 `ReflectionRegistry`，管理可反射类型的运行时信息。

#### Scenario: 注册和查询类型
- **WHEN** 调用 `registry.register::<T>()` 注册类型
- **THEN** `registry.is_registered(type_id)` 返回 true
- **WHEN** 调用 `registry.get_type_info(type_id)` 获取类型信息
- **THEN** 返回该类型的注册信息

### Requirement: PropertyInfo 属性信息

系统 SHALL 提供 `PropertyInfo` 结构体，描述可反射类型的属性元数据。

#### Scenario: 创建属性信息
- **WHEN** 调用 `PropertyInfo::new(name, type_name, writable)` 创建属性信息
- **THEN** 属性信息包含名称、类型名、可写标志
- **WHEN** 调用 `.with_description(desc)` 设置描述
- **THEN** 属性信息包含可选的描述文本

### Requirement: PropertyEditor 属性编辑器

系统 SHALL 提供 `PropertyEditor` trait，支持动态编辑可反射类型的属性。

#### Scenario: 获取和设置属性
- **WHEN** 调用 `editor.editable_properties()` 获取可编辑属性列表
- **THEN** 返回 `Vec<PropertyInfo>`
- **WHEN** 调用 `editor.get_property(name)` 获取属性值
- **THEN** 返回 `Option<&dyn PartialReflect>`
- **WHEN** 调用 `editor.set_property(name, value)` 设置属性值
- **THEN** 属性值被更新，成功时返回 Ok(())

### Requirement: StructPropertyEditor 结构体属性编辑器

系统 SHALL 提供 `StructPropertyEditor<T>` 泛型结构体，为结构体类型实现属性编辑。

#### Scenario: 创建和使用结构体编辑器
- **WHEN** 调用 `StructPropertyEditor::new(value)` 创建编辑器
- **THEN** 编辑器持有该值的内部可变引用
- **WHEN** 调用 `editor.get()` / `editor.get_mut()` 获取内部值
- **THEN** 返回内部值的不可变/可变引用
- **WHEN** 调用 `editor.into_inner()` 消费编辑器
- **THEN** 返回内部值

## MODIFIED Requirements

### Requirement: gg-ecs 公共 API 重构

现有 `World` 类型 SHALL 重命名为 `GgWorld`，`Scheduler` 类型 SHALL 被移除（调度功能由 `gg-schedule` 提供），`System` trait 的 `execute` 方法签名 SHALL 接受 `&mut GgWorld`。

#### Scenario: API 兼容性迁移
- **WHEN** 现有代码使用 `World::new()` 创建世界
- **THEN** SHALL 改用 `GgWorld::new()`
- **WHEN** 现有代码使用 `Scheduler` 执行系统
- **THEN** SHALL 改用 `gg-schedule` 的 `Schedule`

### Requirement: gg-asset 公共 API 重构

现有 `AssetManager` SHALL 重构为 `AssetServer`，`AssetHandle<T>` (Arc<T>) SHALL 替换为 `Handle<T>` 类型安全句柄，`AssetLoader` trait SHALL 支持异步加载。

#### Scenario: API 兼容性迁移
- **WHEN** 现有代码使用 `AssetManager::new()` 创建管理器
- **THEN** SHALL 改用 `AssetServer::new()`
- **WHEN** 现有代码使用 `AssetManager::load()` 同步加载
- **THEN** SHALL 改用 `AssetServer::add_asset()` 直接添加或异步加载接口

## REMOVED Requirements

### Requirement: gg-ecs 中的 Scheduler 类型
**Reason**: 调度功能由独立的 `gg-schedule` crate 提供，ECS 核心不应包含调度逻辑
**Migration**: 使用 `gg-schedule::Schedule` 替代 `gg-ecs::Scheduler`

### Requirement: gg-asset 中的 AssetManager 类型
**Reason**: 重构为 `AssetServer`，提供更完整的资源管理能力
**Migration**: 使用 `gg-asset::AssetServer` 替代 `gg-asset::AssetManager`
