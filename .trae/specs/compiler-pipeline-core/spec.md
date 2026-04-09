# 编译器流水线核心架构 Spec

## Why
按照 roadmap 编译器组本月工作重点，当前项目缺少编译流水线的核心调度架构（`gg-compiler-core`）、可移植字节码格式（`gg-bytecode`）和 AOT 编译后端（`gg-compiler-aot`）。现有的 `gg-ir`/`gg-vm`/`gg-script` 虽已实现基本的脚本编译与执行，但 IR 指令与 VM 执行紧耦合，缺乏流水线编排、多级 IR 优化和可移植字节码序列化能力。需要构建编译器核心架构，将各编译阶段解耦为可组合的转换器，为后续多语言前端、AOT 编译和热更新奠定基础。

## What Changes
- 新增 `gg-compiler-core` 模块：定义 Transformer trait、ArtifactSet、BuildContext、Pipeline DAG 调度器
- 新增 `gg-bytecode` 模块：定义可移植字节码格式（序列化/反序列化）、独立的字节码解释器
- 新增 `gg-compiler-aot` 模块骨架：定义 AOT 后端 trait 和目标平台抽象，预留 Cranelift 集成接口
- 重构 `gg-ir`：将 IR 指令集与 VM 执行解耦，IR 专注于中间表示，VM 从 `gg-bytecode` 读取字节码执行
- 重构 `gg-vm`：改为从 `gg-bytecode` 的可移植字节码格式执行，不再直接消费 `gg-ir` 的 OpCode
- 更新 `gg-script`：编译输出从 `IrModule` 改为经 IR 优化后序列化为 `BytecodeModule`
- 更新 `gg-runtime-core`：适配新的字节码解释器，移除对 `gg-ir` OpCode 的直接依赖
- 更新根 `Cargo.toml`：添加 `gg-compiler-core`、`gg-bytecode`、`gg-compiler-aot` 到 workspace

## Impact
- Affected specs: 运行时层（Runtime Layer）、元编译器架构、脚本执行环境
- Affected code: `gg-ir`、`gg-vm`、`gg-script`、`gg-runtime-core`、根 `Cargo.toml`

## ADDED Requirements

### Requirement: 编译流水线核心（gg-compiler-core）
系统 SHALL 提供编译流水线的核心架构，支持将编译阶段组织为 DAG 图并按依赖顺序调度执行。

#### Scenario: 定义和执行编译流水线
- **WHEN** 开发者定义一组 Transformer 并声明它们之间的依赖关系
- **THEN** 系统构建 DAG 图，按拓扑顺序调度执行每个 Transformer
- **AND** 每个 Transformer 接收 ArtifactSet 输入，产出 ArtifactSet 输出
- **AND** 未变更的 Transformer 可被跳过（增量编译支持）

#### Scenario: Transformer 标准接口
- **WHEN** 开发者实现 Transformer trait
- **THEN** 该 trait 定义为 `fn transform(inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet>`
- **AND** ArtifactSet 是类型化的产物集合，支持按类型键查找
- **AND** BuildContext 提供编译配置、诊断收集和日志能力

#### Scenario: 增量编译
- **WHEN** 源文件发生变更
- **THEN** Pipeline 仅重新执行受影响的 Transformer 节点
- **AND** 未受影响的节点使用缓存的产物结果

### Requirement: 可移植字节码格式（gg-bytecode）
系统 SHALL 定义一种可移植的字节码格式，支持序列化和反序列化，可在不同平台间传输和执行。

#### Scenario: IR 到字节码的序列化
- **WHEN** 系统将 IrModule 编译为字节码
- **THEN** 字节码包含：魔数头、版本号、常量池、函数表、调试符号表
- **AND** 常量池中的值按紧凑二进制格式编码
- **AND** 函数表包含每个函数的名称、参数数量、局部变量数量和指令序列
- **AND** 指令序列使用操作码字节 + 操作数的紧凑编码

#### Scenario: 字节码反序列化
- **WHEN** 系统从二进制数据加载字节码
- **THEN** 系统验证魔数和版本号
- **AND** 正确重建 BytecodeModule 结构
- **AND** 如果格式不匹配则返回错误

#### Scenario: 字节码解释器执行
- **WHEN** 字节码解释器接收到 BytecodeModule
- **THEN** 解释器按指令顺序执行字节码
- **AND** 解释器维护执行栈、调用栈和全局状态
- **AND** 解释器通过 Host trait 与引擎交互

### Requirement: AOT 编译后端骨架（gg-compiler-aot）
系统 SHALL 提供 AOT 编译后端的 trait 定义和目标平台抽象，为后续集成 Cranelift 奠定基础。

#### Scenario: AOT 后端 trait 定义
- **WHEN** 开发者实现 AotBackend trait
- **THEN** 该 trait 定义为 `fn compile(module: &BytecodeModule, target: &TargetPlatform) -> GResult<Vec<u8>>`
- **AND** TargetPlatform 枚举支持：WindowsX64、MacOSArm64、LinuxX64、WebWasm32、AndroidArm64、IOSArm64
- **AND** 返回的 Vec<u8> 为目标平台的原生机器码或 WASM 模块

#### Scenario: AOT 后端注册和查询
- **WHEN** 编译流水线需要执行 AOT 编译
- **THEN** 系统通过 AotBackendRegistry 查找目标平台对应的后端实现
- **AND** 如果没有注册对应平台的后端，返回错误

### Requirement: IR 优化 Pass
系统 SHALL 在 IR 层提供基本的优化 Pass，作为编译流水线中的 Transformer 节点。

#### Scenario: 常量折叠优化
- **WHEN** IR 中存在编译时可计算的常量表达式（如 `LoadConst(1) + LoadConst(2)`）
- **THEN** 优化 Pass 将其折叠为 `LoadConst(3)`
- **AND** 减少运行时计算开销

#### Scenario: 死代码消除
- **WHEN** IR 中存在不可达的代码块（如 Jump 之后的指令、永假条件分支）
- **THEN** 优化 Pass 将其移除
- **AND** 减少字节码体积

## MODIFIED Requirements

### Requirement: 脚本编译输出
脚本编译器 SHALL 将源码编译为 IrModule 后，经过 IR 优化 Pass，最终序列化为 BytecodeModule 输出。原有的直接输出 IrModule 供 VM 执行的方式改为输出 BytecodeModule。

### Requirement: 虚拟机执行
虚拟机 SHALL 从 BytecodeModule 加载和执行字节码，不再直接消费 IrModule 的 OpCode。VM 的 Host trait 和执行语义保持不变。

## REMOVED Requirements

（无移除的需求，所有现有功能均保留但重构实现方式）
