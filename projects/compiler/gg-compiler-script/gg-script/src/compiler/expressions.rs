use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{IrValue, OpCode};
use oak_valkyrie::{ast::{Pattern, StringSegment, TermExpression}, lexer::token_type::ValkyrieTokenType};

use super::{ValkyrieCompiler};

impl ValkyrieCompiler {
    /// 编译表达式
    pub(crate) fn compile_expr(&mut self, expr: &TermExpression, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match expr {
            TermExpression::NamePath(name_path) => {
                if name_path.parts.len() == 1 {
                    let first = &name_path.parts[0];
                    match self.locals.get(&first.name) {
                        Some(&idx) => {
                            instructions.push(OpCode::LoadLocal(idx));
                        }
                        None => {
                            return Err(GError {
                                kind: GErrorKind::Runtime,
                                message: format!("Undefined variable: {}", first.name),
                            });
                        }
                    }
                }
                else if let Some(first) = name_path.parts.first() {
                    match self.locals.get(&first.name) {
                        Some(&idx) => {
                            instructions.push(OpCode::LoadLocal(idx));
                        }
                        None => {
                            return Err(GError {
                                kind: GErrorKind::Runtime,
                                message: format!("Undefined variable: {}", first.name),
                            });
                        }
                    }
                }
                else {
                    instructions.push(OpCode::LoadNull);
                }
            }

            TermExpression::Bool { value, .. } => {
                if *value {
                    instructions.push(OpCode::LoadTrue);
                }
                else {
                    instructions.push(OpCode::LoadFalse);
                }
            }

            TermExpression::StringLiteral(string_literal) => {
                let has_interpolation =
                    string_literal.segments.iter().any(|seg| matches!(seg, StringSegment::Interpolation(_)));

                if !has_interpolation {
                    let content: String = string_literal
                        .segments
                        .iter()
                        .filter_map(|seg| match seg {
                            StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                            _ => None,
                        })
                        .collect();
                    let idx = self.module.add_constant(IrValue::String(content));
                    instructions.push(OpCode::LoadConst(idx));
                }
                else {
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
                    }
                    else {
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
                        if super::BUILTIN_FUNCTIONS.contains(&name.as_str()) {
                            for arg in args {
                                self.compile_expr(arg, instructions)?;
                            }
                            instructions.push(OpCode::HostCall(self.module.add_or_get_string(name), args.len()));
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
                    }
                    else {
                        self.compile_expr(callee, instructions)?;
                        instructions.push(OpCode::LoadNull);
                    }
                }
                else {
                    self.compile_expr(callee, instructions)?;
                    instructions.push(OpCode::LoadNull);
                }

                instructions.push(OpCode::Call(args.len()));
            }

            TermExpression::DotCall { receiver, field, .. } => {
                self.compile_expr(receiver, instructions)?;
                instructions.push(OpCode::GetField(self.module.add_or_get_string(field.name.clone())));
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
                    }
                    else {
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
                }
                else {
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
                    }
                    else {
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
                }
                else {
                    instructions.push(OpCode::LoadNull);
                }
                instructions.push(OpCode::Return);
            }

            TermExpression::Loop { condition, body, .. } => {
                let loop_start = instructions.len();

                let loop_ctx = super::LoopContext { loop_start, break_jumps: Vec::new() };
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

            TermExpression::Break(_) => {
                if let Some(ctx) = self.loop_stack.last_mut() {
                    let jump_addr = instructions.len();
                    instructions.push(OpCode::Jump(0));
                    ctx.break_jumps.push(jump_addr);
                }
                else {
                    return Err(GError { kind: GErrorKind::Runtime, message: "Break outside of loop".to_string() });
                }
            }

            TermExpression::Continue(_) => {
                if let Some(ctx) = self.loop_stack.last() {
                    instructions.push(OpCode::Jump(ctx.loop_start));
                }
                else {
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
    pub(crate) fn is_wildcard_pattern(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard(_) => true,
            Pattern::Else(_) => true,
            Pattern::Variable(var) => var.name.name.starts_with('_'),
            _ => false,
        }
    }

    /// 将匹配模式编译为用于比较的值
    pub(crate) fn compile_match_pattern(&mut self, pattern: &Pattern, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match pattern {
            Pattern::Variable(var) => {
                if var.name.name.starts_with('_') {
                    instructions.push(OpCode::LoadTrue);
                }
                else {
                    let idx = self.module.add_constant(IrValue::String(var.name.name.clone()));
                    instructions.push(OpCode::LoadConst(idx));
                }
            }
            Pattern::Wildcard(_) => {
                instructions.push(OpCode::LoadTrue);
            }
            Pattern::Literal(lit) => {
                let ir_value = self.parse_literal_value(&lit.value);
                let idx = self.module.add_constant(ir_value);
                instructions.push(OpCode::LoadConst(idx));
            }
            Pattern::Type(type_pat) => {
                let type_name = type_pat.name.parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join("::");
                let idx = self.module.add_constant(IrValue::String(type_name));
                instructions.push(OpCode::LoadConst(idx));
            }
            Pattern::Class(class_pat) => {
                let class_name = class_pat.name.parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join("::");
                let idx = self.module.add_constant(IrValue::String(class_name));
                instructions.push(OpCode::LoadConst(idx));
            }
            Pattern::Else(_) => {
                instructions.push(OpCode::LoadTrue);
            }
        }
        Ok(())
    }

    /// 将字面量字符串解析为 IrValue
    pub(crate) fn parse_literal_value(&self, value: &str) -> IrValue {
        if let Ok(int_val) = value.parse::<i64>() {
            return IrValue::Int(int_val);
        }
        if let Ok(float_val) = value.parse::<f64>() {
            return IrValue::Float(float_val);
        }
        if value == "true" {
            return IrValue::Bool(true);
        }
        if value == "false" {
            return IrValue::Bool(false);
        }
        let trimmed = value.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(value);
        IrValue::String(trimmed.to_string())
    }

    /// 将二元运算符 token 转换为 OpCode
    pub(crate) fn binary_op_to_opcode(&self, op: &ValkyrieTokenType) -> GResult<OpCode> {
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
    pub(crate) fn unary_op_to_opcode(&self, op: &ValkyrieTokenType) -> GResult<OpCode> {
        match op {
            ValkyrieTokenType::Minus => Ok(OpCode::Neg),
            ValkyrieTokenType::Bang => Ok(OpCode::Not),
            _ => Err(GError { kind: GErrorKind::Runtime, message: format!("Unsupported unary operator: {:?}", op) }),
        }
    }
}
