#![warn(missing_docs)]

//! Valkyrie 脚本类型检查器
//!
//! 对 Valkyrie AST 进行渐进式类型检查，生成类型诊断信息（警告和错误）。
//! 类型检查不会阻止编译流程，仅产生诊断信息供开发者参考。
//!
//! # 渐进式类型系统
//!
//! - 如果变量没有类型注解且无法推断，返回 `Unknown` 且不报告错误
//! - 类型不匹配时产生警告或错误，但不阻止编译
//! - 支持基本类型推断：字面量、二元运算、一元运算、函数调用等

use std::collections::HashMap;

use oak_valkyrie::{
    ast::{
        Block, MicroDeclaration, NamePath, Pattern, Statement, StatementNode, StringLiteral, StringSegment,
        TermExpression, ValkyrieRoot,
    },
    lexer::token_type::ValkyrieTokenType,
};

/// 类型信息枚举，表示 Valkyrie 脚本中表达式的类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeInfo {
    /// 整数类型
    Int,
    /// 浮点数类型
    Float,
    /// 布尔类型
    Bool,
    /// 字符串类型
    String,
    /// 空值类型
    Null,
    /// 函数类型，包含参数类型列表和返回类型
    Function {
        /// 函数参数的类型列表
        param_types: Vec<TypeInfo>,
        /// 函数返回值的类型
        return_type: Box<TypeInfo>,
    },
    /// 未知类型（无法推断或缺少类型注解）
    Unknown,
}

impl std::fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeInfo::Int => write!(f, "Int"),
            TypeInfo::Float => write!(f, "Float"),
            TypeInfo::Bool => write!(f, "Bool"),
            TypeInfo::String => write!(f, "String"),
            TypeInfo::Null => write!(f, "Null"),
            TypeInfo::Function { param_types, return_type } => {
                let params: Vec<String> = param_types.iter().map(|t| t.to_string()).collect();
                write!(f, "({}) -> {}", params.join(", "), return_type)
            }
            TypeInfo::Unknown => write!(f, "Unknown"),
        }
    }
}

/// 诊断严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// 警告：类型可能不正确但不会阻止编译
    Warning,
    /// 错误：类型确定不正确但不会阻止编译
    Error,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Error => write!(f, "error"),
        }
    }
}

/// 类型诊断信息，记录类型检查过程中发现的问题
#[derive(Debug, Clone)]
pub struct TypeDiagnostic {
    /// 诊断消息
    pub message: String,
    /// 诊断严重级别
    pub severity: DiagnosticSeverity,
    /// 源代码位置（字节偏移起始）
    pub span_start: usize,
    /// 源代码位置（字节偏移结束）
    pub span_end: usize,
}

impl std::fmt::Display for TypeDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} ({}-{})", self.severity, self.message, self.span_start, self.span_end)
    }
}

/// 函数签名，记录函数的参数类型和返回类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    /// 函数名称
    pub name: String,
    /// 参数类型列表
    pub param_types: Vec<TypeInfo>,
    /// 返回类型
    pub return_type: TypeInfo,
}

/// 类型环境，维护变量和函数的类型信息
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    /// 变量名到类型的映射
    pub variables: HashMap<String, TypeInfo>,
    /// 函数名到签名的映射
    pub function_signatures: HashMap<String, FunctionSignature>,
}

impl TypeEnvironment {
    /// 创建空的类型环境
    pub fn new() -> Self {
        Self { variables: HashMap::new(), function_signatures: HashMap::new() }
    }

    /// 查找变量的类型
    pub fn lookup_variable(&self, name: &str) -> Option<&TypeInfo> {
        self.variables.get(name)
    }

    /// 插入变量类型
    pub fn insert_variable(&mut self, name: String, ty: TypeInfo) {
        self.variables.insert(name, ty);
    }

    /// 查找函数签名
    pub fn lookup_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.function_signatures.get(name)
    }

    /// 插入函数签名
    pub fn insert_function(&mut self, sig: FunctionSignature) {
        self.function_signatures.insert(sig.name.clone(), sig);
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Valkyrie 脚本类型检查器
///
/// 对 ValkyrieRoot AST 进行渐进式类型检查，生成类型诊断信息。
/// 类型检查不会阻止编译流程。
pub struct TypeChecker {
    /// 类型环境
    env: TypeEnvironment,
    /// 收集的诊断信息
    diagnostics: Vec<TypeDiagnostic>,
    /// 当前函数的期望返回类型
    current_return_type: Option<TypeInfo>,
}

impl TypeChecker {
    /// 创建新的类型检查器
    pub fn new() -> Self {
        Self { env: TypeEnvironment::new(), diagnostics: Vec::new(), current_return_type: None }
    }

    /// 检查 ValkyrieRoot 的所有顶层项
    ///
    /// 遍历 AST 中的顶层 Item，对每个 micro 函数进行类型检查。
    pub fn check_root(&mut self, root: &ValkyrieRoot) -> Vec<TypeDiagnostic> {
        for item in &root.items {
            self.check_item(item);
        }
        std::mem::take(&mut self.diagnostics)
    }

    /// 检查顶层 Item
    fn check_item(&mut self, item: &StatementNode) {
        match item {
            StatementNode::Micro(micro) => {
                let diags = self.check_micro(micro);
                self.diagnostics.extend(diags);
            }
            StatementNode::Namespace(namespace) => {
                for inner_item in &namespace.items {
                    self.check_item(inner_item);
                }
            }
            StatementNode::Let(let_stmt) => {
                let expr_ty = self.infer_expr(&let_stmt.expr);
                if let Some(ty) = &let_stmt.ty {
                    let annotated_ty = self.type_expr_to_type_info(ty);
                    if expr_ty != TypeInfo::Unknown && annotated_ty != TypeInfo::Unknown && expr_ty != annotated_ty {
                        self.diagnostics.push(TypeDiagnostic {
                            message: format!(
                                "Type mismatch: variable is annotated as {} but expression has type {}",
                                annotated_ty, expr_ty
                            ),
                            severity: DiagnosticSeverity::Warning,
                            span_start: let_stmt.span.start,
                            span_end: let_stmt.span.end,
                        });
                    }
                }
                if let Pattern::Variable(var) = &let_stmt.pattern {
                    let var_ty = if let Some(ty) = &let_stmt.ty {
                        self.type_expr_to_type_info(ty)
                    } else {
                        expr_ty
                    };
                    self.env.insert_variable(var.name.name.clone(), var_ty);
                }
            }
            StatementNode::ExprStmt(expr_stmt) => {
                self.infer_expr(&expr_stmt.expr);
            }
            _ => {}
        }
    }

    /// 检查 micro 函数定义
    ///
    /// 为函数参数建立类型环境，然后检查函数体中的语句和表达式。
    pub fn check_micro(&mut self, micro: &MicroDeclaration) -> Vec<TypeDiagnostic> {
        let saved_env = self.env.clone();
        self.env.variables.clear();

        let mut param_types = Vec::new();

        for param in &micro.params {
            let param_ty = param
                .ty
                .as_ref()
                .map(|t| self.type_expr_to_type_info(t))
                .unwrap_or(TypeInfo::Unknown);
            self.env.insert_variable(param.name.name.clone(), param_ty.clone());
            param_types.push(param_ty);
        }

        let return_type = micro
            .return_type
            .as_ref()
            .map(|t| self.type_expr_to_type_info(t))
            .unwrap_or(TypeInfo::Unknown);

        self.current_return_type = Some(return_type.clone());

        self.check_block(&micro.body);

        let sig = FunctionSignature {
            name: micro.name.name.clone(),
            param_types,
            return_type: return_type.clone(),
        };
        self.env.insert_function(sig);

        self.current_return_type = None;

        let diags = std::mem::take(&mut self.diagnostics);
        self.env = saved_env;
        diags
    }

    /// 检查语句块
    fn check_block(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.check_statement(stmt);
        }
    }

    /// 检查语句
    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(let_stmt) => {
                let expr_ty = self.infer_expr(&let_stmt.expr);
                if let Some(ty) = &let_stmt.ty {
                    let annotated_ty = self.type_expr_to_type_info(ty);
                    if expr_ty != TypeInfo::Unknown && annotated_ty != TypeInfo::Unknown && expr_ty != annotated_ty {
                        self.diagnostics.push(TypeDiagnostic {
                            message: format!(
                                "Type mismatch: variable is annotated as {} but expression has type {}",
                                annotated_ty, expr_ty
                            ),
                            severity: DiagnosticSeverity::Warning,
                            span_start: let_stmt.span.start,
                            span_end: let_stmt.span.end,
                        });
                    }
                }
                if let Pattern::Variable(var) = &let_stmt.pattern {
                    let var_ty = if let Some(ty) = &let_stmt.ty {
                        self.type_expr_to_type_info(ty)
                    } else {
                        expr_ty
                    };
                    self.env.insert_variable(var.name.name.clone(), var_ty);
                }
            }
            Statement::ExprStmt(expr_stmt) => {
                self.infer_expr(&expr_stmt.expr);
            }
        }
    }

    /// 推断表达式的类型
    ///
    /// 对表达式进行类型推断，同时进行类型检查。
    /// 对于无法推断类型的表达式，返回 `TypeInfo::Unknown`（渐进式类型系统）。
    pub fn infer_expr(&mut self, expr: &TermExpression) -> TypeInfo {
        match expr {
            TermExpression::NamePath(name_path) => self.infer_name_path(name_path),

            TermExpression::Bool { .. } => TypeInfo::Bool,

            TermExpression::StringLiteral(string_literal) => self.infer_string_literal(string_literal),

            TermExpression::Binary(node) => {
                let lhs_ty = self.infer_expr(&node.lhs);
                let rhs_ty = self.infer_expr(&node.rhs);
                self.infer_binary(&node.operator, &lhs_ty, &rhs_ty, node.span.start, node.span.end)
            }

            TermExpression::Unary(node) => {
                let base_ty = self.infer_expr(&node.base);
                self.infer_unary(&node.operator, &base_ty, node.span.start, node.span.end)
            }

            TermExpression::ApplyCall { callee, args, span } => {
                self.check_apply_call(callee, args, span.start, span.end)
            }

            TermExpression::If { condition, then_branch, else_branch, span, .. } => {
                self.check_if_expr(condition, then_branch, else_branch, *span)
            }

            TermExpression::Return(ret) => {
                if let Some(return_expr) = ret.base.as_ref() {
                    let return_ty = self.infer_expr(return_expr);
                    if let Some(expected) = &self.current_return_type {
                        if *expected != TypeInfo::Unknown && return_ty != TypeInfo::Unknown && return_ty != *expected {
                            self.diagnostics.push(TypeDiagnostic {
                                message: format!(
                                    "Return type mismatch: expected {} but found {}",
                                    expected, return_ty
                                ),
                                severity: DiagnosticSeverity::Error,
                                span_start: ret.span.start,
                                span_end: ret.span.end,
                            });
                        }
                    }
                    return_ty
                } else {
                    if let Some(expected) = &self.current_return_type {
                        if *expected != TypeInfo::Unknown && *expected != TypeInfo::Null {
                            self.diagnostics.push(TypeDiagnostic {
                                message: format!(
                                    "Return type mismatch: expected {} but found Null",
                                    expected
                                ),
                                severity: DiagnosticSeverity::Error,
                                span_start: ret.span.start,
                                span_end: ret.span.end,
                            });
                        }
                    }
                    TypeInfo::Null
                }
            }

            TermExpression::Paren { expr, .. } => self.infer_expr(expr),

            TermExpression::Block(block) => self.infer_block(block),

            TermExpression::Loop { condition, body, .. } => {
                if let Some(cond) = condition {
                    let cond_ty = self.infer_expr(cond);
                    if cond_ty != TypeInfo::Unknown && cond_ty != TypeInfo::Bool {
                        self.diagnostics.push(TypeDiagnostic {
                            message: format!("Loop condition must be Bool, found {}", cond_ty),
                            severity: DiagnosticSeverity::Error,
                            span_start: expr.span().start,
                            span_end: expr.span().end,
                        });
                    }
                }
                self.check_block(body);
                TypeInfo::Null
            }

            TermExpression::DotCall { receiver, .. } => {
                self.infer_expr(receiver);
                TypeInfo::Unknown
            }

            TermExpression::Index { receiver, index, .. } => {
                self.infer_expr(receiver);
                self.infer_expr(index);
                TypeInfo::Unknown
            }

            TermExpression::Object { fields, .. } => {
                for (_name, value_expr) in fields {
                    if let Some(expr) = value_expr {
                        self.infer_expr(expr);
                    }
                }
                TypeInfo::Unknown
            }

            TermExpression::Match { scrutinee, arms, .. } => {
                self.infer_expr(scrutinee);
                let mut result_ty = TypeInfo::Unknown;
                for arm in arms {
                    let arm_ty = self.infer_expr(&arm.body);
                    if result_ty == TypeInfo::Unknown {
                        result_ty = arm_ty;
                    }
                }
                result_ty
            }

            TermExpression::Micro(lambda) => {
                let mut param_types = Vec::new();
                for param in &lambda.params {
                    let param_ty = param
                        .ty
                        .as_ref()
                        .map(|t| self.type_expr_to_type_info(t))
                        .unwrap_or(TypeInfo::Unknown);
                    param_types.push(param_ty);
                }
                let return_type = lambda
                    .return_type
                    .as_ref()
                    .map(|t| self.type_expr_to_type_info(t))
                    .unwrap_or(TypeInfo::Unknown);
                TypeInfo::Function {
                    param_types,
                    return_type: Box::new(return_type),
                }
            }

            _ => TypeInfo::Unknown,
        }
    }

    /// 推断 NamePath 表达式的类型
    fn infer_name_path(&mut self, name_path: &NamePath) -> TypeInfo {
        if name_path.parts.len() == 1 {
            let name = &name_path.parts[0].name;
            match self.env.lookup_variable(name) {
                Some(ty) => ty.clone(),
                None => TypeInfo::Unknown,
            }
        } else {
            TypeInfo::Unknown
        }
    }

    /// 推断字符串字面量的类型
    ///
    /// 由于 oak-valkyrie 将整数和浮点数字面量也存储为 StringLiteral
    /// （quote_count == 0 且 prefix == None），需要区分真正的字符串和数值字面量。
    fn infer_string_literal(&mut self, string_literal: &StringLiteral) -> TypeInfo {
        if string_literal.prefix.is_none() && string_literal.quote_count == 0 {
            let content: String = string_literal
                .segments
                .iter()
                .filter_map(|seg| match seg {
                    StringSegment::Text(text_seg) => Some(text_seg.content.as_str()),
                    _ => None,
                })
                .collect();

            if content.contains('.') || content.contains('e') || content.contains('E') {
                if content.parse::<f64>().is_ok() {
                    return TypeInfo::Float;
                }
            } else if content.parse::<i64>().is_ok() {
                return TypeInfo::Int;
            }

            if content == "null" {
                return TypeInfo::Null;
            }
        }
        TypeInfo::String
    }

    /// 推断二元运算表达式的类型
    fn infer_binary(
        &mut self,
        op: &ValkyrieTokenType,
        lhs_ty: &TypeInfo,
        rhs_ty: &TypeInfo,
        span_start: usize,
        span_end: usize,
    ) -> TypeInfo {
        match op {
            ValkyrieTokenType::Plus | ValkyrieTokenType::Minus | ValkyrieTokenType::Star | ValkyrieTokenType::Slash | ValkyrieTokenType::Percent => {
                if *lhs_ty == TypeInfo::Float || *rhs_ty == TypeInfo::Float {
                    TypeInfo::Float
                } else if *lhs_ty == TypeInfo::Int && *rhs_ty == TypeInfo::Int {
                    TypeInfo::Int
                } else if *lhs_ty == TypeInfo::String && *rhs_ty == TypeInfo::String && *op == ValkyrieTokenType::Plus {
                    TypeInfo::String
                } else if *lhs_ty == TypeInfo::Unknown || *rhs_ty == TypeInfo::Unknown {
                    TypeInfo::Unknown
                } else {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!(
                            "Cannot apply operator {:?} to types {} and {}",
                            op, lhs_ty, rhs_ty
                        ),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                    });
                    TypeInfo::Unknown
                }
            }
            ValkyrieTokenType::EqEq
            | ValkyrieTokenType::NotEq
            | ValkyrieTokenType::LessThan
            | ValkyrieTokenType::LessEq
            | ValkyrieTokenType::GreaterThan
            | ValkyrieTokenType::GreaterEq => {
                if *lhs_ty != TypeInfo::Unknown
                    && *rhs_ty != TypeInfo::Unknown
                    && lhs_ty != rhs_ty
                {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!(
                            "Comparison between different types: {} and {}",
                            lhs_ty, rhs_ty
                        ),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                    });
                }
                TypeInfo::Bool
            }
            ValkyrieTokenType::AndAnd | ValkyrieTokenType::OrOr => {
                if *lhs_ty != TypeInfo::Unknown && *lhs_ty != TypeInfo::Bool {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!(
                            "Logical operator requires Bool, found {} on left side",
                            lhs_ty
                        ),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                    });
                }
                if *rhs_ty != TypeInfo::Unknown && *rhs_ty != TypeInfo::Bool {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!(
                            "Logical operator requires Bool, found {} on right side",
                            rhs_ty
                        ),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                    });
                }
                TypeInfo::Bool
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// 推断一元运算表达式的类型
    fn infer_unary(
        &mut self,
        op: &ValkyrieTokenType,
        base_ty: &TypeInfo,
        span_start: usize,
        span_end: usize,
    ) -> TypeInfo {
        match op {
            ValkyrieTokenType::Minus => {
                if *base_ty == TypeInfo::Int || *base_ty == TypeInfo::Float {
                    base_ty.clone()
                } else if *base_ty == TypeInfo::Unknown {
                    TypeInfo::Unknown
                } else {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Cannot negate type {}", base_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                    });
                    TypeInfo::Unknown
                }
            }
            ValkyrieTokenType::Bang => {
                if *base_ty != TypeInfo::Unknown && *base_ty != TypeInfo::Bool {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Logical NOT requires Bool, found {}", base_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                    });
                }
                TypeInfo::Bool
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// 检查函数调用表达式
    fn check_apply_call(
        &mut self,
        callee: &TermExpression,
        args: &[TermExpression],
        span_start: usize,
        span_end: usize,
    ) -> TypeInfo {
        let mut arg_types = Vec::new();
        for arg in args {
            arg_types.push(self.infer_expr(arg));
        }

        if let TermExpression::NamePath(name_path) = callee {
            if name_path.parts.len() == 1 {
                let name = &name_path.parts[0].name;
                if let Some(sig) = self.env.lookup_function(name) {
                    if arg_types.len() != sig.param_types.len() {
                        self.diagnostics.push(TypeDiagnostic {
                            message: format!(
                                "Function {} expects {} arguments but {} were provided",
                                name,
                                sig.param_types.len(),
                                arg_types.len()
                            ),
                            severity: DiagnosticSeverity::Error,
                            span_start,
                            span_end,
                        });
                    } else {
                        for (i, (arg_ty, param_ty)) in arg_types.iter().zip(sig.param_types.iter()).enumerate() {
                            if *arg_ty != TypeInfo::Unknown
                                && *param_ty != TypeInfo::Unknown
                                && arg_ty != param_ty
                            {
                                self.diagnostics.push(TypeDiagnostic {
                                    message: format!(
                                        "Argument {} of function {} expects type {} but found {}",
                                        i + 1,
                                        name,
                                        param_ty,
                                        arg_ty
                                    ),
                                    severity: DiagnosticSeverity::Error,
                                    span_start,
                                    span_end,
                                });
                            }
                        }
                    }
                    return sig.return_type.clone();
                }
            }
        }

        TypeInfo::Unknown
    }

    /// 检查 if 表达式
    fn check_if_expr(
        &mut self,
        condition: &TermExpression,
        then_branch: &Block,
        else_branch: &Option<Block>,
        span: oak_core::Range<usize>,
    ) -> TypeInfo {
        let cond_ty = self.infer_expr(condition);
        if cond_ty != TypeInfo::Unknown && cond_ty != TypeInfo::Bool {
            self.diagnostics.push(TypeDiagnostic {
                message: format!("If condition must be Bool, found {}", cond_ty),
                severity: DiagnosticSeverity::Error,
                span_start: span.start,
                span_end: span.end,
            });
        }

        self.check_block(then_branch);
        let then_ty = self.infer_block(then_branch);

        if let Some(else_br) = else_branch {
            self.check_block(else_br);
            let else_ty = self.infer_block(else_br);
            if then_ty != TypeInfo::Unknown && else_ty != TypeInfo::Unknown && then_ty != else_ty {
                self.diagnostics.push(TypeDiagnostic {
                    message: format!(
                        "If branches have different types: {} and {}",
                        then_ty, else_ty
                    ),
                    severity: DiagnosticSeverity::Warning,
                    span_start: span.start,
                    span_end: span.end,
                });
            }
            then_ty
        } else {
            TypeInfo::Null
        }
    }

    /// 推断语句块的类型（基于最后一个表达式语句）
    fn infer_block(&mut self, block: &Block) -> TypeInfo {
        let mut last_ty = TypeInfo::Null;
        for stmt in &block.statements {
            match stmt {
                Statement::Let(let_stmt) => {
                    let expr_ty = self.infer_expr(&let_stmt.expr);
                    if let Pattern::Variable(var) = &let_stmt.pattern {
                        let var_ty = if let Some(ty) = &let_stmt.ty {
                            self.type_expr_to_type_info(ty)
                        } else {
                            expr_ty
                        };
                        self.env.insert_variable(var.name.name.clone(), var_ty);
                    }
                    last_ty = TypeInfo::Null;
                }
                Statement::ExprStmt(expr_stmt) => {
                    let ty = self.infer_expr(&expr_stmt.expr);
                    if !expr_stmt.semi {
                        last_ty = ty;
                    } else {
                        last_ty = TypeInfo::Null;
                    }
                }
            }
        }
        last_ty
    }

    /// 将 TypeExpression 转换为 TypeInfo
    fn type_expr_to_type_info(&self, type_expr: &oak_valkyrie::ast::TypeExpression) -> TypeInfo {
        match type_expr {
            oak_valkyrie::ast::TypeExpression::Namepath(name_path) => {
                if name_path.parts.len() == 1 {
                    let name = &name_path.parts[0].name;
                    match name.as_str() {
                        "Int" | "i32" | "i64" | "u32" | "u64" => TypeInfo::Int,
                        "Float" | "f32" | "f64" => TypeInfo::Float,
                        "Bool" => TypeInfo::Bool,
                        "String" => TypeInfo::String,
                        "Null" => TypeInfo::Null,
                        _ => TypeInfo::Unknown,
                    }
                } else {
                    TypeInfo::Unknown
                }
            }
            oak_valkyrie::ast::TypeExpression::Function(func_type) => {
                let param_types: Vec<TypeInfo> =
                    func_type.params.iter().map(|t| self.type_expr_to_type_info(t)).collect();
                let return_type = self.type_expr_to_type_info(&func_type.return_type);
                TypeInfo::Function {
                    param_types,
                    return_type: Box::new(return_type),
                }
            }
            oak_valkyrie::ast::TypeExpression::Optional(opt_type) => {
                self.type_expr_to_type_info(&opt_type.inner)
            }
            _ => TypeInfo::Unknown,
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
