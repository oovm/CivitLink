#![warn(missing_docs)]

//! Valkyrie AST 到 IR 编译器
//! 将 oak-valkyrie 生成的 ValkyrieRoot AST 编译为 gg-ir 的 IrModule

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_ir::{EntryPoint, IrFunction, IrModule, IrValue, OpCode, TargetPlatform};
use oak_valkyrie::{
    ast::{
        Attribute, Block, ClassDeclaration, ComponentDeclaration, Enums, MethodDeclaration, MicroDeclaration, Pattern,
        SingletonDeclaration, Statement, StatementNode, StringSegment, StructureDeclaration, SystemDeclaration, TermExpression,
        Trait, ValkyrieRoot,
        items_nodes::{Flags, ShaderDeclaration},
    },
    lexer::token_type::ValkyrieTokenType,
};

/// 内置宿主函数列表，编译为 HostCall 指令
const BUILTIN_FUNCTIONS: &[&str] = &[
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
    /// 局部变量名称列表，按索引顺序排列，用于调试
    local_names: Vec<String>,
    /// 下一个可用的局部变量索引
    next_local: usize,
    /// 循环上下文栈
    loop_stack: Vec<LoopContext>,
    /// 入口点列表
    entry_points: Vec<EntryPoint>,
    /// 目标平台
    target_platform: Option<TargetPlatform>,
}

impl ValkyrieCompiler {
    /// 创建新的编译器
    pub fn new(module_name: &str) -> Self {
        Self {
            module: IrModule::new(module_name),
            locals: HashMap::new(),
            local_names: Vec::new(),
            next_local: 0,
            loop_stack: Vec::new(),
            entry_points: Vec::new(),
            target_platform: None,
            module_init_instructions: Vec::new(),
        }
    }

    /// 创建带有目标平台的编译器
    pub fn with_target(module_name: &str, target: TargetPlatform) -> Self {
        Self {
            module: IrModule::new(module_name),
            locals: HashMap::new(),
            local_names: Vec::new(),
            next_local: 0,
            loop_stack: Vec::new(),
            entry_points: Vec::new(),
            target_platform: Some(target),
            module_init_instructions: Vec::new(),
        }
    }

    /// 编译 ValkyrieRoot AST 为 IrModule
    ///
    /// 遍历 AST 中的顶层 Item，对每个 micro 函数定义编译为 IrFunction。
    /// 编译完成后，若存在类型注册指令，则生成 `__module_init__` 函数作为入口点。
    pub fn compile(mut self, root: &ValkyrieRoot, module_name: &str) -> GResult<IrModule> {
        self.module = IrModule::new(module_name);
        self.module_init_instructions.clear();
        for item in &root.items {
            self.compile_item(item)?;
        }
        if !self.module_init_instructions.is_empty() {
            let mut init_instructions = std::mem::take(&mut self.module_init_instructions);
            init_instructions.push(OpCode::LoadNull);
            init_instructions.push(OpCode::Return);
            let init_func = IrFunction {
                name: "__module_init__".to_string(),
                param_count: 0,
                local_count: 0,
                local_names: vec![],
                instructions: init_instructions,
                is_entry: true,
                target: None,
            };
            self.module.add_function(init_func);
            self.entry_points
                .insert(0, EntryPoint { name: "__module_init__".to_string(), target: TargetPlatform::All, priority: 0 });
            for entry in &mut self.entry_points {
                entry.priority += 1;
            }
        }
        self.module.entry_points = self.entry_points.clone();
        self.module.target_platform = self.target_platform;
        Ok(self.module)
    }

    /// 编译顶层 Item
    fn compile_item(&mut self, item: &StatementNode) -> GResult<()> {
        match item {
            StatementNode::Micro(micro) => {
                if !self.filter_by_target(&micro.annotations) {
                    return Ok(());
                }
                let func = self.compile_micro(micro)?;
                self.module.add_function(func);
            }
            StatementNode::Namespace(namespace) => {
                for inner_item in &namespace.items {
                    self.compile_item(inner_item)?;
                }
            }
            StatementNode::Let(let_stmt) => {
                if !self.filter_by_target(&let_stmt.annotations) {
                    return Ok(());
                }
                let mut instructions = Vec::new();
                self.compile_statement(&Statement::Let((**let_stmt).clone()), &mut instructions)?;
            }
            StatementNode::ExprStmt(expr_stmt) => {
                if !self.filter_by_target(&expr_stmt.annotations) {
                    return Ok(());
                }
                let mut instructions = Vec::new();
                self.compile_statement(&Statement::ExprStmt((**expr_stmt).clone()), &mut instructions)?;
            }
            StatementNode::Shader(shader) => {
                let _ = shader;
            }
            StatementNode::Class(class) => {
                self.compile_class(class)?;
            }
            StatementNode::Structure(structure) => {
                self.compile_structure(structure)?;
            }
            StatementNode::Trait(trait_decl) => {
                self.compile_trait_decl(trait_decl)?;
            }
            StatementNode::Singleton(singleton) => {
                self.compile_singleton(singleton)?;
            }
            StatementNode::Enums(enums) => {
                self.compile_enums(enums)?;
            }
            StatementNode::Flags(flags) => {
                let _ = flags;
            }
            StatementNode::Component(component) => {
                self.compile_component(component)?;
            }
            StatementNode::System(system) => {
                self.compile_system(system)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// 编译 micro 函数定义
    ///
    /// 为每个 micro 函数生成 IrFunction，包含参数、局部变量和指令序列。
    fn compile_micro(&mut self, micro: &MicroDeclaration) -> GResult<IrFunction> {
        self.reset_locals();
        self.process_annotations(micro);

        let param_count = micro.params.len();

        for param in &micro.params {
            self.declare_local(param.name.name.clone());
        }

        let mut instructions = Vec::new();
        self.compile_block(&micro.body, &mut instructions)?;

        instructions.push(OpCode::LoadNull);
        instructions.push(OpCode::Return);

        let is_entry = micro.annotations.iter().any(|a| a.name.name == "main");
        let target = self.resolve_target_annotation(micro);
        Ok(IrFunction {
            name: micro.name.name.clone(),
            param_count,
            local_count: self.next_local,
            local_names: std::mem::take(&mut self.local_names),
            instructions,
            is_entry,
            target,
        })
    }

    /// 编译语句块
    fn compile_block(&mut self, block: &Block, instructions: &mut Vec<OpCode>) -> GResult<()> {
        for stmt in &block.statements {
            self.compile_statement(stmt, instructions)?;
        }
        Ok(())
    }

    /// 编译语句
    ///
    /// TODO: Pattern 枚举目前没有 Tuple 变体，待 oak-valkyrie 添加后需实现元组解构绑定：
    /// 编译表达式后，为每个元素生成 GetIndex + StoreLocal 指令。
    fn compile_statement(&mut self, stmt: &Statement, instructions: &mut Vec<OpCode>) -> GResult<()> {
        match stmt {
            Statement::Let(let_stmt) => {
                self.compile_expr(&let_stmt.expr, instructions)?;
                match &let_stmt.pattern {
                    Pattern::Variable(var) => {
                        let var_name = var.name.name.clone();
                        let idx = self.declare_local(var_name);
                        instructions.push(OpCode::StoreLocal(idx));
                    }
                    Pattern::Wildcard(_) => {
                        instructions.push(OpCode::Pop);
                    }
                    Pattern::Class(class_pat) => {
                        let temp_local = self.declare_local("__destructure_temp".to_string());
                        instructions.push(OpCode::StoreLocal(temp_local));
                        for (field_name, field_pattern) in &class_pat.fields {
                            instructions.push(OpCode::LoadLocal(temp_local));
                            let field_idx = self.module.add_or_get_string(field_name.name.clone());
                            instructions.push(OpCode::GetField(field_idx));
                            match field_pattern {
                                Some(Pattern::Variable(var)) => {
                                    let var_name = var.name.name.clone();
                                    let idx = self.declare_local(var_name);
                                    instructions.push(OpCode::StoreLocal(idx));
                                }
                                Some(Pattern::Wildcard(_)) => {
                                    instructions.push(OpCode::Pop);
                                }
                                Some(Pattern::Class(_)) => {
                                    let idx = self.declare_local(format!("__destructure_{}", self.next_local));
                                    instructions.push(OpCode::StoreLocal(idx));
                                }
                                None => {
                                    let var_name = field_name.name.clone();
                                    let idx = self.declare_local(var_name);
                                    instructions.push(OpCode::StoreLocal(idx));
                                }
                                _ => {
                                    instructions.push(OpCode::Pop);
                                }
                            }
                        }
                    }
                    Pattern::Literal(_) | Pattern::Type(_) | Pattern::Else(_) => {
                        instructions.push(OpCode::Pop);
                    }
                }
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
                        if BUILTIN_FUNCTIONS.contains(&name.as_str()) {
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
    fn is_wildcard_pattern(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard(_) => true,
            Pattern::Else(_) => true,
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
    fn parse_literal_value(&self, value: &str) -> IrValue {
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

    /// 处理 micro 声明上的注解，提取入口点信息
    fn process_annotations(&mut self, micro: &MicroDeclaration) {
        for attr in &micro.annotations {
            match attr.name.name.as_str() {
                "main" => {
                    let target = if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                        let content: String = sl
                            .segments
                            .iter()
                            .filter_map(|seg| match seg {
                                StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                                _ => None,
                            })
                            .collect();
                        TargetPlatform::from_str(&content).unwrap_or(TargetPlatform::All)
                    }
                    else {
                        TargetPlatform::All
                    };
                    let priority = self.entry_points.len() as u32;
                    self.entry_points.push(EntryPoint { name: micro.name.name.clone(), target, priority });
                }
                _ => {}
            }
        }
    }

    /// 解析 @target 注解，返回目标平台
    fn resolve_target_annotation(&self, micro: &MicroDeclaration) -> Option<TargetPlatform> {
        for attr in &micro.annotations {
            if attr.name.name == "target" {
                if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                    let content: String = sl
                        .segments
                        .iter()
                        .filter_map(|seg| match seg {
                            StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                            _ => None,
                        })
                        .collect();
                    return TargetPlatform::from_str(content.split(',').next().unwrap_or("").trim());
                }
            }
        }
        None
    }

    /// 编译类对 trait 的实现块
    ///
    /// 当类通过 parents 继承 trait 时，类的方法即为 trait 方法的实现。
    /// 为每个有方法体的方法生成 `类名_trait名_方法名` 格式的 IR 函数。
    fn compile_impl_block(&mut self, class_name: &str, trait_name: &str, methods: &[MethodDeclaration]) -> GResult<()> {
        for method in methods {
            if method.body.is_none() {
                continue;
            }
            self.reset_locals();
            let param_count = method.params.len();
            for param in &method.params {
                self.declare_local(param.name.name.clone());
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let impl_name = format!("{}_{}_{}", class_name, trait_name, method.name.name);
            let func = IrFunction {
                name: impl_name,
                param_count,
                local_count: self.next_local,
                local_names: std::mem::take(&mut self.local_names),
                instructions,
                is_entry: false,
                target: None,
            };
            self.module.add_function(func);
        }
        Ok(())
    }

    /// 编译类声明
    ///
    /// 为类中每个有方法体的方法生成 `类名_方法名` 格式的 IR 函数，
    /// 并处理 @main 注解标记的入口点。对于类的父 trait，调用 compile_impl_block 生成实现函数。
    fn compile_class(&mut self, class: &ClassDeclaration) -> GResult<()> {
        if !self.filter_by_target(&class.annotations) {
            return Ok(());
        }
        for method in &class.methods {
            if method.body.is_none() {
                continue;
            }
            self.locals.clear();
            self.next_local = 0;
            self.loop_stack.clear();
            let param_count = method.params.len();
            for param in &method.params {
                self.locals.insert(param.name.name.clone(), self.next_local);
                self.next_local += 1;
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let func_name = format!("{}_{}", class.name.name, method.name.name);
            let is_entry = method.annotations.iter().any(|a| a.name.name == "main");
            let target = self.resolve_target_from_annotations(&method.annotations);
            let func = IrFunction {
                name: func_name.clone(),
                param_count,
                local_count: self.next_local,
                local_names: vec![],
                instructions,
                is_entry,
                target,
            };
            self.module.add_function(func);
            if is_entry {
                let entry_target = target.unwrap_or(TargetPlatform::All);
                self.entry_points.push(EntryPoint {
                    name: func_name.clone(),
                    target: entry_target,
                    priority: self.entry_points.len() as u32,
                });
            }
        }
        for parent in &class.parents {
            let trait_name = parent.name.parts.last().map(|p| p.name.clone()).unwrap_or_default();
            self.compile_impl_block(&class.name.name, &trait_name, &class.methods)?;
        }
        Ok(())
    }

    /// 编译结构体声明
    ///
    /// 根据 @target 注解进行过滤，将结构体名称和每个字段名称注册到字符串池，
    /// 并生成 `register_structure` 宿主调用指令。
    /// 指令参数为：名称索引、字段数量、各字段索引。
    fn compile_structure(&mut self, structure: &StructureDeclaration) -> GResult<()> {
        if !self.filter_by_target(&structure.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(structure.name.name.clone());
        let field_count = structure.fields.len();
        let mut field_indices = Vec::with_capacity(field_count);
        for field in &structure.fields {
            let idx = self.module.add_or_get_string(field.name.name.clone());
            field_indices.push(idx);
        }
        let register_fn_idx = self.module.add_or_get_string("register_structure".to_string());
        self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(field_count as i64))));
        for idx in field_indices {
            self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(idx as i64))));
        }
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2 + field_count));
        Ok(())
    }

    /// 编译 trait 声明
    ///
    /// 根据 @target 注解进行过滤，将 trait 名称和每个方法名称注册到字符串池，
    /// 并生成 `register_trait` 宿主调用指令。
    /// 指令参数为：名称索引、方法数量、各方法索引。
    fn compile_trait_decl(&mut self, trait_decl: &Trait) -> GResult<()> {
        if !self.filter_by_target(&trait_decl.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(trait_decl.name.name.clone());
        let method_count = trait_decl.methods.len();
        let mut method_indices = Vec::with_capacity(method_count);
        for method in &trait_decl.methods {
            let idx = self.module.add_or_get_string(method.name.name.clone());
            method_indices.push(idx);
        }
        let register_fn_idx = self.module.add_or_get_string("register_trait".to_string());
        self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(method_count as i64))));
        for idx in method_indices {
            self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(idx as i64))));
        }
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2 + method_count));
        Ok(())
    }

    /// 编译单例声明
    ///
    /// 为单例中每个有方法体的方法生成 `单例名_方法名` 格式的 IR 函数。
    fn compile_singleton(&mut self, singleton: &SingletonDeclaration) -> GResult<()> {
        if !self.filter_by_target(&singleton.annotations) {
            return Ok(());
        }
        for method in &singleton.methods {
            if method.body.is_none() {
                continue;
            }
            self.locals.clear();
            self.next_local = 0;
            self.loop_stack.clear();
            let param_count = method.params.len();
            for param in &method.params {
                self.locals.insert(param.name.name.clone(), self.next_local);
                self.next_local += 1;
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let func_name = format!("{}_{}", singleton.name.name, method.name.name);
            let func = IrFunction {
                name: func_name,
                param_count,
                local_count: self.next_local,
                local_names: vec![],
                instructions,
                is_entry: false,
                target: None,
            };
            self.module.add_function(func);
        }
        Ok(())
    }

    /// 编译枚举声明
    ///
    /// 为枚举的每个变体生成 `枚举名_变体名` 格式的字符串常量并加入常量池。
    fn compile_enums(&mut self, enums: &Enums) -> GResult<()> {
        if !self.filter_by_target(&enums.annotations) {
            return Ok(());
        }
        for variant in &enums.variants {
            let idx = self.module.add_constant(IrValue::String(format!("{}_{}", enums.name.name, variant.name.name)));
            let _ = idx;
        }
        Ok(())
    }

    /// 编译 ECS 组件声明
    ///
    /// 将组件名称加入字符串池，目前仅记录组件名称。
    fn compile_component(&mut self, component: &ComponentDeclaration) -> GResult<()> {
        if !self.filter_by_target(&component.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(component.name.name.clone());
        let _ = name_idx;
        Ok(())
    }

    /// 编译着色器声明
    ///
    /// 将着色器名称和类型注册到字符串池，并生成 `register_shader` 宿主调用指令。
    /// 着色器类型包括 PBR、Unlit、Phong、Compute、UiUnlit、UiSdf、UiCustom。
    fn compile_shader(&mut self, shader: &ShaderDeclaration) -> GResult<()> {
        if !self.filter_by_target(&shader.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(shader.name.name.clone());
        let kind_idx = self.module.add_or_get_string(shader.kind.name.clone());
        let register_fn_idx = self.module.add_or_get_string("register_shader".to_string());
        self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(name_idx as i64))));
        self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(kind_idx as i64))));
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2));
        Ok(())
    }

    /// 编译位标志声明
    ///
    /// 将位标志名称和每个变体名称注册到字符串池，并生成 `register_flags` 宿主调用指令。
    /// 指令参数为：名称索引、变体数量、各变体索引。
    fn compile_flags(&mut self, flags: &Flags) -> GResult<()> {
        if !self.filter_by_target(&flags.annotations) {
            return Ok(());
        }
        let name_idx = self.module.add_or_get_string(flags.name.name.clone());
        let variant_count = flags.variants.len();
        let mut variant_indices = Vec::with_capacity(variant_count);
        for variant in &flags.variants {
            let idx = self.module.add_or_get_string(variant.name.name.clone());
            variant_indices.push(idx);
        }
        let register_fn_idx = self.module.add_or_get_string("register_flags".to_string());
        self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(name_idx as i64))));
        self.module_init_instructions
            .push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(variant_count as i64))));
        for idx in variant_indices {
            self.module_init_instructions.push(OpCode::LoadConst(self.module.add_or_get_constant(IrValue::Int(idx as i64))));
        }
        self.module_init_instructions.push(OpCode::HostCall(register_fn_idx, 2 + variant_count));
        Ok(())
    }

    /// 编译 ECS 系统声明
    ///
    /// 为系统中每个有方法体的方法生成 `系统名_方法名` 格式的 IR 函数。
    fn compile_system(&mut self, system: &SystemDeclaration) -> GResult<()> {
        if !self.filter_by_target(&system.annotations) {
            return Ok(());
        }
        for method in &system.methods {
            if method.body.is_none() {
                continue;
            }
            self.locals.clear();
            self.next_local = 0;
            self.loop_stack.clear();
            let param_count = method.params.len();
            for param in &method.params {
                self.locals.insert(param.name.name.clone(), self.next_local);
                self.next_local += 1;
            }
            let mut instructions = Vec::new();
            if let Some(body) = &method.body {
                self.compile_block(body, &mut instructions)?;
            }
            instructions.push(OpCode::LoadNull);
            instructions.push(OpCode::Return);
            let func_name = format!("{}_{}", system.name.name, method.name.name);
            let func = IrFunction {
                name: func_name,
                param_count,
                local_count: self.next_local,
                local_names: vec![],
                instructions,
                is_entry: false,
                target: None,
            };
            self.module.add_function(func);
        }
        Ok(())
    }

    /// 从注解列表中解析 @target 注解，返回目标平台
    ///
    /// 支持 @target("platform") 格式的注解，取逗号分隔的第一个平台名称进行解析。
    fn resolve_target_from_annotations(&self, annotations: &[Attribute]) -> Option<TargetPlatform> {
        for attr in annotations {
            if attr.name.name == "target" {
                if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                    let content: String = sl
                        .segments
                        .iter()
                        .filter_map(|seg| match seg {
                            StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                            _ => None,
                        })
                        .collect();
                    return TargetPlatform::from_str(content.split(',').next().unwrap_or("").trim());
                }
            }
        }
        None
    }

    /// 根据 @target 注解过滤，判断当前声明是否应该编译
    fn filter_by_target(&self, annotations: &[Attribute]) -> bool {
        if let Some(current_target) = self.target_platform {
            for attr in annotations {
                if attr.name.name == "target" {
                    if let Some(TermExpression::StringLiteral(sl)) = attr.args.first() {
                        let content: String = sl
                            .segments
                            .iter()
                            .filter_map(|seg| match seg {
                                StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                                _ => None,
                            })
                            .collect();
                        let platforms: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
                        let matches = platforms.iter().any(|p| {
                            TargetPlatform::from_str(p).map_or(false, |tp| tp == current_target || tp == TargetPlatform::All)
                        });
                        if !matches {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
}
