use std::collections::HashMap;

use gg_ir::{EntryPoint, IrModule, OpCode, TargetPlatform};

/// 内置宿主函数列表，编译为 HostCall 指令
pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "spawn_entity",
    "add_component",
    "set_field",
    "get_field",
    "print",
    "db_query",
    "db_insert",
    "db_update",
    "db_delete",
    "db_find",
    "db_count",
];

/// 循环上下文，用于 break/continue 的跳转地址回填
pub struct LoopContext {
    /// 循环起始地址（用于 continue 跳回）
    pub loop_start: usize,
    /// 需要回填的 break 跳转地址列表（循环结束时跳转到循环后）
    pub break_jumps: Vec<usize>,
}

/// Valkyrie AST 到 IR 编译器
///
/// 将 ValkyrieRoot AST 编译为 IrModule，支持 Valkyrie 语言的核心子集：
/// - micro 函数定义
/// - let 绑定语句
/// - 表达式语句
/// - 二元/一元运算
/// - 函数调用（内置宿主函数 + 用户函数）
/// - if/else 条件表达式
/// - return 表达式
/// - loop 循环（含 break/continue）
/// - 布尔、字符串字面量和标识符引用
pub struct ValkyrieCompiler {
    /// IR 模块
    pub module: IrModule,
    /// 局部变量名到索引的映射
    pub locals: HashMap<String, usize>,
    /// 局部变量名称列表，按索引顺序排列，用于调试
    pub local_names: Vec<String>,
    /// 下一个可用的局部变量索引
    pub next_local: usize,
    /// 循环上下文栈
    pub loop_stack: Vec<LoopContext>,
    /// 入口点列表
    pub entry_points: Vec<EntryPoint>,
    /// 目标平台
    pub target_platform: Option<TargetPlatform>,
    /// 模块初始化指令
    pub module_init_instructions: Vec<OpCode>,
}
