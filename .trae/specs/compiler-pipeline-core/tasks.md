# Tasks

- [x] Task 1: 创建 gg-compiler-core 模块，定义编译流水线核心架构
  - [x] SubTask 1.1: 创建 `projects/compiler/gg-compiler-core` 目录和 Cargo.toml
  - [x] SubTask 1.2: 定义 ArtifactKey 和 ArtifactSet 类型，支持类型化的编译产物集合
  - [x] SubTask 1.3: 定义 BuildContext 结构，提供编译配置、诊断收集和日志能力
  - [x] SubTask 1.4: 定义 Transformer trait：`fn transform(inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet>`
  - [x] SubTask 1.5: 定义 Pipeline 结构，支持添加 Transformer 节点和声明依赖关系
  - [x] SubTask 1.6: 实现 Pipeline 的 DAG 拓扑排序和调度执行
  - [x] SubTask 1.7: 实现增量编译支持：基于 ArtifactSet 哈希缓存，跳过未变更节点
  - [x] SubTask 1.8: 在根 Cargo.toml 中添加 gg-compiler-core 到 workspace

- [x] Task 2: 创建 gg-bytecode 模块，定义可移植字节码格式和解释器
  - [x] SubTask 2.1: 创建 `projects/runtime/gg-bytecode` 目录和 Cargo.toml
  - [x] SubTask 2.2: 定义字节码二进制格式：魔数头（0x47474243 = "GGBC"）、版本号（u32）、常量池、函数表、调试符号表
  - [x] SubTask 2.3: 定义 BytecodeModule 结构，包含反序列化后的字节码模块表示
  - [x] SubTask 2.4: 定义 BytecodeValue 枚举，与 IrValue 对应但面向字节码的值类型
  - [x] SubTask 2.5: 定义 BytecodeOp 枚举，紧凑的操作码表示（操作码字节 + 操作数）
  - [x] SubTask 2.6: 实现 BytecodeWriter，将 IrModule 序列化为字节码二进制格式
  - [x] SubTask 2.7: 实现 BytecodeReader，从二进制数据反序列化为 BytecodeModule
  - [x] SubTask 2.8: 实现 BytecodeInterpreter，从 BytecodeModule 执行字节码，维护执行栈和调用栈
  - [x] SubTask 2.9: BytecodeInterpreter 通过 Host trait 与引擎交互（复用 gg-vm 的 Host trait）
  - [x] SubTask 2.10: 在根 Cargo.toml 中添加 gg-bytecode 到 workspace

- [x] Task 3: 创建 gg-compiler-aot 模块骨架，定义 AOT 后端 trait 和目标平台抽象
  - [x] SubTask 3.1: 创建 `projects/compiler/gg-compiler-aot` 目录和 Cargo.toml
  - [x] SubTask 3.2: 定义 TargetPlatform 枚举：WindowsX64、MacOSArm64、LinuxX64、WebWasm32、AndroidArm64、IOSArm64
  - [x] SubTask 3.3: 定义 AotBackend trait：`fn compile(module: &BytecodeModule, target: &TargetPlatform) -> GResult<Vec<u8>>`
  - [x] SubTask 3.4: 定义 AotBackendRegistry，支持注册和查询目标平台对应的 AOT 后端
  - [x] SubTask 3.5: 在根 Cargo.toml 中添加 gg-compiler-aot 到 workspace

- [x] Task 4: 在 gg-ir 中实现 IR 优化 Pass
  - [x] SubTask 4.1: 定义 IrPass trait：`fn run(module: &mut IrModule) -> GResult<bool>`（返回是否做了修改）
  - [x] SubTask 4.2: 实现常量折叠 Pass：识别编译时可计算的常量表达式并折叠
  - [x] SubTask 4.3: 实现死代码消除 Pass：移除不可达的代码块
  - [x] SubTask 4.4: 实现 IrOptimizer，按顺序执行多个 IrPass
  - [x] SubTask 4.5: 为 IrPass 实现 Transformer trait 适配器，使其可嵌入 Pipeline

- [x] Task 5: 重构 gg-vm，改为从 gg-bytecode 执行
  - [x] SubTask 5.1: 重构 Vm 结构体，改为从 BytecodeModule 加载和执行
  - [x] SubTask 5.2: 保留 Host trait 定义（移至 gg-bytecode 或保留在 gg-vm 中重新导出）
  - [x] SubTask 5.3: 保留 VmResult、CallFrame 等公共 API 不变
  - [x] SubTask 5.4: 更新 gg-vm 的 Cargo.toml 依赖，添加 gg-bytecode，移除对 gg-ir OpCode 的直接消费

- [x] Task 6: 更新 gg-script，编译输出改为 BytecodeModule
  - [x] SubTask 6.1: 在 ScriptCompiler::compile 中增加 IR 优化步骤
  - [x] SubTask 6.2: 在 ScriptCompiler::compile 中增加 IR 到字节码的序列化步骤
  - [x] SubTask 6.3: 更新 ScriptLoader 的返回类型和加载逻辑
  - [x] SubTask 6.4: 更新 gg-script 的 Cargo.toml 依赖，添加 gg-bytecode、gg-compiler-core

- [x] Task 7: 更新 gg-runtime-core，适配新的字节码解释器
  - [x] SubTask 7.1: 更新 ScriptEngine，使用 BytecodeModule 替代 IrModule
  - [x] SubTask 7.2: 更新 EngineHost，适配新的 Host trait 位置
  - [x] SubTask 7.3: 更新 Runtime 的脚本加载和执行流程
  - [x] SubTask 7.4: 更新 gg-runtime-core 的 Cargo.toml 依赖

- [x] Task 8: 编写测试验证编译流水线端到端工作
  - [x] SubTask 8.1: 在 gg-compiler-core 中测试 Pipeline DAG 调度
  - [x] SubTask 8.2: 在 gg-bytecode 中测试字节码序列化/反序列化往返
  - [x] SubTask 8.3: 在 gg-bytecode 中测试字节码解释器执行
  - [x] SubTask 8.4: 在 gg-ir 中测试 IR 优化 Pass
  - [x] SubTask 8.5: 端到端测试：脚本源码 → IR → 优化 → 字节码 → 执行

# Task Dependencies
- Task 2 depends on Task 1（gg-bytecode 的 BytecodeWriter 需要 gg-ir 的 IrModule 定义）
- Task 3 depends on Task 2（gg-compiler-aot 依赖 gg-bytecode 的 BytecodeModule）
- Task 4 depends on Task 1（IrPass 的 Transformer 适配器依赖 gg-compiler-core 的 Transformer trait）
- Task 5 depends on Task 2（gg-vm 重构依赖 gg-bytecode 的 BytecodeModule 和 BytecodeInterpreter）
- Task 6 depends on Task 2, Task 4（gg-script 需要 gg-bytecode 的序列化和 gg-ir 的优化 Pass）
- Task 7 depends on Task 5, Task 6（gg-runtime-core 依赖重构后的 gg-vm 和 gg-script）
- Task 8 depends on Task 7（端到端测试需要所有模块就绪）
