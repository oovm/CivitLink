# Tasks

- [x] Task 1: 重构 gg-ecs — 将 World 重命名为 GgWorld，增加 Resource 管理和 EntityBuilder
  - [x] SubTask 1.1: 将 `World` 重命名为 `GgWorld`，移除 `Scheduler` 类型
  - [x] SubTask 1.2: 实现 `Resource` 管理（`insert_resource`、`get_resource`、`get_resource_mut`）
  - [x] SubTask 1.3: 实现 `EntityBuilder` 构建器模式（`spawn()` → `EntityBuilder` → `.insert()` → `.id()`）
  - [x] SubTask 1.4: 实现 `Query` 查询系统（基本组件查询、`With`/`Without` 过滤器）
  - [x] SubTask 1.5: 更新 `gg-ecs/Cargo.toml` 依赖（如需要）
  - [x] SubTask 1.6: 为所有新增 public 项添加文档注释

- [x] Task 2: 重构 gg-asset — 引入 AssetServer、Handle<T>、AssetCache、AssetError
  - [x] SubTask 2.1: 实现 `AssetError` 枚举（NotFound、LoadError、TypeMismatch）
  - [x] SubTask 2.2: 实现 `Handle<T>` 类型安全句柄（id、path）
  - [x] SubTask 2.3: 实现 `AssetCache`（基于 HashMap 的并发安全缓存，insert/get/get_handle/contains/remove/clear）
  - [x] SubTask 2.4: 实现 `AssetServer`（new、add_asset、cache 方法）
  - [x] SubTask 2.5: 重构 `AssetLoader` trait 支持异步接口（`async fn load`）
  - [x] SubTask 2.6: 保留 `TextAsset`/`BinaryAsset` 及其加载器，适配新 API
  - [x] SubTask 2.7: 为所有新增 public 项添加文档注释

- [x] Task 3: 新建 gg-schedule crate — 实现调度标签、系统集、Schedule
  - [x] SubTask 3.1: 创建 `projects/core/gg-schedule/` 目录和 `Cargo.toml`
  - [x] SubTask 3.2: 实现 `ScheduleLabel` trait 和标准标签（Startup、Update、FixedUpdate、PostUpdate、Render、Exit）
  - [x] SubTask 3.3: 实现 `SystemSet` trait 和 `CoreSet` 枚举（Startup、First、PreUpdate、Update、PostUpdate、Last）
  - [x] SubTask 3.4: 实现 `Schedule` 结构体（new、add_systems、in_set、before、after、run）
  - [x] SubTask 3.5: 实现 `System` trait 适配（函数系统包装）
  - [x] SubTask 3.6: 为所有新增 public 项添加文档注释

- [x] Task 4: 新建 gg-reflection crate — 实现反射注册表和属性编辑器
  - [x] SubTask 4.1: 创建 `projects/core/gg-reflection/` 目录和 `Cargo.toml`
  - [x] SubTask 4.2: 实现 `ReflectionRegistry`（new、register、get、get_mut、get_type_info、get_registration、is_registered）
  - [x] SubTask 4.3: 实现 `PropertyInfo` 结构体（name、type_name、writable、description）
  - [x] SubTask 4.4: 实现 `PropertyEditor` trait（editable_properties、get_property、set_property）
  - [x] SubTask 4.5: 实现 `StructPropertyEditor<T>`（new、get、get_mut、into_inner）
  - [x] SubTask 4.6: 为所有新增 public 项添加文档注释

- [x] Task 5: 新建 gg-world crate — 实现 GameWorld 和 WorldManager
  - [x] SubTask 5.1: 创建 `projects/core/gg-world/` 目录和 `Cargo.toml`
  - [x] SubTask 5.2: 实现 `GameWorld`（new、name、is_destroyed、destroy、clear、spawn、entity、entity_mut、despawn、insert_resource、get_resource、get_resource_mut、reflection_registry 字段）
  - [x] SubTask 5.3: 实现 `WorldManager`（new、create_world、destroy_world、get_world、get_world_mut、active_world、active_world_mut、set_active_world、world_ids、destroy_all_worlds）
  - [x] SubTask 5.4: 为所有新增 public 项添加文档注释

- [x] Task 6: 更新 workspace 配置和依赖关系
  - [x] SubTask 6.1: 在根 `Cargo.toml` 的 workspace members 和 dependencies 中添加 gg-schedule、gg-world、gg-reflection
  - [x] SubTask 6.2: 更新 `gg-runtime-core` 的依赖和代码，适配新 ECS/调度器 API
  - [x] SubTask 6.3: 确保 `cargo check` 通过

# Task Dependencies
- [Task 3] depends on [Task 1] (gg-schedule 的 Schedule 需要 GgWorld 类型)
- [Task 4] depends on nothing (gg-reflection 独立)
- [Task 5] depends on [Task 1], [Task 2], [Task 4] (GameWorld 整合 GgWorld + AssetServer + ReflectionRegistry)
- [Task 6] depends on [Task 1], [Task 2], [Task 3], [Task 4], [Task 5]
