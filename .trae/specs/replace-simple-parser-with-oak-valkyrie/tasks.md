# Tasks

- [ ] Task 1: 更新依赖配置，添加 oak-valkyrie 到 workspace
  - [ ] SubTask 1.1: 在根 `Cargo.toml` 的 `[workspace.dependencies]` 中添加 `oak-valkyrie = { path = "../../oaks/examples/oak-valkyrie" }`
  - [ ] SubTask 1.2: 在 `gg-script/Cargo.toml` 中添加 `oak-valkyrie = { workspace = true }` 依赖
  - [ ] SubTask 1.3: 验证 `cargo check -p gg-script` 通过

- [ ] Task 2: 实现 ValkyrieCompiler，将 ValkyrieRoot AST 编译为 IrModule
  - [ ] SubTask 2.1: 创建 `gg-script/src/compiler.rs`，定义 `ValkyrieCompiler` 结构体和编译上下文（局部变量映射、下一个局部变量索引、常量池等）
  - [ ] SubTask 2.2: 实现 `compile_module(root: &ValkyrieRoot, module_name: &str) -> GResult<IrModule>`，遍历 `ValkyrieRoot.items`，对每个 `Item::Micro` 调用 `compile_micro`
  - [ ] SubTask 2.3: 实现 `compile_micro(micro: &MicroDefinition) -> GResult<IrFunction>`，处理参数、遍历 body 语句、生成指令序列
  - [ ] SubTask 2.4: 实现 `compile_statement(stmt: &Statement, instructions: &mut Vec<OpCode>)`，处理 `Statement::Let` 和 `Statement::ExprStmt`
  - [ ] SubTask 2.5: 实现 `compile_expr(expr: &Expr, instructions: &mut Vec<OpCode>)`，处理所有表达式类型：Ident、Path、StringLiteral、Bool、Binary、Unary、Call、Field、Paren、If、Return、Loop、Break、Continue、Block
  - [ ] SubTask 2.6: 实现内置宿主函数识别逻辑（spawn_entity、add_component、set_field、get_field、print），对内置函数生成 `HostCall` OpCode
  - [ ] SubTask 2.7: 为所有 public 结构体、方法、字段添加文档注释

- [ ] Task 3: 重构 gg-script lib.rs，移除 SimpleLexer/SimpleParser，使用 oak-valkyrie 前端
  - [ ] SubTask 3.1: 移除 `SimpleToken` 枚举、`KEYWORDS` 常量、`BUILTIN_FUNCTIONS` 常量
  - [ ] SubTask 3.2: 移除 `SimpleLexer` 结构体及其实现
  - [ ] SubTask 3.3: 移除 `SimpleParser` 结构体及其实现
  - [ ] SubTask 3.4: 重构 `ScriptCompiler`：使用 `ValkyrieBuilder` 解析源码为 `ValkyrieRoot`，再通过 `ValkyrieCompiler` 编译为 `IrModule`
  - [ ] SubTask 3.5: 重构 `ScriptLoader`：适配新的 `ScriptCompiler` 接口（接口签名不变）
  - [ ] SubTask 3.6: 添加 `pub mod compiler;` 导出 ValkyrieCompiler
  - [ ] SubTask 3.7: 更新模块级文档注释
  - [ ] SubTask 3.8: 移除旧的测试代码，为新的编译管线编写测试

- [ ] Task 4: 验证端到端编译管线
  - [ ] SubTask 4.1: 验证 `cargo check -p gg-script` 通过
  - [ ] SubTask 4.2: 验证 `cargo check -p gg-runtime-core` 通过（gg-runtime-core 依赖 gg-script）
  - [ ] SubTask 4.3: 验证 `cargo check -p gg-game-engine` 全 workspace 通过
  - [ ] SubTask 4.4: 验证 `cargo test -p gg-script` 通过

# Task Dependencies
- Task 2 depends on Task 1（ValkyrieCompiler 需要 oak-valkyrie 的 AST 类型）
- Task 3 depends on Task 1, Task 2（重构 lib.rs 需要依赖和编译器就绪）
- Task 4 depends on Task 3（验证需要完整的重构完成）
