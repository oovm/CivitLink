# Tasks

- [ ] Task 1: 创建 gg-manifest crate，定义引擎清单类型
  - [ ] SubTask 1.1: 创建 `projects/toolchain/gg-manifest/` 目录和 `Cargo.toml`
  - [ ] SubTask 1.2: 实现 `EngineManifest` 结构体及子结构（EngineSection, ModulesSection, PlatformEntry, ToolchainSection, DisplaySection）
  - [ ] SubTask 1.3: 实现 TOML 反序列化和验证逻辑（必填字段校验）
  - [ ] SubTask 1.4: 实现游戏类型模板函数（`visual_novel_template()`, `arpg_template()`, `custom_template()`）
  - [ ] SubTask 1.5: 编写单元测试（解析、验证、模板生成）

- [ ] Task 2: 填充 gg-core 平台抽象结构体
  - [ ] SubTask 2.1: 为 `BuildConfig` 添加字段（target_triple, release, features, output_dir）
  - [ ] SubTask 2.2: 为 `GenerateContext` 添加字段（manifest 引用、output_dir、template_dir）
  - [ ] SubTask 2.3: 为 `PackageContext` 添加字段（build_output_dir, package_output_dir, manifest 引用、assets_dir）
  - [ ] SubTask 2.4: 为 `RunContext` 添加字段（executable_path, project_dir, args）
  - [ ] SubTask 2.5: 为所有新增结构体添加构造函数和文档注释

- [ ] Task 3: 创建 gg-factory crate，实现引擎工厂核心
  - [ ] SubTask 3.1: 创建 `projects/toolchain/gg-factory/` 目录和 `Cargo.toml`
  - [ ] SubTask 3.2: 实现 `EngineFactory` 结构体和 `generate()` 方法
  - [ ] SubTask 3.3: 实现 Cargo.toml 生成器（根据清单生成依赖项和 features）
  - [ ] SubTask 3.4: 实现 main.rs 代码生成器（入口点、参数解析、插件初始化）
  - [ ] SubTask 3.5: 实现 config.rs 代码生成器（游戏配置类型定义）
  - [ ] SubTask 3.6: 实现 engine.rs 代码生成器（引擎结构体、初始化、主循环）
  - [ ] SubTask 3.7: 编写集成测试（从清单生成完整项目并验证文件结构）

- [ ] Task 4: 创建 gg-cli crate，实现命令行工具
  - [ ] SubTask 4.1: 创建 `projects/toolchain/gg-cli/` 目录和 `Cargo.toml`（binary crate）
  - [ ] SubTask 4.2: 实现 CLI 参数解析（init, generate, build, new-game 子命令）
  - [ ] SubTask 4.3: 实现 `init` 子命令（创建引擎项目目录和 Engine.toml 模板）
  - [ ] SubTask 4.4: 实现 `generate` 子命令（调用 gg-factory 生成引擎代码）
  - [ ] SubTask 4.5: 实现 `build` 子命令（调用 cargo build 编译生成的项目）
  - [ ] SubTask 4.6: 实现 `new-game` 子命令（创建游戏项目骨架）
  - [ ] SubTask 4.7: 编写端到端测试（init → generate → build 流程）

- [ ] Task 5: 更新 gg-galgame-schema manifest 模块
  - [ ] SubTask 5.1: 添加对 `gg-manifest` 的依赖
  - [ ] SubTask 5.2: 实现 `From<&EngineManifest>` for `GalgameManifest` 转换
  - [ ] SubTask 5.3: 编写转换测试

- [ ] Task 6: 更新 workspace Cargo.toml 和 CI 流水线
  - [ ] SubTask 6.1: 将 `projects/toolchain/*` 添加到 workspace members
  - [ ] SubTask 6.2: 在 workspace.dependencies 中添加 gg-manifest 和 gg-factory
  - [ ] SubTask 6.3: 扩展 CI 流水线，添加标签触发的多平台构建 job
  - [ ] SubTask 6.4: 添加 WASM 构建 job（wasm32-unknown-unknown + wasm-bindgen）

- [ ] Task 7: 验证整体流程
  - [ ] SubTask 7.1: 使用 gg-cli init 创建 VisualNovel 引擎项目
  - [ ] SubTask 7.2: 使用 gg-cli generate 生成引擎代码
  - [ ] SubTask 7.3: 验证生成的项目可通过 cargo check
  - [ ] SubTask 7.4: 运行 workspace 全量测试确保无回归

# Task Dependencies
- [Task 2] depends on [Task 1]（GenerateContext 需要 manifest 类型）
- [Task 3] depends on [Task 1]（gg-factory 依赖 gg-manifest）
- [Task 4] depends on [Task 3]（gg-cli 调用 gg-factory）
- [Task 5] depends on [Task 1]（GalgameManifest 转换依赖 EngineManifest）
- [Task 6] depends on [Task 1, Task 3, Task 4]（workspace 和 CI 需要新 crate）
- [Task 7] depends on [Task 4, Task 6]（验证需要完整工具链和 CI）
