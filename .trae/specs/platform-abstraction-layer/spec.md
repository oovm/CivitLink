# 平台抽象层实现 Spec

## Why

当前 GG 引擎的 `gg-core` 中仅有占位式的 `Platform` trait（仅包含构建/打包相关接口），缺少运行时所需的文件系统、输入、时间等核心平台抽象。按照架构设计文档，平台抽象层是太一的"遍在性"，屏蔽操作系统和运行环境差异，为上层提供统一接口。目前 `gg-asset` 直接使用 `std::fs` 读取文件，无法在 Web/移动端运行；运行时游戏循环中也没有统一的输入事件抽象。需要实现完整的平台抽象层，使引擎能适配 Windows、macOS、Web（WASM）等多平台。

## What Changes

- 重构 `gg-core::platform` 模块：从仅含构建接口扩展为包含运行时平台服务的完整抽象
- 新增文件系统抽象：定义 `FileSystem` trait 及桌面/Web 两种实现
- 新增输入抽象：定义 `Input` trait 统一处理键盘、鼠标/触摸、游戏手柄事件
- 新增时间抽象：定义 `Time` trait 统一获取系统时间、帧间隔
- 新增 `gg-platform-desktop` crate：桌面平台（Windows/macOS/Linux）的具体实现
- 新增 `gg-platform-web` crate：Web 平台（WASM）的具体实现
- 新增 `gg-platform-mobile` crate 占位：移动平台架构设计（仅定义 trait，暂不实现）
- 更新 `gg-asset`：从直接使用 `std::fs` 改为通过 `FileSystem` trait 抽象访问
- 更新 `Cargo.toml` workspace：注册新平台 crate
- **BREAKING** 重写 `gg-core::platform` 模块的公共 API

## Impact

- Affected specs: 平台抽象层、资源系统、运行时核心
- Affected code:
  - `projects/core/gg-core/src/lib.rs` — 重构 platform 模块
  - `projects/core/gg-asset/src/lib.rs` — 改用 FileSystem trait
  - `projects/runtime/gg-runtime-core/src/lib.rs` — 集成平台服务
  - 新增 `projects/platforms/gg-platform-desktop/`
  - 新增 `projects/platforms/gg-platform-web/`
  - 新增 `projects/platforms/gg-platform-mobile/`
  - `Cargo.toml` — workspace 成员更新

---

## ADDED Requirements

### Requirement: 文件系统抽象

系统 SHALL 在 `gg-core::platform` 中定义 `FileSystem` trait，提供跨平台的文件访问接口。

#### Scenario: 读取文件内容
- **WHEN** 引擎需要读取资源文件
- **THEN** 通过 `FileSystem::read(&self, path: &Path) -> GResult<Vec<u8>>` 读取文件二进制内容
- **AND** 桌面平台使用 `std::fs` 实现
- **AND** Web 平台通过 `fetch` API 实现

#### Scenario: 读取文本文件
- **WHEN** 引擎需要读取文本内容
- **THEN** 通过 `FileSystem::read_to_string(&self, path: &Path) -> GResult<String>` 读取
- **AND** 提供默认实现基于 `read` 方法

#### Scenario: 检查文件是否存在
- **WHEN** 引擎需要判断文件是否可访问
- **THEN** 通过 `FileSystem::exists(&self, path: &Path) -> bool` 判断

#### Scenario: 读取目录内容
- **WHEN** 引擎需要列举目录下的文件
- **THEN** 通过 `FileSystem::read_dir(&self, path: &Path) -> GResult<Vec<DirEntry>>` 获取目录条目列表
- **AND** Web 平台此方法返回空列表或错误（浏览器不支持目录遍历）

#### Scenario: 写入文件
- **WHEN** 引擎需要持久化数据（如存档）
- **THEN** 通过 `FileSystem::write(&self, path: &Path, content: &[u8]) -> GResult<()>` 写入
- **AND** Web 平台使用 localStorage 或 IndexedDB 模拟

#### Scenario: 创建目录
- **WHEN** 引擎需要创建目录结构
- **THEN** 通过 `FileSystem::create_dir_all(&self, path: &Path) -> GResult<()>` 递归创建
- **AND** Web 平台此方法为空操作

#### Scenario: 获取文件元数据
- **WHEN** 引擎需要获取文件大小、修改时间等信息
- **THEN** 通过 `FileSystem::metadata(&self, path: &Path) -> GResult<FileMetadata>` 获取
- **AND** `FileMetadata` 包含 `file_type: FileType`、`len: u64`、`modified: Option<SystemTime>`

### Requirement: 输入抽象

系统 SHALL 在 `gg-core::platform` 中定义 `Input` trait 和输入事件类型，统一处理各平台输入。

#### Scenario: 轮询输入事件
- **WHEN** 游戏循环每帧需要处理输入
- **THEN** 通过 `Input::poll_events(&mut self) -> Vec<InputEvent>` 获取当前帧的所有输入事件
- **AND** `InputEvent` 包含键盘、鼠标、触摸、游戏手柄等变体

#### Scenario: 键盘输入
- **WHEN** 用户按下或释放键盘按键
- **THEN** 产生 `InputEvent::Keyboard { key: KeyCode, state: KeyState }` 事件
- **AND** `KeyCode` 枚举覆盖常用按键（字母、数字、方向键、功能键等）
- **AND** `KeyState` 区分 `Pressed` 和 `Released`

#### Scenario: 鼠标/触摸输入
- **WHEN** 用户操作鼠标或触摸屏
- **THEN** 产生 `InputEvent::Pointer { position: (f32, f32), action: PointerAction, button: Option<PointerButton> }` 事件
- **AND** `PointerAction` 包含 `Down`、`Up`、`Move`、`Scroll(f32)` 变体
- **AND** 桌面平台映射鼠标事件，Web/移动端映射触摸事件

#### Scenario: 查询按键状态
- **WHEN** 游戏逻辑需要查询某按键是否处于按下状态
- **THEN** 通过 `Input::is_key_pressed(&self, key: KeyCode) -> bool` 查询
- **AND** 通过 `Input::is_pointer_down(&self) -> bool` 查询指针按下状态
- **AND** 通过 `Input::pointer_position(&self) -> (f32, f32)` 获取指针当前位置

### Requirement: 时间抽象

系统 SHALL 在 `gg-core::platform` 中定义 `Time` trait，统一获取时间信息。

#### Scenario: 获取帧间隔
- **WHEN** 游戏循环需要计算帧间隔
- **THEN** 通过 `Time::delta(&self) -> Duration` 获取上一帧到当前帧的时间间隔

#### Scenario: 获取运行时间
- **WHEN** 游戏逻辑需要获取自启动以来的时间
- **THEN** 通过 `Time::elapsed(&self) -> Duration` 获取

#### Scenario: 更新时间
- **WHEN** 游戏循环每帧开始时
- **THEN** 通过 `Time::update(&mut self)` 更新内部时间状态

### Requirement: 平台服务注册表

系统 SHALL 在 `gg-core::platform` 中定义 `PlatformServices` 结构，聚合所有平台服务实例。

#### Scenario: 创建平台服务
- **WHEN** 引擎初始化时
- **THEN** 通过 `PlatformServices::new()` 创建，根据编译目标自动选择平台实现
- **AND** `PlatformServices` 包含 `file_system: Box<dyn FileSystem>`、`input: Box<dyn Input>`、`time: Box<dyn Time>` 字段

#### Scenario: 桌面平台自动选择
- **WHEN** 编译目标为 `not(target_arch = "wasm32")`
- **THEN** `PlatformServices::new()` 使用桌面平台实现（`gg-platform-desktop`）

#### Scenario: Web 平台自动选择
- **WHEN** 编译目标为 `target_arch = "wasm32"`
- **THEN** `PlatformServices::new()` 使用 Web 平台实现（`gg-platform-web`）

### Requirement: 桌面平台实现

系统 SHALL 提供 `gg-platform-desktop` crate，为 Windows/macOS/Linux 实现所有平台 trait。

#### Scenario: 桌面文件系统
- **WHEN** `DesktopFileSystem` 实现 `FileSystem` trait
- **THEN** 使用 `std::fs` 进行文件操作
- **AND** 所有方法均为同步实现（在异步上下文中通过 `block_on` 适配）

#### Scenario: 桌面输入
- **WHEN** `DesktopInput` 实现 `Input` trait
- **THEN** 维护内部按键状态映射和指针状态
- **AND** 通过 `push_event` 方法接收来自窗口系统的事件
- **AND** `poll_events` 返回自上次调用以来累积的事件并清空缓冲

#### Scenario: 桌面时间
- **WHEN** `DesktopTime` 实现 `Time` trait
- **THEN** 使用 `std::time::Instant` 计算帧间隔和运行时间

### Requirement: Web 平台实现

系统 SHALL 提供 `gg-platform-web` crate，为 WASM/Web 环境实现所有平台 trait。

#### Scenario: Web 文件系统
- **WHEN** `WebFileSystem` 实现 `FileSystem` trait
- **THEN** `read` 方法通过 HTTP fetch 从服务器获取文件
- **AND** `write` 方法使用 `web_sys::window()` 的 localStorage 或 IndexedDB
- **AND** `exists` 方法通过 HEAD 请求检测
- **AND** `read_dir`、`create_dir_all` 等方法返回不支持错误

#### Scenario: Web 输入
- **WHEN** `WebInput` 实现 `Input` trait
- **THEN** 通过 JavaScript 互操作接收 DOM 事件（keydown/keyup、mousedown/mousemove/mouseup、touchstart/touchmove/touchend）
- **AND** 将 DOM 事件转换为统一的 `InputEvent`

#### Scenario: Web 时间
- **WHEN** `WebTime` 实现 `Time` trait
- **THEN** 使用 `performance.now()` 或 `Date.now()` 获取高精度时间
- **AND** 通过 `web_sys::window()` 访问浏览器时间 API

### Requirement: 移动平台占位

系统 SHALL 提供 `gg-platform-mobile` crate 占位，定义移动平台架构设计。

#### Scenario: 移动平台 trait 定义
- **WHEN** 开发者查看 `gg-platform-mobile`
- **THEN** 包含移动平台特有的 trait 定义（如 `MobileLifecycle`、`MobileInput`）
- **AND** `MobileLifecycle` trait 包含 `on_pause`、`on_resume`、`on_destroy` 方法
- **AND** 不包含具体实现（仅架构设计）

### Requirement: 资源系统适配

`gg-asset` SHALL 通过 `FileSystem` trait 访问文件，而非直接使用 `std::fs`。

#### Scenario: AssetManager 使用 FileSystem
- **WHEN** `AssetManager` 加载资源文件
- **THEN** 通过注入的 `FileSystem` trait 对象调用 `read` 方法
- **AND** 不再直接使用 `std::fs::File` 或 `std::io::Read`

#### Scenario: AssetManager 构造
- **WHEN** 创建 `AssetManager` 实例
- **THEN** 通过 `AssetManager::new(file_system: Box<dyn FileSystem>)` 注入平台文件系统

## MODIFIED Requirements

### Requirement: gg-core::platform 模块

`gg-core::platform` 模块 SHALL 从仅含构建时接口扩展为包含运行时平台服务的完整抽象。原有的 `Platform` trait（`id`、`display_name`、`configure_build`、`generate_code`、`package`、`run`）保留但移至 `gg-core::platform::build` 子模块，新增 `FileSystem`、`Input`、`Time` 等 trait 在 `gg-core::platform` 根模块。

### Requirement: gg-asset 资源加载

`AssetManager` 的资源加载 SHALL 通过 `FileSystem` trait 进行，不再直接使用 `std::fs`。构造函数签名从 `AssetManager::new()` 改为 `AssetManager::new(file_system: Box<dyn FileSystem>)`。

## REMOVED Requirements

### Requirement: gg-asset 直接使用 std::fs
**Reason**: 资源系统需要跨平台支持，不能直接依赖 `std::fs`（Web 平台不可用）
**Migration**: 所有 `std::fs::File::open` 和 `std::io::Read` 调用替换为 `FileSystem::read` 调用
