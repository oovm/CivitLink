# Valkyrie 脚本语言集成 Spec

## Why
当前 GG 引擎的示例使用 Rust 编写游戏逻辑，但按照设计文档，游戏逻辑应使用脚本语言开发，编译到 IR/字节码，以实现跨平台支持和热更新能力。Valkyrie 语言（基于 oaks 框架）已具备完整的词法分析、语法分析和 AST 构建能力，适合作为 GG 引擎的脚本前端。

## What Changes
- 新增 `gg-script` 模块：集成 Valkyrie 语言前端，将 Valkyrie 源码编译为 AST
- 新增 `gg-ir` 模块：定义引擎中间表示（IR），作为脚本编译和优化的核心
- 新增 `gg-vm` 模块：实现字节码虚拟机，执行 IR 编译后的字节码
- 新增 `gg-compiler` 模块：实现从 Valkyrie AST 到 IR 再到字节码的编译流水线
- **BREAKING** 重写 `examples/basic`：使用 Valkyrie 脚本而非 Rust 编写游戏逻辑
- 更新 `gg-runtime-core`：集成脚本执行环境，支持脚本驱动的游戏循环

## Impact
- Affected specs: 运行时层（Runtime Layer）、元编译器架构、脚本执行环境
- Affected code: `gg-runtime-core`、`examples/basic`、新增 `gg-script`/`gg-ir`/`gg-vm`/`gg-compiler` 模块

## ADDED Requirements

### Requirement: Valkyrie 脚本前端集成
系统 SHALL 提供 Valkyrie 脚本语言的编译前端，将 `.valkyrie` 源文件编译为 AST。

#### Scenario: 成功编译 Valkyrie 脚本
- **WHEN** 用户提供有效的 Valkyrie 源码字符串
- **THEN** 系统通过 ValkyrieLexer -> ValkyrieParser -> ValkyrieBuilder 流程生成 ValkyrieRoot AST
- **AND** 返回编译成功的 GResult

#### Scenario: 脚本编译失败
- **WHEN** 用户提供包含语法错误的 Valkyrie 源码
- **THEN** 系统返回包含诊断信息的 GError

### Requirement: IR 中间表示定义
系统 SHALL 定义引擎中间表示（IR），作为 Valkyrie AST 到字节码的桥梁。

#### Scenario: AST 到 IR 转换
- **WHEN** 系统接收到有效的 ValkyrieRoot AST
- **THEN** 系统将 AST 转换为 IR 指令序列
- **AND** IR 指令包含：加载常量、二元运算、函数调用、实体操作、组件操作等

#### Scenario: IR 优化
- **WHEN** 系统生成 IR 指令序列
- **THEN** 系统可对 IR 进行常量折叠、死代码消除等基本优化

### Requirement: 字节码虚拟机
系统 SHALL 提供字节码虚拟机，执行 IR 编译后的字节码。

#### Scenario: 字节码执行
- **WHEN** 虚拟机接收到字节码
- **THEN** 虚拟机按指令顺序执行字节码
- **AND** 虚拟机维护执行栈、调用栈和全局状态

#### Scenario: 脚本调用引擎 API
- **WHEN** 字节码中包含引擎 API 调用指令（如 spawn_entity、add_component）
- **THEN** 虚拟机通过宿主接口调用引擎 Rust 实现
- **AND** 返回结果到虚拟机执行栈

### Requirement: 脚本驱动的游戏循环
系统 SHALL 支持通过 Valkyrie 脚本定义游戏逻辑，而非 Rust 代码。

#### Scenario: 脚本定义游戏初始化
- **WHEN** 引擎启动并加载 Valkyrie 脚本
- **THEN** 脚本中的 `init` 函数被调用，创建实体和组件

#### Scenario: 脚本定义游戏更新
- **WHEN** 游戏循环每帧执行
- **THEN** 脚本中的 `update` 函数被调用，更新游戏状态

#### Scenario: 热更新脚本
- **WHEN** 开发者修改 Valkyrie 脚本文件并保存
- **THEN** 引擎检测文件变更，重新编译脚本并替换运行中的字节码
- **AND** 游戏状态不丢失

### Requirement: Valkyrie 脚本示例
系统 SHALL 提供使用 Valkyrie 脚本编写的游戏示例。

#### Scenario: 基本游戏示例
- **WHEN** 用户运行 basic 示例
- **THEN** 引擎加载 `.valkyrie` 脚本文件
- **AND** 脚本定义实体创建、组件添加和系统逻辑
- **AND** 游戏窗口显示运行结果

## MODIFIED Requirements

### Requirement: 运行时系统
运行时系统 SHALL 集成脚本执行环境，支持脚本驱动的游戏循环。原有的 Rust 直接编写游戏逻辑的方式改为通过脚本 VM 执行。

## REMOVED Requirements

### Requirement: Rust 编写的游戏逻辑示例
**Reason**: 按照设计文档，游戏逻辑应使用脚本语言编写，而非直接用 Rust
**Migration**: 将 Rust 游戏逻辑迁移到 Valkyrie 脚本文件，通过 VM 执行
