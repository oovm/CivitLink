# Tasks

- [x] Task 1: 实现阶段驱动的系统调度器（StageScheduler）
  - [x] SubTask 1.1: 定义 `Stage` 枚举：`Startup`、`PreUpdate`、`Update`、`PostUpdate`、`Render`、`Exit`
  - [x] SubTask 1.2: 定义 `SystemSet` trait 和 `SystemSetId` 类型，支持系统集注册和标识
  - [x] SubTask 1.3: 定义 `SystemDescriptor` 结构，包含系统函数指针、所属阶段、所属系统集、执行顺序约束（before/after）
  - [x] SubTask 1.4: 实现 `StageScheduler` 结构：按阶段分组存储系统，支持 `add_system`、`add_system_to_stage`、`configure_set` 方法
  - [x] SubTask 1.5: 实现 `StageScheduler::tick`：按阶段顺序执行系统，Startup 阶段仅执行一次
  - [x] SubTask 1.6: 实现 `Plugin::build` 接口扩展：插件可通过 `App` 注册系统到指定阶段

- [x] Task 2: 创建 gg-runtime-audio crate，定义音频 HAL
  - [x] SubTask 2.1: 创建 `projects/runtime/gg-runtime-audio` 目录和 `Cargo.toml`，依赖 gg-core
  - [x] SubTask 2.2: 定义 `SoundId` 类型（newtype wrapper around u64）和 `SoundDescriptor` 结构（格式、时长、声道数）
  - [x] SubTask 2.3: 定义 `AudioCommand` 枚举：`Play`、`Stop`、`SetVolume`、`Pause`、`Resume`，每个变体携带完整参数
  - [x] SubTask 2.4: 定义 `AudioEngine` trait：`load_sound`、`play`、`stop`、`set_volume`、`pause`、`resume`、`update` 方法
  - [x] SubTask 2.5: 定义 `AudioContext` 结构，收集一帧中的音频命令并提交给 AudioEngine
  - [x] SubTask 2.6: 在根 `Cargo.toml` 中添加 `gg-runtime-audio` 到 workspace

- [x] Task 3: 实现桌面端音频后端（CpalAudioEngine）
  - [x] SubTask 3.1: 在 `gg-runtime-audio/Cargo.toml` 中添加 cpal、rodio 依赖（可选 feature gate）
  - [x] SubTask 3.2: 实现 `CpalAudioEngine` 结构：初始化 cpal 音频输出设备和 rodio 输出流
  - [x] SubTask 3.3: 实现音频文件加载：通过 rodio::Decoder 解码 WAV/OGG/MP3，缓存解码数据，返回 SoundId
  - [x] SubTask 3.4: 实现播放控制：play（创建 rodio::Sink 播放）、stop、pause、resume、set_volume
  - [x] SubTask 3.5: 实现 `AudioEngine` trait 的 `update` 方法：处理 AudioCommand 队列
  - [x] SubTask 3.6: 实现 `AudioEngine` trait 的其余方法

- [x] Task 4: 实现 HMR 热更新基础架构
  - [x] SubTask 4.1: 定义 `HmrEvent` 枚举：`ScriptChanged`（模块名、新字节码）、`AssetChanged`（资源路径、新数据）
  - [x] SubTask 4.2: 定义 `HmrManager` 结构：接收 HmrEvent 队列，管理热替换流程
  - [x] SubTask 4.3: 实现脚本热替换：替换 ScriptEngine 中的 IrModule，保留全局变量状态
  - [x] SubTask 4.4: 实现资源热替换：更新 AssetManager 中的资源缓存，通知引用该资源的系统
  - [x] SubTask 4.5: 实现状态迁移钩子：调用 `on_hot_reload_out`/`on_hot_reload_in` 脚本函数
  - [x] SubTask 4.6: 实现热替换失败时的回滚机制

- [x] Task 5: 定义 WASM 脚本虚拟机集成接口
  - [x] SubTask 5.1: 定义 `WasmRuntime` trait：`load_module`、`instantiate`、`call_function`、`get_memory` 方法
  - [x] SubTask 5.2: 定义 `WasmHostFunctions` trait：暴露引擎 API 给 WASM 模块（spawn_entity、add_component、get_component、set_component 等）
  - [x] SubTask 5.3: 定义 `WasmSandboxConfig` 结构：内存限制、执行时间限制、允许的导入函数列表
  - [x] SubTask 5.4: 定义 `WasmModuleId` 和 `WasmInstanceId` 类型

- [x] Task 6: 重构 gg-runtime-core Runtime 结构
  - [x] SubTask 6.1: 定义 `RuntimeBuilder` 结构：提供 builder 模式构建 Runtime，支持 `renderer()`、`audio_engine()`、`hmr_enabled()` 方法
  - [x] SubTask 6.2: 重构 `Runtime` 结构：替换 `RenderSystem` 为 `Box<dyn Renderer>`，替换 `Scheduler` 为 `StageScheduler`，添加 `Option<Box<dyn AudioEngine>>` 和 `Option<HmrManager>`
  - [x] SubTask 6.3: 实现组件注册表（ComponentRegistry）：替代 EngineHost 中硬编码的 Position/Velocity，支持动态注册组件类型
  - [x] SubTask 6.4: 重构 `EngineHost`：使用 ComponentRegistry 处理组件操作，移除硬编码组件类型
  - [x] SubTask 6.5: 重构 `Runtime::tick`：使用 StageScheduler 按阶段执行系统，集成 HMR 检查
  - [x] SubTask 6.6: 重构 `Runtime::start`：使用 RuntimeBuilder 构建参数初始化渲染器和音频引擎
  - [x] SubTask 6.7: 更新 `gg-runtime-core/Cargo.toml` 依赖：添加 gg-runtime-audio

- [x] Task 7: 更新示例和集成验证
  - [x] SubTask 7.1: 更新 `examples/basic` 使用 RuntimeBuilder 构建 Runtime
  - [x] SubTask 7.2: 验证阶段调度器正确执行各阶段系统
  - [x] SubTask 7.3: 验证音频后端能加载和播放音频文件
  - [x] SubTask 7.4: 验证 HMR 热替换脚本能保留游戏状态

# Task Dependencies
- Task 2 depends on nothing（音频 HAL 定义独立）
- Task 3 depends on Task 2（CpalAudioEngine 需要实现 AudioEngine trait）
- Task 4 depends on nothing（HMR 基础架构独立定义）
- Task 5 depends on nothing（WASM 接口定义独立）
- Task 1 depends on nothing（调度器独立定义）
- Task 6 depends on Task 1, Task 2, Task 4, Task 5（Runtime 重构需要调度器、音频 HAL、HMR、WASM 接口）
- Task 7 depends on Task 6（示例验证需要完整的 Runtime）
