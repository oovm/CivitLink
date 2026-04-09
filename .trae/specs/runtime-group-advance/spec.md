# 运行时组推进 Spec

## Why
按照 roadmap 运行时组的工作安排，核心运行时（25%）、渲染后端（20%）、音频后端（15%）、脚本运行时（10%）均处于早期阶段。当前 `gg-runtime-core` 的调度器仅是简单包装 `World::run_systems()`，缺乏阶段管理和系统集编排能力；音频后端完全缺失；HMR 热更新仅有编辑器侧的文件监视器，运行时侧无状态迁移机制；WASM 脚本虚拟机尚未规划。需要系统性地推进这些模块，使运行时组从"基础骨架"演进为"可工作的运行时框架"。

## What Changes
- 重构 `gg-runtime-core` 中的调度器：引入阶段（Stage）驱动的系统调度，支持 Startup/PreUpdate/Update/PostUpdate/Render/Exit 标准阶段，支持系统集（SystemSet）注册和阶段间依赖排序
- 新增 `gg-runtime-audio` crate：定义音频 HAL（`AudioEngine` trait、`SoundId`、`AudioCommand`），基于 `cpal` + `rodio` 实现桌面端音频后端
- 在 `gg-runtime-core` 中实现 HMR 热更新基础架构：运行时侧的脚本热替换和资源热替换机制，支持状态序列化与重载
- 在 `gg-runtime-core` 中规划 WASM 脚本虚拟机集成：定义 `WasmRuntime` trait 和沙箱接口，为后续 wasmtime/wasmer 集成奠定基础
- **BREAKING** 重构 `gg-runtime-core::Runtime` 的公共 API：`Runtime` 将不再直接持有 `RenderSystem`，改为通过 `Renderer` trait 动态注入渲染后端

## Impact
- Affected specs: 运行时层（Runtime Layer）、音频子系统、HMR 热更新、脚本沙箱
- Affected code: `gg-runtime-core`（重构调度器和 Runtime 结构）、新增 `gg-runtime-audio`、`gg-ecs`（Scheduler 拆分到 runtime-core）
- 与已有 spec 的关系：
  - `wgpu-render-ui` 已完成 Task 1（渲染 HAL），其 Task 2-5（WGPU 实现）独立推进，本 spec 不重复
  - `valkyrie-script-integration` 已全部完成，本 spec 在其基础上扩展 HMR 和 WASM 集成

## ADDED Requirements

### Requirement: 阶段驱动的系统调度器
系统 SHALL 在 `gg-runtime-core` 中实现阶段驱动的系统调度器，替代当前简单的 `Scheduler::tick()` 调用。

#### Scenario: 标准阶段定义
- **WHEN** 运行时创建调度器
- **THEN** 调度器提供标准阶段：`Startup`、`PreUpdate`、`Update`、`PostUpdate`、`Render`、`Exit`
- **AND** 各阶段按固定顺序执行：Startup → PreUpdate → Update → PostUpdate → Render，Exit 在停止时执行

#### Scenario: 系统注册到指定阶段
- **WHEN** 插件或游戏代码注册系统到调度器
- **THEN** 系统可指定注册到某个阶段（如 `Update`）
- **AND** 系统可指定属于某个系统集（SystemSet）
- **AND** 系统可声明与其他系统的执行顺序（before/after）

#### Scenario: 阶段执行
- **WHEN** 调度器执行一帧（tick）
- **THEN** 按阶段顺序依次执行各阶段中的系统
- **AND** 同一阶段内的系统按注册顺序和依赖关系执行
- **AND** Startup 阶段仅在首次 tick 时执行一次

#### Scenario: 插件注册系统
- **WHEN** 插件通过 `Plugin::build` 注册系统
- **THEN** 插件可将系统注册到调度器的任意阶段
- **AND** 多个插件注册的系统在调度器中统一编排

### Requirement: 音频硬件抽象层
系统 SHALL 在 `gg-runtime-audio` 中定义音频硬件抽象层，使上层代码与具体音频库解耦。

#### Scenario: 定义 AudioEngine trait
- **WHEN** 系统初始化音频模块
- **THEN** `gg-runtime-audio` 提供 `AudioEngine` trait，包含方法：`play`、`stop`、`set_volume`、`pause`、`resume`、`load_sound`
- **AND** `AudioEngine` trait 不依赖任何具体音频库

#### Scenario: 定义音频命令
- **WHEN** 游戏逻辑需要控制音频播放
- **THEN** 系统提供 `AudioCommand` 枚举，包含变体：`Play`、`Stop`、`SetVolume`、`Pause`、`Resume`
- **AND** 每个 `AudioCommand` 携带完整的参数（声音 ID、音量、是否循环等）

#### Scenario: 定义声音标识
- **WHEN** 系统需要引用已加载的声音资源
- **THEN** `gg-runtime-audio` 提供 `SoundId` 标识已加载声音
- **AND** 提供 `SoundDescriptor` 描述声音元数据（格式、时长、声道数）

### Requirement: 桌面端音频后端
系统 SHALL 提供 `CpalAudioEngine`，基于 `cpal` + `rodio` 实现 `AudioEngine` trait。

#### Scenario: 初始化音频设备
- **WHEN** 应用启动并请求创建音频引擎
- **THEN** `CpalAudioEngine` 初始化 cpal 音频输出设备
- **AND** 创建 rodio 音频输出流

#### Scenario: 加载音频文件
- **WHEN** 游戏请求加载音频文件（WAV/OGG/MP3）
- **THEN** `CpalAudioEngine` 解码音频数据并缓存
- **AND** 返回 `SoundId` 供后续播放引用

#### Scenario: 播放控制
- **WHEN** 游戏提交 `AudioCommand::Play` 指令
- **THEN** `CpalAudioEngine` 播放指定声音
- **AND** 支持音量控制、循环播放、暂停/恢复

### Requirement: HMR 热更新基础架构
系统 SHALL 在 `gg-runtime-core` 中实现 HMR 热更新的运行时侧支持，包括脚本热替换和资源热替换。

#### Scenario: 脚本热替换
- **WHEN** HMR 管线检测到脚本文件变更并推送新字节码到运行时
- **THEN** 运行时将当前脚本模块替换为新版本
- **AND** 正在执行的函数完成当前帧后再切换
- **AND** 全局变量状态通过序列化/反序列化保留

#### Scenario: 资源热替换
- **WHEN** HMR 管线检测到资源文件变更并推送新资源到运行时
- **THEN** 运行时将旧资源替换为新资源
- **AND** 已加载的纹理/音频句柄自动指向新资源
- **AND** 引用该资源的实体/组件在下一帧使用新资源渲染

#### Scenario: HMR 状态迁移
- **WHEN** 脚本热替换需要迁移游戏状态
- **THEN** 运行时调用旧脚本的 `on_hot_reload_out` 钩子序列化状态
- **AND** 调用新脚本的 `on_hot_reload_in` 钩子反序列化状态
- **AND** 状态迁移失败时回滚到旧脚本并报告错误

### Requirement: WASM 脚本虚拟机集成规划
系统 SHALL 在 `gg-runtime-core` 中定义 WASM 脚本虚拟机的集成接口，为后续 wasmtime/wasmer 集成奠定基础。

#### Scenario: 定义 WasmRuntime trait
- **WHEN** 系统需要集成 WASM 脚本运行时
- **THEN** `gg-runtime-core` 提供 `WasmRuntime` trait，定义 WASM 模块加载、实例化、函数调用的标准接口
- **AND** `WasmRuntime` trait 与具体 WASM 引擎（wasmtime/wasmer）解耦

#### Scenario: 定义沙箱接口
- **WHEN** WASM 脚本需要访问引擎 API
- **THEN** 系统提供 `WasmHostFunctions` trait，定义 WASM 模块可导入的宿主函数集合
- **AND** 沙箱限制脚本只能通过 `WasmHostFunctions` 访问引擎功能
- **AND** 沙箱限制脚本的内存和执行时间

#### Scenario: WASM 与 ECS 交互
- **WHEN** WASM 脚本需要操作 ECS 世界
- **THEN** 通过 `WasmHostFunctions` 暴露 spawn_entity、add_component、get_component、set_component 等 API
- **AND** 这些 API 与现有 `Host` trait 的语义一致

### Requirement: Runtime 结构重构
系统 SHALL 重构 `gg-runtime-core::Runtime`，使其支持动态注入渲染后端和音频后端。

#### Scenario: 动态注入渲染后端
- **WHEN** 创建 Runtime 实例
- **THEN** Runtime 接受实现了 `Renderer` trait 的渲染后端实例
- **AND** Runtime 不再直接持有 `RenderSystem`，改为持有 `Box<dyn Renderer>`

#### Scenario: 动态注入音频后端
- **WHEN** 创建 Runtime 实例
- **THEN** Runtime 接受实现了 `AudioEngine` trait 的音频后端实例
- **AND** Runtime 持有 `Option<Box<dyn AudioEngine>>`，音频后端可选

#### Scenario: 集成阶段调度器
- **WHEN** Runtime 执行游戏循环
- **THEN** Runtime 使用阶段调度器替代当前的 `Scheduler::tick()`
- **AND** 每帧按阶段顺序执行：PreUpdate → Update → PostUpdate → Render

#### Scenario: 集成 HMR 支持
- **WHEN** Runtime 启用 HMR 模式
- **THEN** Runtime 持有 `HmrManager`，监听脚本和资源的变更通知
- **AND** 变更到达时执行热替换流程

## MODIFIED Requirements

### Requirement: gg-runtime-core Runtime 结构
`Runtime` SHALL 从直接持有 `RenderSystem` 和 `Scheduler` 改为持有 `Box<dyn Renderer>`、`Option<Box<dyn AudioEngine>>`、`StageScheduler`、`Option<HmrManager>`。原有的 `render_system()` 和 `scheduler()` 访问器将被替换为 `renderer()`、`audio_engine()`、`stage_scheduler()` 访问器。

### Requirement: gg-runtime-core EngineHost
`EngineHost` SHALL 移除硬编码的 `Position`/`Velocity` 组件类型，改为通过反射系统或组件注册表动态处理组件操作。

### Requirement: gg-ecs Scheduler
`gg-ecs::Scheduler` SHALL 保留为轻量级的世界包装器，但运行时调度逻辑迁移到 `gg-runtime-core::StageScheduler`。`gg-ecs::Scheduler` 仅作为 `World` 的便捷访问器。

## REMOVED Requirements

### Requirement: Runtime 直接持有 RenderSystem
**Reason**: 渲染后端应通过 `Renderer` trait 动态注入，而非硬编码 `RenderSystem`
**Migration**: 使用 `RuntimeBuilder::renderer()` 方法注入渲染后端实例

### Requirement: EngineHost 硬编码组件类型
**Reason**: 硬编码 Position/Velocity 组件类型不利于扩展，应通过组件注册表动态处理
**Migration**: 使用组件注册表（ComponentRegistry）注册组件类型，EngineHost 通过注册表查找和操作组件
