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

use std::collections::{HashMap, HashSet};

use oak_valkyrie::{
    ast::{
        Attribute, Block, ClassDeclaration, ComponentDeclaration, Enums, FieldDeclaration, Flags, MethodDeclaration,
        MicroDeclaration, NamePath, Pattern, SingletonDeclaration, Statement, StatementNode, StringLiteral, StringSegment,
        StructureDeclaration, SystemDeclaration, TermExpression, Trait, ValkyrieRoot,
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
    /// 数组类型，包含元素类型
    Array(Box<TypeInfo>),
    /// 映射类型，包含键类型和值类型
    Map(Box<TypeInfo>, Box<TypeInfo>),
    /// 对象/类类型，包含类型名称
    Object(String),
    /// 特征类型，包含特征名称
    Trait(String),
    /// ECS 组件类型，包含组件名称
    Component(String),
    /// 闭包类型，包含参数类型、返回类型和捕获变量列表
    Closure {
        /// 闭包参数的类型列表
        param_types: Vec<TypeInfo>,
        /// 闭包返回值的类型
        return_type: Box<TypeInfo>,
        /// 闭包捕获的变量列表，每项为变量名与类型的元组
        captures: Vec<(String, TypeInfo)>,
    },
    /// 元组类型，包含有序的元素类型列表
    Tuple(Vec<TypeInfo>),
    /// 可选类型，表示可能为空的值
    Optional(Box<TypeInfo>),
    /// 泛型类型，包含名称和类型参数列表
    Generic(
        /// 泛型类型名称
        String,
        /// 泛型类型参数列表
        Vec<TypeInfo>,
    ),
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
            TypeInfo::Array(elem) => write!(f, "Array<{}>", elem),
            TypeInfo::Map(key, val) => write!(f, "Map<{}, {}>", key, val),
            TypeInfo::Object(name) => write!(f, "{}", name),
            TypeInfo::Trait(name) => write!(f, "{}", name),
            TypeInfo::Component(name) => write!(f, "{}", name),
            TypeInfo::Closure { param_types, return_type, captures } => {
                let caps: Vec<String> = captures.iter().map(|(name, ty)| format!("{}: {}", name, ty)).collect();
                let params: Vec<String> = param_types.iter().map(|t| t.to_string()).collect();
                write!(f, "(captures: [{}], params: [{}]) -> {}", caps.join(", "), params.join(", "), return_type)
            }
            TypeInfo::Tuple(types) => {
                let parts: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                write!(f, "({})", parts.join(", "))
            }
            TypeInfo::Optional(inner) => write!(f, "{}?", inner),
            TypeInfo::Generic(name, type_args) => {
                let args: Vec<String> = type_args.iter().map(|t| t.to_string()).collect();
                write!(f, "{}<{}>", name, args.join(", "))
            }
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
    /// 修复建议
    pub suggestion: Option<String>,
}

impl std::fmt::Display for TypeDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} ({}-{})", self.severity, self.message, self.span_start, self.span_end)?;
        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }
        Ok(())
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
    /// 自定义类型名到类型信息的映射
    pub types: HashMap<String, TypeInfo>,
    /// 类/组件/结构体的字段类型映射（类型名 -> 字段名 -> 字段类型）
    pub class_fields: HashMap<String, HashMap<String, TypeInfo>>,
    /// trait 要求的方法签名映射（trait名 -> 方法签名列表）
    pub trait_methods: HashMap<String, Vec<FunctionSignature>>,
}

impl TypeEnvironment {
    /// 创建空的类型环境
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            function_signatures: HashMap::new(),
            types: HashMap::new(),
            class_fields: HashMap::new(),
            trait_methods: HashMap::new(),
        }
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

    /// 注册自定义类型
    pub fn register_type(&mut self, name: String, ty: TypeInfo) {
        self.types.insert(name, ty);
    }

    /// 查找自定义类型
    pub fn lookup_type(&self, name: &str) -> Option<&TypeInfo> {
        self.types.get(name)
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
    pub env: TypeEnvironment,
    /// 收集的诊断信息
    pub diagnostics: Vec<TypeDiagnostic>,
    /// 当前函数的期望返回类型
    current_return_type: Option<TypeInfo>,
}

impl TypeChecker {
    /// 创建新的类型检查器
    pub fn new() -> Self {
        Self { env: TypeEnvironment::new(), diagnostics: Vec::new(), current_return_type: None }
    }

    /// 计算两个字符串之间的 Levenshtein 编辑距离
    pub fn levenshtein_distance(a: &str, b: &str) -> usize {
        let a_len = a.chars().count();
        let b_len = b.chars().count();
        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }
        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
        for (i, row) in matrix.iter_mut().enumerate() {
            row[0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }
        for (i, a_char) in a.chars().enumerate() {
            for (j, b_char) in b.chars().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1).min(matrix[i + 1][j] + 1).min(matrix[i][j] + cost);
            }
        }
        matrix[a_len][b_len]
    }

    /// 查找与目标名称相似的变量名列表
    ///
    /// 使用 Levenshtein 编辑距离算法，返回最多3个相似名称。
    /// 最大距离阈值为目标名称长度的一半（最小为2）。
    pub fn find_similar_names(target: &str, candidates: &[String], max_results: usize) -> Vec<String> {
        let threshold = std::cmp::max(target.len() / 2, 2);
        let mut scored: Vec<(usize, String)> = candidates
            .iter()
            .filter(|c| *c != target)
            .filter_map(|c| {
                let dist = Self::levenshtein_distance(target, c);
                if dist <= threshold { Some((dist, c.clone())) } else { None }
            })
            .collect();
        scored.sort_by_key(|(dist, _)| *dist);
        scored.into_iter().take(max_results).map(|(_, name)| name).collect()
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
                            suggestion: Some(format!(
                                "Consider changing the type annotation to {} or converting the expression to {}",
                                expr_ty, annotated_ty
                            )),
                        });
                    }
                }
                if let Pattern::Variable(var) = &let_stmt.pattern {
                    let var_ty = if let Some(ty) = &let_stmt.ty { self.type_expr_to_type_info(ty) } else { expr_ty };
                    self.env.insert_variable(var.name.name.clone(), var_ty);
                }
            }
            StatementNode::ExprStmt(expr_stmt) => {
                self.infer_expr(&expr_stmt.expr);
            }
            StatementNode::Class(class) => {
                self.check_class(class);
            }
            StatementNode::Structure(structure) => {
                self.check_structure(structure);
            }
            StatementNode::Trait(trait_decl) => {
                self.check_trait(trait_decl);
            }
            StatementNode::Singleton(singleton) => {
                self.check_singleton(singleton);
            }
            StatementNode::Enums(enums) => {
                self.check_enums(enums);
            }
            StatementNode::Flags(flags) => {
                self.check_flags(flags);
            }
            StatementNode::Component(component) => {
                self.check_component(component);
            }
            StatementNode::System(system) => {
                self.check_system(system);
            }
            _ => {}
        }
    }

    /// 检查类声明
    ///
    /// 注册类类型，为字段建立类型环境，然后检查类中的方法。
    /// 同时收集字段类型到 class_fields，并验证类是否实现了所有父 trait 的方法。
    pub fn check_class(&mut self, class: &ClassDeclaration) {
        if !self.filter_by_target(&class.annotations) {
            return;
        }
        self.env.register_type(class.name.name.clone(), TypeInfo::Object(class.name.name.clone()));
        let mut fields = HashMap::new();
        for field in &class.fields {
            let field_ty = self.type_expr_to_type_info(&field.ty);
            fields.insert(field.name.name.clone(), field_ty.clone());
            self.env.insert_variable(format!("self.{}", field.name.name), field_ty);
        }
        self.env.class_fields.insert(class.name.name.clone(), fields);
        let saved_env = self.env.variables.clone();
        for method in &class.methods {
            self.check_method(method, &class.name.name);
        }
        for parent in &class.parents {
            let trait_name = parent.name.parts.last().map(|p| p.name.clone()).unwrap_or_default();
            self.check_impl_block(&class.name.name, &trait_name, &class.methods);
        }
        self.env.variables = saved_env;
    }

    /// 检查结构体声明
    ///
    /// 注册结构体类型信息，并收集字段类型到 class_fields。
    pub fn check_structure(&mut self, structure: &StructureDeclaration) {
        if !self.filter_by_target(&structure.annotations) {
            return;
        }
        self.env.register_type(structure.name.name.clone(), TypeInfo::Object(structure.name.name.clone()));
        let mut fields = HashMap::new();
        for field in &structure.fields {
            let field_ty = self.type_expr_to_type_info(&field.ty);
            fields.insert(field.name.name.clone(), field_ty);
        }
        self.env.class_fields.insert(structure.name.name.clone(), fields);
    }

    /// 检查特征声明
    ///
    /// 注册特征类型，并为特征中的方法注册函数签名。
    /// 同时收集方法签名到 trait_methods，用于后续 impl 验证。
    pub fn check_trait(&mut self, trait_decl: &Trait) {
        if !self.filter_by_target(&trait_decl.annotations) {
            return;
        }
        self.env.register_type(trait_decl.name.name.clone(), TypeInfo::Trait(trait_decl.name.name.clone()));
        let mut trait_method_sigs = Vec::new();
        for method in &trait_decl.methods {
            let mut param_types = Vec::new();
            for param in &method.params {
                let param_ty = param.ty.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);
                param_types.push(param_ty);
            }
            let return_type = method.return_type.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);
            let sig =
                FunctionSignature { name: format!("{}_{}", trait_decl.name.name, method.name.name), param_types, return_type };
            self.env.insert_function(sig);
            trait_method_sigs.push(FunctionSignature {
                name: method.name.name.clone(),
                param_types: method.params.iter().map(|p| p.ty.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown)).collect(),
                return_type: method.return_type.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown),
            });
        }
        self.env.trait_methods.insert(trait_decl.name.name.clone(), trait_method_sigs);
    }

    /// 检查单例声明
    ///
    /// 注册单例类型信息。
    pub fn check_singleton(&mut self, singleton: &SingletonDeclaration) {
        if !self.filter_by_target(&singleton.annotations) {
            return;
        }
        self.env.register_type(singleton.name.name.clone(), TypeInfo::Object(singleton.name.name.clone()));
    }

    /// 检查枚举声明
    ///
    /// 注册枚举类型信息。
    pub fn check_enums(&mut self, enums: &Enums) {
        if !self.filter_by_target(&enums.annotations) {
            return;
        }
        self.env.register_type(enums.name.name.clone(), TypeInfo::Object(enums.name.name.clone()));
    }

    /// 检查标志位声明
    ///
    /// 注册标志位类型信息。
    pub fn check_flags(&mut self, flags: &Flags) {
        if !self.filter_by_target(&flags.annotations) {
            return;
        }
        self.env.register_type(flags.name.name.clone(), TypeInfo::Object(flags.name.name.clone()));
    }

    /// 检查 ECS 组件声明
    ///
    /// 注册组件类型信息，并收集字段类型到 class_fields。
    pub fn check_component(&mut self, component: &ComponentDeclaration) {
        if !self.filter_by_target(&component.annotations) {
            return;
        }
        self.env.register_type(component.name.name.clone(), TypeInfo::Component(component.name.name.clone()));
        let mut fields = HashMap::new();
        for field in &component.fields {
            let field_ty = self.type_expr_to_type_info(&field.ty);
            fields.insert(field.name.name.clone(), field_ty);
        }
        self.env.class_fields.insert(component.name.name.clone(), fields);
    }

    /// 检查 ECS 系统声明
    ///
    /// 检查系统中的方法定义。
    pub fn check_system(&mut self, system: &SystemDeclaration) {
        if !self.filter_by_target(&system.annotations) {
            return;
        }
        for method in &system.methods {
            self.check_method(method, &system.name.name);
        }
    }

    /// 检查方法声明
    ///
    /// 为方法参数建立类型环境，检查方法体，并注册方法签名。
    pub fn check_method(&mut self, method: &MethodDeclaration, _owner_name: &str) {
        if method.body.is_none() {
            return;
        }
        let saved_env = self.env.variables.clone();
        self.env.variables.clear();
        let mut param_types = Vec::new();
        for param in &method.params {
            let param_ty = param.ty.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);
            self.env.insert_variable(param.name.name.clone(), param_ty.clone());
            param_types.push(param_ty);
        }
        let return_type = method.return_type.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);
        self.current_return_type = Some(return_type.clone());
        if let Some(body) = &method.body {
            self.check_block(body);
        }
        let sig = FunctionSignature { name: method.name.name.clone(), param_types, return_type };
        self.env.insert_function(sig);
        self.current_return_type = None;
        self.env.variables = saved_env;
    }

    /// 检查类是否实现了 trait 要求的所有方法
    ///
    /// 对比 trait_methods 中记录的方法签名与类实际定义的方法，
    /// 如果缺失方法则生成 Error 级别诊断（含 suggestion 字段）。
    pub fn check_impl_block(&mut self, class_name: &str, trait_name: &str, class_methods: &[MethodDeclaration]) {
        let required_methods = match self.env.trait_methods.get(trait_name) {
            Some(methods) => methods,
            None => return,
        };
        let implemented_names: HashSet<String> = class_methods.iter().map(|m| m.name.name.clone()).collect();
        for required in required_methods {
            if !implemented_names.contains(&required.name) {
                self.diagnostics.push(TypeDiagnostic {
                    message: format!(
                        "Type '{}' does not implement all methods of trait '{}': missing '{}'",
                        class_name, trait_name, required.name
                    ),
                    severity: DiagnosticSeverity::Error,
                    span_start: 0,
                    span_end: 0,
                    suggestion: Some(format!(
                        "Add method '{}' to type '{}' to satisfy trait '{}'",
                        required.name, class_name, trait_name
                    )),
                });
            }
        }
    }

    /// 根据注解过滤目标平台
    ///
    /// 检查声明是否应被当前目标平台处理。
    pub fn filter_by_target(&self, annotations: &[Attribute]) -> bool {
        true
    }

    /// 检查 micro 函数定义
    ///
    /// 为函数参数建立类型环境，然后检查函数体中的语句和表达式。
    pub fn check_micro(&mut self, micro: &MicroDeclaration) -> Vec<TypeDiagnostic> {
        let saved_env = self.env.clone();
        self.env.variables.clear();

        let mut param_types = Vec::new();

        for param in &micro.params {
            let param_ty = param.ty.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);
            self.env.insert_variable(param.name.name.clone(), param_ty.clone());
            param_types.push(param_ty);
        }

        let return_type = micro.return_type.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);

        self.current_return_type = Some(return_type.clone());

        self.check_block(&micro.body);

        let sig = FunctionSignature { name: micro.name.name.clone(), param_types, return_type: return_type.clone() };
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
                            suggestion: Some(format!(
                                "Consider changing the type annotation to {} or converting the expression to {}",
                                expr_ty, annotated_ty
                            )),
                        });
                    }
                }
                if let Pattern::Variable(var) = &let_stmt.pattern {
                    let var_ty = if let Some(ty) = &let_stmt.ty { self.type_expr_to_type_info(ty) } else { expr_ty };
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

            TermExpression::ApplyCall { callee, args, span } => self.check_apply_call(callee, args, span.start, span.end),

            TermExpression::If { condition, then_branch, else_branch, span, .. } => {
                self.check_if_expr(condition, then_branch, else_branch, *span)
            }

            TermExpression::Return(ret) => {
                if let Some(return_expr) = ret.base.as_ref() {
                    let return_ty = self.infer_expr(return_expr);
                    if let Some(expected) = &self.current_return_type {
                        if *expected != TypeInfo::Unknown && return_ty != TypeInfo::Unknown && return_ty != *expected {
                            self.diagnostics.push(TypeDiagnostic {
                                message: format!("Return type mismatch: expected {} but found {}", expected, return_ty),
                                severity: DiagnosticSeverity::Error,
                                span_start: ret.span.start,
                                span_end: ret.span.end,
                                suggestion: Some(format!("Expected return type {} but found {}", expected, return_ty)),
                            });
                        }
                    }
                    return_ty
                }
                else {
                    if let Some(expected) = &self.current_return_type {
                        if *expected != TypeInfo::Unknown && *expected != TypeInfo::Null {
                            self.diagnostics.push(TypeDiagnostic {
                                message: format!("Return type mismatch: expected {} but found Null", expected),
                                severity: DiagnosticSeverity::Error,
                                span_start: ret.span.start,
                                span_end: ret.span.end,
                                suggestion: Some(format!("Expected return type {} but found Null", expected)),
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
                            suggestion: Some(format!("Convert the loop condition to Bool, found {}", cond_ty)),
                        });
                    }
                }
                self.check_block(body);
                TypeInfo::Null
            }

            TermExpression::DotCall { receiver, field, span, .. } => {
                let receiver_ty = self.infer_expr(receiver);
                match &receiver_ty {
                    TypeInfo::Object(type_name) | TypeInfo::Component(type_name) => {
                        if let Some(fields) = self.env.class_fields.get(type_name) {
                            if let Some(field_ty) = fields.get(&field.name) {
                                field_ty.clone()
                            } else {
                                let candidates: Vec<String> = fields.keys().cloned().collect();
                                let similar = Self::find_similar_names(&field.name, &candidates, 3);
                                let suggestion = if similar.is_empty() { None } else { Some(format!("Did you mean {}?", similar.join(", "))) };
                                self.diagnostics.push(TypeDiagnostic {
                                    message: format!("Type {} has no field '{}'", type_name, field.name),
                                    severity: DiagnosticSeverity::Error,
                                    span_start: span.start,
                                    span_end: span.end,
                                    suggestion,
                                });
                                TypeInfo::Unknown
                            }
                        } else {
                            TypeInfo::Unknown
                        }
                    }
                    _ => TypeInfo::Unknown,
                }
            }

            TermExpression::Index { receiver, index, .. } => {
                let receiver_ty = self.infer_expr(receiver);
                let _index_ty = self.infer_expr(index);
                match receiver_ty {
                    TypeInfo::Array(elem_ty) => *elem_ty,
                    TypeInfo::Map(_, val_ty) => *val_ty,
                    TypeInfo::Generic(name, args) => match name.as_str() {
                        "Array" | "List" => args.first().cloned().unwrap_or(TypeInfo::Unknown),
                        "Map" | "Dict" => args.get(1).cloned().unwrap_or(TypeInfo::Unknown),
                        _ => TypeInfo::Unknown,
                    },
                    _ => TypeInfo::Unknown,
                }
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
                let outer_vars = self.env.variables.clone();
                let mut param_types = Vec::new();
                let mut param_names = Vec::new();
                for param in &lambda.params {
                    let param_ty = param.ty.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or(TypeInfo::Unknown);
                    self.env.insert_variable(param.name.name.clone(), param_ty.clone());
                    param_types.push(param_ty);
                    param_names.push(param.name.name.clone());
                }
                let return_type = lambda.return_type.as_ref().map(|t| self.type_expr_to_type_info(t)).unwrap_or_else(|| {
                    let saved_return = self.current_return_type.take();
                    let inferred = self.infer_block(&lambda.body);
                    self.current_return_type = saved_return;
                    inferred
                });
                let referenced = Self::collect_variable_references(&TermExpression::Micro(lambda.clone()));
                let captures = Self::infer_closure_captures(&referenced, &outer_vars, &param_names);
                self.env.variables = outer_vars;
                if captures.is_empty() {
                    TypeInfo::Function { param_types, return_type: Box::new(return_type) }
                }
                else {
                    TypeInfo::Closure { param_types, return_type: Box::new(return_type), captures }
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
                None => {
                    let candidates: Vec<String> = self.env.variables.keys().cloned().collect();
                    let similar = Self::find_similar_names(name, &candidates, 3);
                    let suggestion =
                        if similar.is_empty() { None } else { Some(format!("Did you mean {}?", similar.join(", "))) };
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Undefined variable: {}", name),
                        severity: DiagnosticSeverity::Warning,
                        span_start: name_path.span.start,
                        span_end: name_path.span.end,
                        suggestion,
                    });
                    TypeInfo::Unknown
                }
            }
        }
        else {
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
            }
            else if content.parse::<i64>().is_ok() {
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
            ValkyrieTokenType::Plus
            | ValkyrieTokenType::Minus
            | ValkyrieTokenType::Star
            | ValkyrieTokenType::Slash
            | ValkyrieTokenType::Percent => {
                if *lhs_ty == TypeInfo::Float || *rhs_ty == TypeInfo::Float {
                    TypeInfo::Float
                }
                else if *lhs_ty == TypeInfo::Int && *rhs_ty == TypeInfo::Int {
                    TypeInfo::Int
                }
                else if *lhs_ty == TypeInfo::String && *rhs_ty == TypeInfo::String && *op == ValkyrieTokenType::Plus {
                    TypeInfo::String
                }
                else if *lhs_ty == TypeInfo::Unknown || *rhs_ty == TypeInfo::Unknown {
                    TypeInfo::Unknown
                }
                else {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Cannot apply operator {:?} to types {} and {}", op, lhs_ty, rhs_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                        suggestion: Some("Consider converting operands to compatible types".to_string()),
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
                if *lhs_ty != TypeInfo::Unknown && *rhs_ty != TypeInfo::Unknown && lhs_ty != rhs_ty {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Comparison between different types: {} and {}", lhs_ty, rhs_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                        suggestion: Some(format!(
                            "Comparing different types may not work as expected: {} vs {}",
                            lhs_ty, rhs_ty
                        )),
                    });
                }
                TypeInfo::Bool
            }
            ValkyrieTokenType::AndAnd | ValkyrieTokenType::OrOr => {
                if *lhs_ty != TypeInfo::Unknown && *lhs_ty != TypeInfo::Bool {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Logical operator requires Bool, found {} on left side", lhs_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                        suggestion: Some("Logical operators require Bool operands".to_string()),
                    });
                }
                if *rhs_ty != TypeInfo::Unknown && *rhs_ty != TypeInfo::Bool {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Logical operator requires Bool, found {} on right side", rhs_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                        suggestion: Some("Logical operators require Bool operands".to_string()),
                    });
                }
                TypeInfo::Bool
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// 推断一元运算表达式的类型
    fn infer_unary(&mut self, op: &ValkyrieTokenType, base_ty: &TypeInfo, span_start: usize, span_end: usize) -> TypeInfo {
        match op {
            ValkyrieTokenType::Minus => {
                if *base_ty == TypeInfo::Int || *base_ty == TypeInfo::Float {
                    base_ty.clone()
                }
                else if *base_ty == TypeInfo::Unknown {
                    TypeInfo::Unknown
                }
                else {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("Cannot negate type {}", base_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start,
                        span_end,
                        suggestion: Some("Negation requires Int or Float".to_string()),
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
                        suggestion: Some("Logical NOT requires Bool".to_string()),
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
                            suggestion: Some(format!("Function '{}' expects {} argument(s)", name, sig.param_types.len())),
                        });
                    }
                    else {
                        for (i, (arg_ty, param_ty)) in arg_types.iter().zip(sig.param_types.iter()).enumerate() {
                            if *arg_ty != TypeInfo::Unknown && *param_ty != TypeInfo::Unknown && arg_ty != param_ty {
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
                                    suggestion: Some(format!("Argument {} should be of type {}", i + 1, param_ty)),
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
                suggestion: Some(format!("Convert the condition to Bool, found {}", cond_ty)),
            });
        }

        self.check_block(then_branch);
        let then_ty = self.infer_block(then_branch);

        if let Some(else_br) = else_branch {
            self.check_block(else_br);
            let else_ty = self.infer_block(else_br);
            let unified = Self::unify_types(&then_ty, &else_ty);
            if then_ty != TypeInfo::Unknown && else_ty != TypeInfo::Unknown && then_ty != else_ty {
                if unified == TypeInfo::Unknown {
                    self.diagnostics.push(TypeDiagnostic {
                        message: format!("If branches have incompatible types: {} and {}", then_ty, else_ty),
                        severity: DiagnosticSeverity::Warning,
                        span_start: span.start,
                        span_end: span.end,
                        suggestion: Some("Ensure both branches return the same type".to_string()),
                    });
                }
            }
            unified
        }
        else {
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
                        let var_ty = if let Some(ty) = &let_stmt.ty { self.type_expr_to_type_info(ty) } else { expr_ty };
                        self.env.insert_variable(var.name.name.clone(), var_ty);
                    }
                    last_ty = TypeInfo::Null;
                }
                Statement::ExprStmt(expr_stmt) => {
                    let ty = self.infer_expr(&expr_stmt.expr);
                    if !expr_stmt.semi {
                        last_ty = ty;
                    }
                    else {
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
                        "Array" | "List" => TypeInfo::Generic(name.clone(), vec![TypeInfo::Unknown]),
                        "Map" | "Dict" => TypeInfo::Generic(name.clone(), vec![TypeInfo::Unknown, TypeInfo::Unknown]),
                        _ => self.env.lookup_type(name).cloned().unwrap_or(TypeInfo::Unknown),
                    }
                }
                else {
                    TypeInfo::Unknown
                }
            }
            oak_valkyrie::ast::TypeExpression::Function(func_type) => {
                let param_types: Vec<TypeInfo> = func_type.params.iter().map(|t| self.type_expr_to_type_info(t)).collect();
                let return_type = self.type_expr_to_type_info(&func_type.return_type);
                TypeInfo::Function { param_types, return_type: Box::new(return_type) }
            }
            oak_valkyrie::ast::TypeExpression::Optional(opt_type) => {
                TypeInfo::Optional(Box::new(self.type_expr_to_type_info(&opt_type.inner)))
            }
            oak_valkyrie::ast::TypeExpression::Generic(r#gen) => {
                let name = &r#gen.name.name;
                let args: Vec<TypeInfo> = r#gen.args.iter().map(|t| self.type_expr_to_type_info(t)).collect();
                TypeInfo::Generic(name.clone(), args)
            }
            oak_valkyrie::ast::TypeExpression::Tuple(tuple_type) => {
                let elements: Vec<TypeInfo> = tuple_type.elements.iter().map(|t| self.type_expr_to_type_info(t)).collect();
                TypeInfo::Tuple(elements)
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// 递归收集表达式中所有 NamePath 引用的变量名
    ///
    /// 遍历表达式的所有子节点，提取单段 NamePath（即局部变量引用）的名称。
    pub fn collect_variable_references(expr: &TermExpression) -> Vec<String> {
        let mut refs = Vec::new();
        Self::collect_refs_recursive(expr, &mut refs);
        refs
    }

    fn collect_refs_recursive(expr: &TermExpression, refs: &mut Vec<String>) {
        match expr {
            TermExpression::NamePath(name_path) => {
                if name_path.parts.len() == 1 {
                    refs.push(name_path.parts[0].name.clone());
                }
            }
            TermExpression::Binary(node) => {
                Self::collect_refs_recursive(&node.lhs, refs);
                Self::collect_refs_recursive(&node.rhs, refs);
            }
            TermExpression::Unary(node) => {
                Self::collect_refs_recursive(&node.base, refs);
            }
            TermExpression::ApplyCall { callee, args, .. } => {
                Self::collect_refs_recursive(callee, refs);
                for arg in args {
                    Self::collect_refs_recursive(arg, refs);
                }
            }
            TermExpression::DotCall { receiver, .. } => {
                Self::collect_refs_recursive(receiver, refs);
            }
            TermExpression::Index { receiver, index, .. } => {
                Self::collect_refs_recursive(receiver, refs);
                Self::collect_refs_recursive(index, refs);
            }
            TermExpression::Offset { receiver, offset, .. } => {
                Self::collect_refs_recursive(receiver, refs);
                Self::collect_refs_recursive(offset, refs);
            }
            TermExpression::Paren { expr: inner, .. } => {
                Self::collect_refs_recursive(inner, refs);
            }
            TermExpression::Block(block) => {
                Self::collect_refs_from_block(block, refs);
            }
            TermExpression::Micro(lambda) => {
                Self::collect_refs_from_block(&lambda.body, refs);
            }
            TermExpression::Object { fields, .. } => {
                for (_name, value_expr) in fields {
                    if let Some(expr) = value_expr {
                        Self::collect_refs_recursive(expr, refs);
                    }
                }
            }
            TermExpression::If { condition, then_branch, else_branch, .. } => {
                Self::collect_refs_recursive(condition, refs);
                Self::collect_refs_from_block(then_branch, refs);
                if let Some(else_br) = else_branch {
                    Self::collect_refs_from_block(else_br, refs);
                }
            }
            TermExpression::Match { scrutinee, arms, .. } => {
                Self::collect_refs_recursive(scrutinee, refs);
                for arm in arms {
                    Self::collect_refs_recursive(&arm.body, refs);
                }
            }
            TermExpression::Loop { condition, body, .. } => {
                if let Some(cond) = condition {
                    Self::collect_refs_recursive(cond, refs);
                }
                Self::collect_refs_from_block(body, refs);
            }
            TermExpression::Return(ret) => {
                if let Some(base) = &ret.base {
                    Self::collect_refs_recursive(base, refs);
                }
            }
            TermExpression::Break(brk) => {
                if let Some(base) = &brk.base {
                    Self::collect_refs_recursive(base, refs);
                }
            }
            TermExpression::Yield { expr, .. } => {
                if let Some(inner) = expr {
                    Self::collect_refs_recursive(inner, refs);
                }
            }
            TermExpression::Raise(raise) => {
                if let Some(base) = &raise.base {
                    Self::collect_refs_recursive(base, refs);
                }
            }
            TermExpression::Resume(resume) => {
                if let Some(base) = &resume.base {
                    Self::collect_refs_recursive(base, refs);
                }
            }
            TermExpression::Catch { expr, arms, .. } => {
                Self::collect_refs_recursive(expr, refs);
                for arm in arms {
                    Self::collect_refs_recursive(&arm.body, refs);
                }
            }
            TermExpression::With { base, updates, .. } => {
                Self::collect_refs_recursive(base, refs);
                for (_name, value_expr) in updates {
                    Self::collect_refs_recursive(value_expr, refs);
                }
            }
            TermExpression::SuperCall { args, .. } => {
                for arg in args {
                    Self::collect_refs_recursive(arg, refs);
                }
            }
            TermExpression::AnonymousClass(_) | TermExpression::Bool { .. } | TermExpression::StringLiteral(_) => {}
            TermExpression::Continue(_) => {}
        }
    }

    fn collect_refs_from_block(block: &Block, refs: &mut Vec<String>) {
        for stmt in &block.statements {
            match stmt {
                Statement::Let(let_stmt) => {
                    Self::collect_refs_recursive(&let_stmt.expr, refs);
                }
                Statement::ExprStmt(expr_stmt) => {
                    Self::collect_refs_recursive(&expr_stmt.expr, refs);
                }
            }
        }
    }

    /// 推断闭包捕获的变量列表
    ///
    /// 根据表达式中引用的变量名、外部变量映射和 lambda 参数名，
    /// 确定哪些外部变量被闭包捕获，并返回捕获变量的名称与类型列表。
    pub fn infer_closure_captures(
        referenced: &[String],
        outer_vars: &HashMap<String, TypeInfo>,
        param_names: &[String],
    ) -> Vec<(String, TypeInfo)> {
        let mut captures = Vec::new();
        let mut seen = HashSet::new();
        for name in referenced {
            if seen.contains(name) {
                continue;
            }
            if param_names.contains(name) {
                continue;
            }
            if let Some(ty) = outer_vars.get(name) {
                seen.insert(name.clone());
                captures.push((name.clone(), ty.clone()));
            }
        }
        captures
    }

    /// 统一两个类型，返回最具体的公共类型
    ///
    /// 类型统一规则：
    /// - 两个相同类型 → 返回该类型
    /// - Int + Float → Float（数值提升）
    /// - 任一方为 Unknown → 返回另一方
    /// - Generic("Array", [a]) + Generic("Array", [b]) → 递归统一 a 和 b
    /// - 不兼容类型 → 返回 Unknown
    pub fn unify_types(a: &TypeInfo, b: &TypeInfo) -> TypeInfo {
        if a == b {
            return a.clone();
        }
        match (a, b) {
            (TypeInfo::Unknown, other) | (other, TypeInfo::Unknown) => other.clone(),
            (TypeInfo::Int, TypeInfo::Float) | (TypeInfo::Float, TypeInfo::Int) => TypeInfo::Float,
            (TypeInfo::Generic(name_a, args_a), TypeInfo::Generic(name_b, args_b)) => {
                if name_a == name_b && args_a.len() == args_b.len() {
                    let unified_args: Vec<TypeInfo> =
                        args_a.iter().zip(args_b.iter()).map(|(ta, tb)| Self::unify_types(ta, tb)).collect();
                    TypeInfo::Generic(name_a.clone(), unified_args)
                }
                else {
                    TypeInfo::Unknown
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identifier(name: &str) -> oak_valkyrie::ast::Identifier {
        oak_valkyrie::ast::Identifier { name: name.to_string(), span: oak_core::Range::default() }
    }

    fn make_namepath_type(name: &str) -> oak_valkyrie::ast::TypeExpression {
        oak_valkyrie::ast::TypeExpression::Namepath(Box::new(oak_valkyrie::ast::NamePath {
            parts: vec![make_identifier(name)],
            span: oak_core::Range::default(),
        }))
    }

    fn make_generic_type(name: &str, args: Vec<oak_valkyrie::ast::TypeExpression>) -> oak_valkyrie::ast::TypeExpression {
        oak_valkyrie::ast::TypeExpression::Generic(Box::new(oak_valkyrie::ast::GenericType {
            name: make_identifier(name),
            args,
            span: oak_core::Range::default(),
        }))
    }

    #[test]
    fn test_type_expr_to_type_info_array_int() {
        let checker = TypeChecker::new();
        let type_expr = make_generic_type("Array", vec![make_namepath_type("Int")]);
        let result = checker.type_expr_to_type_info(&type_expr);
        assert_eq!(result, TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]));
    }

    #[test]
    fn test_type_expr_to_type_info_map_string_int() {
        let checker = TypeChecker::new();
        let type_expr = make_generic_type("Map", vec![make_namepath_type("String"), make_namepath_type("Int")]);
        let result = checker.type_expr_to_type_info(&type_expr);
        assert_eq!(result, TypeInfo::Generic("Map".to_string(), vec![TypeInfo::String, TypeInfo::Int]));
    }

    #[test]
    fn test_type_expr_to_type_info_list_float() {
        let checker = TypeChecker::new();
        let type_expr = make_generic_type("List", vec![make_namepath_type("Float")]);
        let result = checker.type_expr_to_type_info(&type_expr);
        assert_eq!(result, TypeInfo::Generic("List".to_string(), vec![TypeInfo::Float]));
    }

    #[test]
    fn test_type_expr_to_type_info_nested_generic() {
        let checker = TypeChecker::new();
        let inner = make_generic_type("Array", vec![make_namepath_type("Int")]);
        let type_expr = make_generic_type("Map", vec![make_namepath_type("String"), inner]);
        let result = checker.type_expr_to_type_info(&type_expr);
        assert_eq!(
            result,
            TypeInfo::Generic(
                "Map".to_string(),
                vec![TypeInfo::String, TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]),],
            )
        );
    }

    #[test]
    fn test_type_expr_to_type_info_namepath_array() {
        let checker = TypeChecker::new();
        let type_expr = make_namepath_type("Array");
        let result = checker.type_expr_to_type_info(&type_expr);
        assert_eq!(result, TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Unknown]));
    }

    #[test]
    fn test_type_expr_to_type_info_namepath_map() {
        let checker = TypeChecker::new();
        let type_expr = make_namepath_type("Map");
        let result = checker.type_expr_to_type_info(&type_expr);
        assert_eq!(result, TypeInfo::Generic("Map".to_string(), vec![TypeInfo::Unknown, TypeInfo::Unknown]));
    }

    #[test]
    fn test_unify_types_same_type() {
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Int, &TypeInfo::Int), TypeInfo::Int);
        assert_eq!(TypeChecker::unify_types(&TypeInfo::String, &TypeInfo::String), TypeInfo::String);
    }

    #[test]
    fn test_unify_types_int_float_promotion() {
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Int, &TypeInfo::Float), TypeInfo::Float);
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Float, &TypeInfo::Int), TypeInfo::Float);
    }

    #[test]
    fn test_unify_types_unknown_propagation() {
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Unknown, &TypeInfo::Int), TypeInfo::Int);
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Int, &TypeInfo::Unknown), TypeInfo::Int);
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Unknown, &TypeInfo::Unknown), TypeInfo::Unknown);
    }

    #[test]
    fn test_unify_types_incompatible() {
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Int, &TypeInfo::String), TypeInfo::Unknown);
        assert_eq!(TypeChecker::unify_types(&TypeInfo::Bool, &TypeInfo::String), TypeInfo::Unknown);
    }

    #[test]
    fn test_unify_types_generic_array() {
        let a = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]);
        let b = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Float]);
        let expected = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Float]);
        assert_eq!(TypeChecker::unify_types(&a, &b), expected);
    }

    #[test]
    fn test_unify_types_generic_array_unknown() {
        let a = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Unknown]);
        let b = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]);
        let expected = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]);
        assert_eq!(TypeChecker::unify_types(&a, &b), expected);
    }

    #[test]
    fn test_unify_types_generic_incompatible_names() {
        let a = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]);
        let b = TypeInfo::Generic("Map".to_string(), vec![TypeInfo::Int, TypeInfo::Int]);
        assert_eq!(TypeChecker::unify_types(&a, &b), TypeInfo::Unknown);
    }

    #[test]
    fn test_unify_types_generic_different_arg_count() {
        let a = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int]);
        let b = TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int, TypeInfo::Int]);
        assert_eq!(TypeChecker::unify_types(&a, &b), TypeInfo::Unknown);
    }

    #[test]
    fn test_unify_types_nested_generic() {
        let a = TypeInfo::Generic(
            "Map".to_string(),
            vec![TypeInfo::String, TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Int])],
        );
        let b = TypeInfo::Generic(
            "Map".to_string(),
            vec![TypeInfo::String, TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Float])],
        );
        let expected = TypeInfo::Generic(
            "Map".to_string(),
            vec![TypeInfo::String, TypeInfo::Generic("Array".to_string(), vec![TypeInfo::Float])],
        );
        assert_eq!(TypeChecker::unify_types(&a, &b), expected);
    }
}
