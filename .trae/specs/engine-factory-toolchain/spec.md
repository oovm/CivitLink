# 引擎工厂与工具链 Spec

## Why
当前 GG 元游戏引擎缺少引擎工厂（gg-factory）和命令行工具（gg-cli），引擎实例（如 galgame）是手动硬编码的 Rust 二进制项目，而非从引擎清单声明式生成。按照架构设计文档，元引擎的核心价值在于"通过引擎清单选择插件组合，编译生成定制引擎"，但目前这一生成机制完全不存在。此外，构建系统仅支持基本的 CI 检查，缺乏多平台构建和分发能力。

## What Changes
- 新增 `gg-factory` crate：引擎工厂核心，解析引擎清单并生成引擎项目代码
- 新增 `gg-cli` crate：命令行工具，提供项目初始化、构建、打包等子命令
- 重构引擎清单格式：从 `GalgameManifest`（Galgame 专用）升级为通用的 `EngineManifest`，支持模块选取、平台目标、工具链扩展点
- 新增 `gg-manifest` crate：独立的引擎清单类型定义和解析库，与具体引擎解耦
- 扩展 `gg-core` 的平台抽象：填充 `BuildConfig`、`GenerateContext`、`PackageContext`、`RunContext` 的实际字段
- 新增桌面平台构建脚本：支持 Windows/macOS/Linux 的构建和打包
- 更新 CI 流水线：添加多平台构建和产物上传

## Impact
- Affected specs: 元引擎工厂架构、平台抽象层、CI/CD 流水线
- Affected code: `gg-core`（平台抽象填充）、`gg-galgame-schema/manifest.rs`（降级为引擎清单的 Galgame 特化视图）、新增 `gg-factory`、新增 `gg-cli`、新增 `gg-manifest`、`.github/workflows/ci.yml`（扩展）

## ADDED Requirements

### Requirement: 引擎清单格式定义
系统 SHALL 在 `gg-manifest` crate 中定义通用的引擎清单格式，作为引擎工厂的输入。

#### Scenario: 清单基本结构
- **WHEN** 开发者编写引擎清单文件 `Engine.toml`
- **THEN** 清单包含以下顶层节：
  - `[engine]`：引擎元数据（name, version, description, game_type）
  - `[modules]`：模块选取（gom, vm, render, plugins）
  - `[platforms]`：目标平台列表及各平台配置
  - `[toolchain]`：工具链扩展点（编译器步骤、编辑器面板）
  - `[display]`：默认窗口配置（width, height, fullscreen, title）

#### Scenario: 模块选取声明
- **WHEN** 开发者在清单中声明模块选取
- **THEN** `[modules]` 节支持以下字段：
  - `gom`：游戏对象模型（如 `"VisualNovel"`, `"ARPG"`, `"STG"`）
  - `vm`：脚本运行时（如 `"Wasm"`, `"Lua"`, `"BehaviorTree"`）
  - `render`：渲染前端（如 `"SpriteStack"`, `"Immediate2D"`）
  - `plugins`：插件列表（如 `["dialogue", "portrait", "scene-transition", "save"]`）

#### Scenario: 平台目标声明
- **WHEN** 开发者在清单中声明目标平台
- **THEN** `[platforms]` 节支持列表形式，每个平台项包含：
  - `target`：编译目标三元组（如 `"x86_64-pc-windows-msvc"`）
  - `name`：平台显示名称（如 `"Windows"`）
  - `features`：平台特定的 Cargo features

#### Scenario: 清单解析
- **WHEN** 系统读取有效的 `Engine.toml` 文件
- **THEN** `gg-manifest` 将 TOML 内容反序列化为 `EngineManifest` 结构体
- **AND** 对必填字段进行验证（name 非空、至少一个 plugin、至少一个 platform）
- **AND** 返回解析成功的 `EngineManifest`

#### Scenario: 清单解析失败
- **WHEN** 系统读取格式错误或缺少必填字段的 `Engine.toml`
- **THEN** 返回包含具体字段路径和错误原因的 `GError`

### Requirement: 引擎工厂核心功能
系统 SHALL 在 `gg-factory` crate 中实现引擎工厂，根据引擎清单生成引擎项目代码。

#### Scenario: 生成引擎项目骨架
- **WHEN** 引擎工厂接收有效的 `EngineManifest`
- **THEN** 在指定输出目录生成完整的 Cargo 项目，包含：
  - `Cargo.toml`：根据清单中的模块和插件声明生成依赖项
  - `src/main.rs`：引擎入口点，包含初始化和主循环代码
  - `src/config.rs`：引擎配置类型定义
  - `src/engine.rs`：引擎结构体，集成所选插件的初始化逻辑

#### Scenario: 生成 Cargo.toml
- **WHEN** 引擎工厂生成 `Cargo.toml`
- **THEN** 文件包含：
  - workspace 级别的 `gg-core`、`gg-ecs`、`gg-asset` 依赖
  - 根据清单 `modules.plugins` 列表添加对应的插件 crate 依赖
  - 根据清单 `modules.gom` 添加对应的 schema crate 依赖
  - 根据清单 `platforms` 配置的 features 生成 `[features]` 节

#### Scenario: 生成引擎入口代码
- **WHEN** 引擎工厂生成 `src/main.rs`
- **THEN** 入口代码包含：
  - 命令行参数解析（`--editor`、`--project`）
  - 加载游戏配置文件
  - 初始化引擎并注册清单中声明的所有插件
  - 运行主循环

#### Scenario: 生成引擎配置类型
- **WHEN** 引擎工厂生成 `src/config.rs`
- **THEN** 配置类型包含：
  - `GameSection`：游戏名称、版本、初始场景
  - `DisplaySection`：窗口宽度、高度、全屏模式
  - `AudioSection`：音量配置
  - 反序列化支持（serde + toml）

#### Scenario: 生成引擎结构体
- **WHEN** 引擎工厂生成 `src/engine.rs`
- **THEN** 引擎结构体包含：
  - 持有 `Scheduler`、`AssetManager`、配置等字段
  - `initialize` 方法：注册清单中声明的所有插件
  - `tick` 方法：执行一帧
  - `run` 方法：运行主循环

### Requirement: 命令行工具 gg-cli
系统 SHALL 提供 `gg-cli` 命令行工具，作为开发者与引擎工厂交互的入口。

#### Scenario: 初始化新引擎项目
- **WHEN** 开发者执行 `gg-cli init <engine-name> --type <game-type>`
- **THEN** 在当前目录创建 `<engine-name>/` 目录
- **AND** 生成 `Engine.toml` 模板文件，预填 game_type 对应的默认模块和插件
- **AND** 生成 `game.toml` 游戏配置模板
- **AND** 生成 `scripts/` 和 `assets/` 目录

#### Scenario: 从清单生成引擎代码
- **WHEN** 开发者在包含 `Engine.toml` 的目录执行 `gg-cli generate`
- **THEN** 引擎工厂解析 `Engine.toml`
- **AND** 在 `generated/` 目录下生成引擎项目代码
- **AND** 输出生成结果摘要（生成的文件列表、依赖的插件列表）

#### Scenario: 构建引擎
- **WHEN** 开发者执行 `gg-cli build [--platform <target>] [--release]`
- **THEN** 系统调用 `cargo build` 编译生成的引擎项目
- **AND** `--platform` 指定编译目标三元组
- **AND** `--release` 启用 release 模式编译

#### Scenario: 创建新游戏项目
- **WHEN** 开发者执行 `gg-cli new-game <game-name>`
- **THEN** 在当前目录创建 `<game-name>/` 目录
- **AND** 生成 `game.toml` 游戏配置文件
- **AND** 生成 `scripts/start.gscript` 初始脚本
- **AND** 生成 `assets/` 资源目录

### Requirement: 引擎清单模板
系统 SHALL 为每种游戏类型提供预设的引擎清单模板。

#### Scenario: VisualNovel 模板
- **WHEN** 开发者使用 `--type VisualNovel` 初始化项目
- **THEN** 生成的 `Engine.toml` 预填：
  - `modules.gom = "VisualNovel"`
  - `modules.plugins = ["dialogue", "portrait", "scene-transition", "save"]`
  - `modules.render = "SpriteStack"`
  - `modules.vm = "Wasm"`

#### Scenario: ARPG 模板
- **WHEN** 开发者使用 `--type ARPG` 初始化项目
- **THEN** 生成的 `Engine.toml` 预填：
  - `modules.gom = "ARPG"`
  - `modules.plugins = ["dialogue", "save"]`
  - `modules.render = "Immediate2D"`
  - `modules.vm = "Wasm"`

#### Scenario: 自定义模板
- **WHEN** 开发者使用 `--type Custom` 初始化项目
- **THEN** 生成的 `Engine.toml` 包含最小化的占位配置
- **AND** 开发者需手动填写模块和插件选择

### Requirement: 平台抽象层填充
系统 SHALL 填充 `gg-core` 中平台抽象的空壳结构体，为构建系统提供类型基础。

#### Scenario: BuildConfig 字段
- **WHEN** 平台实现需要配置构建参数
- **THEN** `BuildConfig` 包含字段：
  - `target_triple`：编译目标三元组
  - `release`：是否 release 模式
  - `features`：Cargo features 列表
  - `output_dir`：输出目录路径

#### Scenario: GenerateContext 字段
- **WHEN** 平台实现需要生成代码
- **THEN** `GenerateContext` 包含字段：
  - `manifest`：引擎清单引用
  - `output_dir`：生成代码输出目录
  - `template_dir`：模板文件目录（可选）

#### Scenario: PackageContext 字段
- **WHEN** 平台实现需要打包产物
- **THEN** `PackageContext` 包含字段：
  - `build_output_dir`：构建输出目录
  - `package_output_dir`：打包输出目录
  - `manifest`：引擎清单引用
  - `assets_dir`：游戏资源目录

#### Scenario: RunContext 字段
- **WHEN** 平台实现需要本地运行
- **THEN** `RunContext` 包含字段：
  - `executable_path`：可执行文件路径
  - `project_dir`：游戏项目目录
  - `args`：运行时参数

### Requirement: 桌面平台构建支持
系统 SHALL 支持桌面平台（Windows、macOS、Linux）的构建和打包。

#### Scenario: Windows 构建
- **WHEN** 执行 `gg-cli build --platform x86_64-pc-windows-msvc --release`
- **THEN** 系统编译生成 Windows 可执行文件
- **AND** 可执行文件位于 `target/x86_64-pc-windows-msvc/release/` 目录

#### Scenario: macOS 构建
- **WHEN** 执行 `gg-cli build --platform aarch64-apple-darwin --release`
- **THEN** 系统编译生成 macOS 可执行文件

#### Scenario: Linux 构建
- **WHEN** 执行 `gg-cli build --platform x86_64-unknown-linux-gnu --release`
- **THEN** 系统编译生成 Linux 可执行文件

### Requirement: CI 多平台构建流水线
系统 SHALL 在 CI 流水线中添加多平台构建和产物上传步骤。

#### Scenario: 标签触发构建
- **WHEN** 推送 `v*` 格式的标签
- **THEN** CI 在 Windows/macOS/Linux 三个平台上构建 `gg-galgame`
- **AND** 将构建产物上传为 GitHub Actions Artifact

#### Scenario: WASM 构建
- **WHEN** 推送 `v*` 格式的标签
- **THEN** CI 构建 `wasm32-unknown-unknown` 目标
- **AND** 使用 `wasm-bindgen` 生成 Web 绑定
- **AND** 上传 WASM 产物

## MODIFIED Requirements

### Requirement: gg-core 平台抽象
`gg-core` 中的 `platform` 模块 SHALL 填充 `BuildConfig`、`GenerateContext`、`PackageContext`、`RunContext` 的实际字段，替代当前的空壳结构体。`Platform` trait 的方法签名保持不变。

### Requirement: gg-galgame-schema manifest
`gg-galgame-schema` 中的 `GalgameManifest` SHALL 保留作为 Galgame 特化的清单视图，但新增从通用 `EngineManifest` 转换的 `From<&EngineManifest>` 实现，使 Galgame 引擎可以从通用清单获取 Galgame 特有配置。

## REMOVED Requirements

无移除项。
