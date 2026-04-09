# 移除 SimpleLexer/SimpleParser，复用 oak-valkyrie 推进 gg-script Spec

## Why
当前 `gg-script` 模块自行实现了简化的 `SimpleLexer` + `SimpleParser`，仅支持 Valkyrie 语法的一个极小子集（micro 函数、let 绑定、if/else、基本表达式），且未依赖同组织的 `oak-valkyrie` crate。`oak-valkyrie` 已具备完整的词法分析、语法分析和 AST 构建能力，支持 namespace、class、enums、trait、match、loop、lambda 等完整语言特性。移除自实现的简易解析器，复用 oak-valkyrie 的完整前端，可以让 gg-script 获得完整的 Valkyrie 语言支持，避免重复造轮子，并为后续类型检查、LSP 集成等功能奠定基础。

## What Changes
- 移除 `gg-script` 中的 `SimpleToken`、`SimpleLexer`、`SimpleParser` 及相关测试
- 添加 `oak-valkyrie` 为 `gg-script` 的依赖
- 新增 `ValkyrieCompiler`：将 `ValkyrieRoot` AST 编译为 `IrModule`
- 重构 `ScriptCompiler`：使用 `ValkyrieBuilder` 解析源码生成 AST，再通过 `ValkyrieCompiler` 编译为 IR
- 重构 `ScriptLoader`：适配新的 `ScriptCompiler` 接口
- 更新 `gg-script/Cargo.toml`：添加 `oak-valkyrie` 依赖
- 更新根 `Cargo.toml`：添加 `oak-valkyrie` workspace 依赖

## Impact
- Affected specs: valkyrie-script-integration（已完成，本变更为其演进）、compiler-pipeline-core
- Affected code: `gg-script`（完全重写 lib.rs）、`gg-script/Cargo.toml`、根 `Cargo.toml`
- 不影响 `gg-ir`、`gg-vm`、`gg-runtime-core`、`gg-compiler-script`（这些模块的接口不变）

## ADDED Requirements

### Requirement: oak-valkyrie 依赖集成
系统 SHALL 将 `oak-valkyrie` crate 作为 `gg-script` 的依赖，通过 `ValkyrieBuilder` 将 Valkyrie 源码编译为 `ValkyrieRoot` AST。

#### Scenario: 成功解析 Valkyrie 源码为 AST
- **WHEN** 用户提供有效的 Valkyrie 源码字符串
- **THEN** 系统通过 `ValkyrieBuilder` 生成 `ValkyrieRoot` AST
- **AND** 返回编译成功的结果

#### Scenario: 源码包含语法错误
- **WHEN** 用户提供包含语法错误的 Valkyrie 源码
- **THEN** 系统返回包含诊断信息的错误

### Requirement: Valkyrie AST 到 IR 编译器
系统 SHALL 提供 `ValkyrieCompiler`，将 `ValkyrieRoot` AST 编译为 `IrModule`，支持 Valkyrie 语言的核心子集。

#### Scenario: 编译 micro 函数定义
- **WHEN** `ValkyrieRoot` 包含 `Item::Micro(MicroDefinition)` 节点
- **THEN** 编译器为每个 micro 函数生成 `IrFunction`，包含参数、局部变量和指令序列
- **AND** micro 函数名作为 `IrFunction.name`

#### Scenario: 编译 let 绑定语句
- **WHEN** AST 中存在 `Statement::Let` 节点
- **THEN** 编译器生成表达式求值指令 + `StoreLocal` 指令
- **AND** 局部变量名映射到索引

#### Scenario: 编译表达式语句
- **WHEN** AST 中存在 `Statement::ExprStmt` 节点
- **THEN** 编译器生成表达式求值指令 + `Pop` 指令（若语句以分号结尾）

#### Scenario: 编译二元运算表达式
- **WHEN** AST 中存在 `Expr::Binary` 节点
- **THEN** 编译器递归编译左右操作数，然后生成对应的算术/比较/逻辑 OpCode

#### Scenario: 编译一元运算表达式
- **WHEN** AST 中存在 `Expr::Unary` 节点
- **THEN** 编译器递归编译操作数，然后生成 `Neg` 或 `Not` OpCode

#### Scenario: 编译函数调用表达式
- **WHEN** AST 中存在 `Expr::Call` 节点
- **THEN** 编译器判断被调用者是否为内置宿主函数（spawn_entity、add_component、set_field、get_field、print）
- **AND** 若为内置函数则生成 `HostCall` OpCode
- **AND** 若为用户函数则生成 `LoadConst` + `Call` OpCode

#### Scenario: 编译 if/else 表达式
- **WHEN** AST 中存在 `Expr::If` 节点
- **THEN** 编译器生成条件求值 + `JumpIfFalse` + then 分支 + 可选 else 分支 + `Jump` 指令
- **AND** 正确回填跳转地址

#### Scenario: 编译 return 表达式
- **WHEN** AST 中存在 `Expr::Return` 节点
- **THEN** 编译器生成可选的返回值求值 + `Return` OpCode

#### Scenario: 编译循环表达式
- **WHEN** AST 中存在 `Expr::Loop` 节点
- **THEN** 编译器生成条件循环（while 风格）或无限循环 + `Jump` 指令
- **AND** 支持 break 和 continue

#### Scenario: 编译布尔字面量
- **WHEN** AST 中存在 `Expr::Bool` 节点
- **THEN** 编译器生成 `LoadTrue` 或 `LoadFalse` OpCode

#### Scenario: 编译字符串字面量
- **WHEN** AST 中存在 `Expr::StringLiteral` 节点
- **THEN** 编译器将字符串内容添加到常量池，生成 `LoadConst` OpCode

#### Scenario: 编译标识符引用
- **WHEN** AST 中存在 `Expr::Ident` 节点
- **THEN** 编译器查找局部变量映射，生成 `LoadLocal` OpCode
- **AND** 若变量未找到则生成 `LoadNull`

#### Scenario: 编译括号表达式
- **WHEN** AST 中存在 `Expr::Paren` 节点
- **THEN** 编译器递归编译内部表达式

### Requirement: 重构后的 ScriptCompiler
系统 SHALL 提供重构后的 `ScriptCompiler`，使用 oak-valkyrie 前端 + ValkyrieCompiler 后端的编译管线。

#### Scenario: 编译 Valkyrie 源码为 IR
- **WHEN** 用户调用 `ScriptCompiler::compile(source, module_name)`
- **THEN** 系统通过 `ValkyrieBuilder` 解析源码为 `ValkyrieRoot`
- **AND** 通过 `ValkyrieCompiler` 将 AST 编译为 `IrModule`
- **AND** 返回 `GResult<IrModule>`

### Requirement: 重构后的 ScriptLoader
系统 SHALL 提供重构后的 `ScriptLoader`，保持与原有接口兼容。

#### Scenario: 从文件加载脚本
- **WHEN** 用户调用 `ScriptLoader::load_file(path)`
- **THEN** 系统读取文件内容，通过 `ScriptCompiler` 编译为 `IrModule`
- **AND** 返回 `GResult<IrModule>`

#### Scenario: 从字符串加载脚本
- **WHEN** 用户调用 `ScriptLoader::load_string(source, module_name)`
- **THEN** 系统通过 `ScriptCompiler` 编译为 `IrModule`
- **AND** 返回 `GResult<IrModule>`

## MODIFIED Requirements

### Requirement: gg-script 模块编译管线
gg-script 模块 SHALL 使用 oak-valkyrie 的 ValkyrieBuilder 作为前端解析器，而非自实现的 SimpleLexer + SimpleParser。编译管线变更为：源码 → ValkyrieBuilder → ValkyrieRoot AST → ValkyrieCompiler → IrModule。

## REMOVED Requirements

### Requirement: SimpleLexer 词法分析器
**Reason**: oak-valkyrie 的 ValkyrieLexer 提供了更完整、更健壮的词法分析能力，SimpleLexer 仅支持极小子集
**Migration**: 所有通过 SimpleLexer 的词法分析由 ValkyrieBuilder 内部的 ValkyrieLexer 替代

### Requirement: SimpleParser 递归下降解析器
**Reason**: oak-valkyrie 的 ValkyrieParser + ValkyrieBuilder 提供了完整的语法分析和 AST 构建能力，SimpleParser 仅支持 micro/let/if/表达式
**Migration**: 所有通过 SimpleParser 的解析由 ValkyrieBuilder 替代，AST 到 IR 的转换由新增的 ValkyrieCompiler 承担

### Requirement: SimpleToken 枚举
**Reason**: 随 SimpleLexer 一同移除，oak-valkyrie 有自己的 token 类型体系
**Migration**: 无需迁移，ValkyrieBuilder 直接从源码生成 AST
