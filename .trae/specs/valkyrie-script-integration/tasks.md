# Tasks

- [x] Task 1: 创建 gg-ir 模块，定义中间表示指令集
  - [ ] SubTask 1.1: 创建 `projects/runtime/gg-ir` 目录和 Cargo.toml
  - [ ] SubTask 1.2: 定义 IR 指令枚举（OpCode），包括：常量加载、二元运算、比较运算、跳转、函数调用、实体操作（spawn/despawn）、组件操作（add/get/set）、返回等
  - [ ] SubTask 1.3: 定义 IR 模块结构（IrModule），包含函数列表和常量池
  - [ ] SubTask 1.4: 定义 IR 函数结构（IrFunction），包含参数、局部变量、指令序列
  - [ ] SubTask 1.5: 在根 Cargo.toml 中添加 gg-ir 到 workspace

- [x] Task 2: 创建 gg-vm 模块，实现字节码虚拟机
  - [ ] SubTask 2.1: 创建 `projects/runtime/gg-vm` 目录和 Cargo.toml
  - [ ] SubTask 2.2: 定义字节码格式（ByteCode），将 IR 指令序列化
  - [ ] SubTask 2.3: 实现虚拟机核心（Vm），包含执行栈、调用栈、指令指针
  - [ ] SubTask 2.4: 实现宿主接口（Host），允许 VM 调用引擎 Rust API
  - [ ] SubTask 2.5: 实现基本指令执行：常量加载、算术运算、比较、跳转、函数调用
  - [ ] SubTask 2.6: 实现实体/组件操作指令：spawn_entity、add_component、get_component、set_component
  - [ ] SubTask 2.7: 在根 Cargo.toml 中添加 gg-vm 到 workspace

- [x] Task 3: 创建 gg-script 模块，集成 Valkyrie 语言前端
  - [ ] SubTask 3.1: 创建 `projects/compiler/gg-script` 目录和 Cargo.toml，添加 oak-valkyrie 依赖
  - [ ] SubTask 3.2: 实现 ScriptCompiler，封装 Valkyrie 的 Lexer -> Parser -> Builder 流程
  - [ ] SubTask 3.3: 实现 AST 到 IR 的转换器（AstToIr），将 ValkyrieRoot 转换为 IrModule
  - [ ] SubTask 3.4: 实现脚本加载器（ScriptLoader），支持从文件系统加载 .valkyrie 脚本
  - [ ] SubTask 3.5: 在根 Cargo.toml 中添加 gg-script 到 workspace

- [x] Task 4: 更新 gg-runtime-core，集成脚本执行环境
  - [ ] SubTask 4.1: 在 gg-runtime-core 中添加 gg-script、gg-ir、gg-vm 依赖
  - [ ] SubTask 4.2: 在 Runtime 中添加 ScriptEngine 字段，管理脚本编译和执行
  - [ ] SubTask 4.3: 实现 ScriptEngine，封装脚本编译、VM 创建和执行
  - [ ] SubTask 4.4: 实现宿主接口绑定，将引擎 ECS API 暴露给 VM
  - [ ] SubTask 4.5: 更新游戏循环，在 tick 中调用脚本的 init/update 函数
  - [ ] SubTask 4.6: 实现基本的热更新支持：文件监视和脚本重载

- [x] Task 5: 重写 examples/basic，使用 Valkyrie 脚本
  - [ ] SubTask 5.1: 创建 `examples/basic/scripts/game.valkyrie` 脚本文件，定义 init 和 update 函数
  - [ ] SubTask 5.2: 脚本中实现实体创建和组件添加逻辑
  - [ ] SubTask 5.3: 脚本中实现移动系统逻辑
  - [ ] SubTask 5.4: 重写 `examples/basic/src/main.rs`，改为加载和执行 Valkyrie 脚本
  - [ ] SubTask 5.5: 更新 `examples/basic/Cargo.toml` 依赖

# Task Dependencies
- Task 2 depends on Task 1（gg-vm 需要 gg-ir 的指令定义）
- Task 3 depends on Task 1（gg-script 的 AST 到 IR 转换需要 gg-ir）
- Task 4 depends on Task 2, Task 3（gg-runtime-core 需要 gg-vm 和 gg-script）
- Task 5 depends on Task 4（示例需要完整的脚本执行环境）
