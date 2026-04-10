#![warn(missing_docs)]

//! Valkyrie AST 到 IR 编译器
//! 将 oak-valkyrie 生成的 ValkyrieRoot AST 编译为 gg-ir 的 IrModule

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};
use oak_valkyrie::{
    ast::{Block, Expr, Item, MicroDefinition, Statement, ValkyrieRoot},
    lexer::token_type::ValkyrieTokenType,
};

/// 内置宿主函数列表，编译为 HostCall 指令
const BUILTIN_FUNCTIONS: &[&str] = &["spawn_entity", "add_component", "set_field", "get_field", "print"];

/// 循环上下文，用于 break/continue 的跳转地址回填
struct LoopContext {
    /// 循环起始地址（用于 continue 跳回）
    loop_start: usize,
    /// 需要回填的 break 跳转地址列表（循环结束时跳转到循环后）
    break_jumps: Vec<usize>,
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
    module: IrModule,
    /// 局部变量名到索引的映射
    locals: HashMap<String, usize>,
    /// 下一个可用的局部变量索引
    next_local: usize,
    /// 循环上下文栈
    loop_stack: Vec<LoopContext>,
}

impl ValkyrieCompiler {
    /// 创建新的编译器
    pub fn new(module_name: &str) -> Self {
        Self { module: IrModule::new(module_name), locals: HashMap::new(), next_local: 0, loop_stack: Vec::new() }
    }

    /// 编译 ValkyrieRoot AST 为 IrModule
    ///
    /// 遍历 AST 中的顶层 Item，对每个 micro 函数定义编译为 IrFunction。
    pub fn compile(mut self, root: &ValkyrieRoot, module_name: &str) -> GResult<IrModule> {
        self.module = IrModule::new(module_name);
        for item in &root.items {
            self.compile_item(item)?;
        }
        Ok(self.module)
    }

    /// 编译顶层 Item
    fn compile_item(&mut self, item: &Item) -> GResult<()> {
        match item {
            Item::Micro(micro) => {
                let func = self.compile_micro(micro)?;
                self.module.add_function(func);
            }
            Item::Namespace(namespace) => {
                for inner_item in &namespace.items {
                    self.compile_item(inner_item)?;
                }
            }
            Item::Statement(stmt) => {
                let mut instructions = Vec::new();
                self.compile_statement(stmt, &mut instructions)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// 编译 micro 函数定义
    ///
    /// 为每个 micro 函数生成 IrFunction，包含参数、局部变量和指令序列。
    fn compile_micro(&mut self, micro: &MicroDefinition) -> GResult<IrFunction> {
        self.locals.clear();
        self.next_local = 0;
        self.loop_stack.clear();

        let param_count = micro.params.len();

        for param in &micro.params {
            self.locals.insert(param.name.name.clone(), self.next_local);
            self.next_local += 1;
        }

        let mut instructions = Vec::new();
        self.compile_block(&micro.body, &mut instructions)?;

        instructions.push(OpCode::LoadNull);
        instructions.push(OpCode::Return);

        Ok(IrFunction { name: micro.name.name.clone(), param_count, local_count: self.next_local, instructions })
    }

    /// 编译语句块
    fn compile_block(&mut self, block: &Block, instructions: &mut Vec<OpCode>) -> GResult<()> {
        for stmt in &block.statements {
            self.compile_statement(stmt, instructions)?;
        }
        Ok(())
    }

    /// 编译语句
    fn compile_statement(&mut self, stmt: &Statement, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match stmt {
            Statement::Let { pattern, expr, .. } => {
                self.compile_expr(expr, instructions)?;
                let var_name = match pattern {
                    oak_valkyrie::ast::Pattern::Variable { name, .. } => name.name.clone(),
                    oak_valkyrie::ast::Pattern::Wildcard { .. } => {
                        instructions.push(OpCode::Pop);
                        return Ok(());
                    }
                    _ => {
                        return Err(GError { kind: GErrorKind::Runtime, message: format!("Unsupported let pattern type") });
                    }
                };
                let idx = self.next_local;
                self.locals.insert(var_name, idx);
                self.next_local += 1;
                instructions.push(OpCode::StoreLocal(idx));
            }
            Statement::ExprStmt { expr, semi, .. } => {
                self.compile_expr(expr, instructions)?;
                if *semi {
                    instructions.push(OpCode::Pop);
                }
            }
        }
        Ok(())
    }

    /// 编译表达式
    fn compile_expr(&mut self, expr: &Expr, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match expr {
            Expr::Ident(ident) => match self.locals.get(&ident.name) {
                Some(&idx) => {
                    instructions.push(OpCode::LoadLocal(idx));
                }
                None => {
                    instructions.push(OpCode::LoadNull);
                }
            },

            Expr::Path(name_path) => {
                if let Some(first) = name_path.parts.first() {
                    match self.locals.get(&first.name) {
                        Some(&idx) => {
                            instructions.push(OpCode::LoadLocal(idx));
                        }
                        None => {
                            instructions.push(OpCode::LoadNull);
                        }
                    }
                }
                else {
                    instructions.push(OpCode::LoadNull);
                }
            }

            Expr::Bool { value, .. } => {
                if *value {
                    instructions.push(OpCode::LoadTrue);
                }
                else {
                    instructions.push(OpCode::LoadFalse);
                }
            }

            Expr::StringLiteral(string_literal) => {
                let content = string_literal
                    .segments
                    .iter()
                    .map(|seg| match seg {
                        oak_valkyrie::ast::StringSegment::Text { content, .. } => content.as_str(),
                        oak_valkyrie::ast::StringSegment::Interpolation { .. } => "",
                    })
                    .collect::<String>();
                if content.parse::<i64>().is_ok() {
                    let value = content.parse::<i64>().unwrap();
                    let idx = self.module.add_constant(IrValue::Int(value));
                    instructions.push(OpCode::LoadConst(idx));
                }
                else if content.parse::<f64>().is_ok() {
                    let value = content.parse::<f64>().unwrap();
                    let idx = self.module.add_constant(IrValue::Float(value));
                    instructions.push(OpCode::LoadConst(idx));
                }
                else {
                    let idx = self.module.add_constant(IrValue::String(content));
                    instructions.push(OpCode::LoadConst(idx));
                }
            }

            Expr::Binary { left, op, right, .. } => {
                self.compile_expr(left, instructions)?;
                self.compile_expr(right, instructions)?;
                let opcode = self.binary_op_to_opcode(op)?;
                instructions.push(opcode);
            }

            Expr::Unary { op, expr, .. } => {
                self.compile_expr(expr, instructions)?;
                let opcode = self.unary_op_to_opcode(op)?;
                instructions.push(opcode);
            }

            Expr::Call { callee, args, .. } => {
                if let Expr::Ident(ident) = callee.as_ref() {
                    let name = ident.name.clone();
                    if BUILTIN_FUNCTIONS.contains(&name.as_str()) {
                        for arg in args {
                            self.compile_expr(arg, instructions)?;
                        }
                        instructions.push(OpCode::HostCall(name, args.len()));
                        return Ok(());
                    }
                }

                for arg in args {
                    self.compile_expr(arg, instructions)?;
                }

                if let Expr::Ident(ident) = callee.as_ref() {
                    let string_idx = self.module.add_constant(IrValue::String(ident.name.clone()));
                    instructions.push(OpCode::LoadConst(string_idx));
                }
                else {
                    self.compile_expr(callee, instructions)?;
                    instructions.push(OpCode::LoadNull);
                }

                instructions.push(OpCode::Call(args.len()));
            }

            Expr::Paren { expr, .. } => {
                self.compile_expr(expr, instructions)?;
            }

            Expr::If { condition, then_branch, else_branch, .. } => {
                self.compile_expr(condition, instructions)?;

                let then_jump = instructions.len();
                instructions.push(OpCode::JumpIfFalse(0));

                self.compile_block(then_branch, instructions)?;

                if let Some(else_branch) = else_branch {
                    let else_jump = instructions.len();
                    instructions.push(OpCode::Jump(0));

                    instructions[then_jump] = OpCode::JumpIfFalse(instructions.len());

                    self.compile_block(else_branch, instructions)?;

                    instructions[else_jump] = OpCode::Jump(instructions.len());
                }
                else {
                    instructions[then_jump] = OpCode::JumpIfFalse(instructions.len());
                }
            }

            Expr::Return { expr, .. } => {
                if let Some(return_expr) = expr {
                    self.compile_expr(return_expr, instructions)?;
                }
                else {
                    instructions.push(OpCode::LoadNull);
                }
                instructions.push(OpCode::Return);
            }

            Expr::Loop { condition, body, .. } => {
                let loop_start = instructions.len();

                let loop_ctx = LoopContext { loop_start, break_jumps: Vec::new() };
                self.loop_stack.push(loop_ctx);

                if let Some(cond) = condition {
                    self.compile_expr(cond, instructions)?;
                    let loop_exit_jump = instructions.len();
                    instructions.push(OpCode::JumpIfFalse(0));

                    self.compile_block(body, instructions)?;
                    instructions.push(OpCode::Jump(loop_start));

                    let loop_end = instructions.len();
                    instructions[loop_exit_jump] = OpCode::JumpIfFalse(loop_end);

                    if let Some(ctx) = self.loop_stack.pop() {
                        for jump_addr in ctx.break_jumps {
                            instructions[jump_addr] = OpCode::Jump(loop_end);
                        }
                    }
                }
                else {
                    self.compile_block(body, instructions)?;
                    instructions.push(OpCode::Jump(loop_start));

                    let loop_end = instructions.len();
                    if let Some(ctx) = self.loop_stack.pop() {
                        for jump_addr in ctx.break_jumps {
                            instructions[jump_addr] = OpCode::Jump(loop_end);
                        }
                    }
                }
            }

            Expr::Break { .. } => {
                if let Some(ctx) = self.loop_stack.last_mut() {
                    let jump_addr = instructions.len();
                    instructions.push(OpCode::Jump(0));
                    ctx.break_jumps.push(jump_addr);
                }
                else {
                    return Err(GError { kind: GErrorKind::Runtime, message: "Break outside of loop".to_string() });
                }
            }

            Expr::Continue { .. } => {
                if let Some(ctx) = self.loop_stack.last() {
                    instructions.push(OpCode::Jump(ctx.loop_start));
                }
                else {
                    return Err(GError { kind: GErrorKind::Runtime, message: "Continue outside of loop".to_string() });
                }
            }

            Expr::Block(block) => {
                self.compile_block(block, instructions)?;
            }

            _ => {
                instructions.push(OpCode::LoadNull);
            }
        }
        Ok(())
    }

    /// 将二元运算符 token 转换为 OpCode
    fn binary_op_to_opcode(&self, op: &ValkyrieTokenType) -> GResult<OpCode> {
        match op {
            ValkyrieTokenType::Plus => Ok(OpCode::Add),
            ValkyrieTokenType::Minus => Ok(OpCode::Sub),
            ValkyrieTokenType::Star => Ok(OpCode::Mul),
            ValkyrieTokenType::Slash => Ok(OpCode::Div),
            ValkyrieTokenType::Percent => Ok(OpCode::Mod),
            ValkyrieTokenType::EqEq => Ok(OpCode::Eq),
            ValkyrieTokenType::NotEq => Ok(OpCode::Ne),
            ValkyrieTokenType::LessThan => Ok(OpCode::Lt),
            ValkyrieTokenType::LessEq => Ok(OpCode::Le),
            ValkyrieTokenType::GreaterThan => Ok(OpCode::Gt),
            ValkyrieTokenType::GreaterEq => Ok(OpCode::Ge),
            ValkyrieTokenType::AndAnd => Ok(OpCode::And),
            ValkyrieTokenType::OrOr => Ok(OpCode::Or),
            _ => Err(GError { kind: GErrorKind::Runtime, message: format!("Unsupported binary operator: {:?}", op) }),
        }
    }

    /// 将一元运算符 token 转换为 OpCode
    fn unary_op_to_opcode(&self, op: &ValkyrieTokenType) -> GResult<OpCode> {
        match op {
            ValkyrieTokenType::Minus => Ok(OpCode::Neg),
            ValkyrieTokenType::Bang => Ok(OpCode::Not),
            _ => Err(GError { kind: GErrorKind::Runtime, message: format!("Unsupported unary operator: {:?}", op) }),
        }
    }
}
