#![warn(missing_docs)]

//! Valkyrie AST 到 IR 编译器
//! 将 oak-valkyrie 生成的 ValkyrieRoot AST 编译为 gg-ir 的 IrModule

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrFunction, IrModule, IrValue, OpCode};
use oak_valkyrie::{
    ast::{Block, MicroDeclaration, Pattern, Statement, StatementNode, StringSegment, TermExpression, ValkyrieRoot},
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
    fn compile_item(&mut self, item: &StatementNode) -> GResult<()> {
        match item {
            StatementNode::Micro(micro) => {
                let func = self.compile_micro(micro)?;
                self.module.add_function(func);
            }
            StatementNode::Namespace(namespace) => {
                for inner_item in &namespace.items {
                    self.compile_item(inner_item)?;
                }
            }
            StatementNode::Let(let_stmt) => {
                let mut instructions = Vec::new();
                self.compile_statement(&Statement::Let((**let_stmt).clone()), &mut instructions)?;
            }
            StatementNode::ExprStmt(expr_stmt) => {
                let mut instructions = Vec::new();
                self.compile_statement(&Statement::ExprStmt((**expr_stmt).clone()), &mut instructions)?;
            }
            StatementNode::Shader(shader) => {
                let _ = shader;
            }
            _ => {}
        }
        Ok(())
    }

    /// 编译 micro 函数定义
    ///
    /// 为每个 micro 函数生成 IrFunction，包含参数、局部变量和指令序列。
    fn compile_micro(&mut self, micro: &MicroDeclaration) -> GResult<IrFunction> {
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
            Statement::Let(let_stmt) => {
                self.compile_expr(&let_stmt.expr, instructions)?;
                let var_name = match &let_stmt.pattern {
                    Pattern::Variable(var) => var.name.name.clone(),
                    Pattern::Wildcard(_) => {
                        instructions.push(OpCode::Pop);
                        return Ok(());
                    }
                    _ => {
                        return Err(GError { kind: GErrorKind::Runtime, message: "Unsupported let pattern type".to_string() });
                    }
                };
                let idx = self.next_local;
                self.locals.insert(var_name, idx);
                self.next_local += 1;
                instructions.push(OpCode::StoreLocal(idx));
            }
            Statement::ExprStmt(expr_stmt) => {
                self.compile_expr(&expr_stmt.expr, instructions)?;
                if expr_stmt.semi {
                    instructions.push(OpCode::Pop);
                }
            }
        }
        Ok(())
    }

    /// 编译表达式
    fn compile_expr(&mut self, expr: &TermExpression, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match expr {
            TermExpression::NamePath(name_path) => {
                if name_path.parts.len() == 1 {
                    let first = &name_path.parts[0];
                    match self.locals.get(&first.name) {
                        Some(&idx) => {
                            instructions.push(OpCode::LoadLocal(idx));
                        }
                        None => {
                            return Err(GError { kind: GErrorKind::Runtime, message: format!("Undefined variable: {}", first.name) });
                        }
                    }
                } else if let Some(first) = name_path.parts.first() {
                    match self.locals.get(&first.name) {
                        Some(&idx) => {
                            instructions.push(OpCode::LoadLocal(idx));
                        }
                        None => {
                            return Err(GError { kind: GErrorKind::Runtime, message: format!("Undefined variable: {}", first.name) });
                        }
                    }
                } else {
                    instructions.push(OpCode::LoadNull);
                }
            }

            TermExpression::Bool { value, .. } => {
                if *value {
                    instructions.push(OpCode::LoadTrue);
                } else {
                    instructions.push(OpCode::LoadFalse);
                }
            }

            TermExpression::StringLiteral(string_literal) => {
                let has_interpolation = string_literal.segments.iter().any(|seg| {
                    matches!(seg, StringSegment::Interpolation(_))
                });

                if !has_interpolation {
                    let content: String = string_literal.segments.iter()
                        .filter_map(|seg| match seg {
                            StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                            _ => None,
                        })
                        .collect();
                    let idx = self.module.add_constant(IrValue::String(content));
                    instructions.push(OpCode::LoadConst(idx));
                } else {
                    let mut concat_count = 0;
                    for seg in &string_literal.segments {
                        match seg {
                            StringSegment::Text(text_seg) => {
                                if !text_seg.content.is_empty() {
                                    let idx = self.module.add_constant(IrValue::String(text_seg.content.clone()));
                                    instructions.push(OpCode::LoadConst(idx));
                                    concat_count += 1;
                                }
                            }
                            StringSegment::Interpolation(interp_seg) => {
                                self.compile_expr(&interp_seg.expr, instructions)?;
                                concat_count += 1;
                            }
                        }
                    }
                    if concat_count > 0 {
                        instructions.push(OpCode::StringConcat(concat_count));
                    } else {
                        let idx = self.module.add_constant(IrValue::String(String::new()));
                        instructions.push(OpCode::LoadConst(idx));
                    }
                }
            }

            TermExpression::Binary(node) => {
                self.compile_expr(&node.lhs, instructions)?;
                self.compile_expr(&node.rhs, instructions)?;
                let opcode = self.binary_op_to_opcode(&node.operator)?;
                instructions.push(opcode);
            }

            TermExpression::Unary(node) => {
                self.compile_expr(&node.base, instructions)?;
                let opcode = self.unary_op_to_opcode(&node.operator)?;
                instructions.push(opcode);
            }

            TermExpression::ApplyCall { callee, args, .. } => {
                if let TermExpression::NamePath(name_path) = callee.as_ref() {
                    if name_path.parts.len() == 1 {
                        let name = name_path.parts[0].name.clone();
                        if BUILTIN_FUNCTIONS.contains(&name.as_str()) {
                            for arg in args {
                                self.compile_expr(arg, instructions)?;
                            }
                            instructions.push(OpCode::HostCall(name, args.len()));
                            return Ok(());
                        }
                    }
                }

                for arg in args {
                    self.compile_expr(arg, instructions)?;
                }

                if let TermExpression::NamePath(name_path) = callee.as_ref() {
                    if name_path.parts.len() == 1 {
                        let string_idx = self.module.add_constant(IrValue::String(name_path.parts[0].name.clone()));
                        instructions.push(OpCode::LoadConst(string_idx));
                    } else {
                        self.compile_expr(callee, instructions)?;
                        instructions.push(OpCode::LoadNull);
                    }
                } else {
                    self.compile_expr(callee, instructions)?;
                    instructions.push(OpCode::LoadNull);
                }

                instructions.push(OpCode::Call(args.len()));
            }

            TermExpression::DotCall { receiver, field, .. } => {
                self.compile_expr(receiver, instructions)?;
                instructions.push(OpCode::GetField(field.name.clone()));
            }

            TermExpression::Index { receiver, index, .. } => {
                self.compile_expr(receiver, instructions)?;
                self.compile_expr(index, instructions)?;
                instructions.push(OpCode::GetIndex);
            }

            TermExpression::Object { fields, .. } => {
                for (_name, value_expr) in fields {
                    if let Some(expr) = value_expr {
                        self.compile_expr(expr, instructions)?;
                    } else {
                        instructions.push(OpCode::LoadNull);
                    }
                }
                instructions.push(OpCode::NewObject(fields.len()));
            }

            TermExpression::Paren { expr, .. } => {
                self.compile_expr(expr, instructions)?;
            }

            TermExpression::If { condition, then_branch, else_branch, .. } => {
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
                } else {
                    instructions[then_jump] = OpCode::JumpIfFalse(instructions.len());
                }
            }

            TermExpression::Match { scrutinee, arms, .. } => {
                self.compile_expr(scrutinee, instructions)?;
                let scrutinee_local = self.next_local;
                self.locals.insert(format!("__match_scrutinee_{}", scrutinee_local), scrutinee_local);
                self.next_local += 1;
                instructions.push(OpCode::StoreLocal(scrutinee_local));

                let mut end_jumps = Vec::new();

                for (i, arm) in arms.iter().enumerate() {
                    let is_last = i == arms.len() - 1;
                    let is_wildcard = self.is_wildcard_pattern(&arm.pattern);

                    if is_wildcard {
                        self.compile_expr(&arm.body, instructions)?;
                    } else {
                        instructions.push(OpCode::LoadLocal(scrutinee_local));
                        self.compile_match_pattern(&arm.pattern, instructions)?;
                        instructions.push(OpCode::Eq);

                        let jump_if_false = instructions.len();
                        instructions.push(OpCode::JumpIfFalse(0));

                        self.compile_expr(&arm.body, instructions)?;

                        let end_jump = instructions.len();
                        instructions.push(OpCode::Jump(0));
                        end_jumps.push(end_jump);

                        instructions[jump_if_false] = OpCode::JumpIfFalse(instructions.len());
                    }

                    let _ = is_last;
                }

                let end_addr = instructions.len();
                for jump_addr in end_jumps {
                    instructions[jump_addr] = OpCode::Jump(end_addr);
                }
            }

            TermExpression::Return(ret) => {
                if let Some(return_expr) = ret.base.as_ref() {
                    self.compile_expr(return_expr, instructions)?;
                } else {
                    instructions.push(OpCode::LoadNull);
                }
                instructions.push(OpCode::Return);
            }

            TermExpression::Loop { condition, body, .. } => {
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
                } else {
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

            TermExpression::Break(_) => {
                if let Some(ctx) = self.loop_stack.last_mut() {
                    let jump_addr = instructions.len();
                    instructions.push(OpCode::Jump(0));
                    ctx.break_jumps.push(jump_addr);
                } else {
                    return Err(GError { kind: GErrorKind::Runtime, message: "Break outside of loop".to_string() });
                }
            }

            TermExpression::Continue(_) => {
                if let Some(ctx) = self.loop_stack.last() {
                    instructions.push(OpCode::Jump(ctx.loop_start));
                } else {
                    return Err(GError { kind: GErrorKind::Runtime, message: "Continue outside of loop".to_string() });
                }
            }

            TermExpression::Block(block) => {
                self.compile_block(block, instructions)?;
            }

            _ => {
                instructions.push(OpCode::LoadNull);
            }
        }
        Ok(())
    }

    /// 检查模式是否为通配符（匹配所有值）
    fn is_wildcard_pattern(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard(_) => true,
            Pattern::Variable(var) => var.name.name.starts_with('_'),
            _ => false,
        }
    }

    /// 将匹配模式编译为用于比较的值
    fn compile_match_pattern(&mut self, pattern: &Pattern, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match pattern {
            Pattern::Variable(var) => {
                if var.name.name.starts_with('_') {
                    instructions.push(OpCode::LoadTrue);
                } else {
                    let idx = self.module.add_constant(IrValue::String(var.name.name.clone()));
                    instructions.push(OpCode::LoadConst(idx));
                }
            }
            Pattern::Wildcard(_) => {
                instructions.push(OpCode::LoadTrue);
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
