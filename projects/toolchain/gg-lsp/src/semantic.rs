//! Valkyrie 脚本语义分析模块。
//!
//! 提供符号表构建、悬停信息查询和语义诊断等功能。
//! 包含基于文本模式匹配的简单分析器和基于 AST 的精确分析器。

use std::collections::HashMap;

use gg_script::type_checker::{TypeChecker, TypeEnvironment, TypeInfo, DiagnosticSeverity as TypeDiagnosticSeverity};

/// 符号类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    /// 变量符号
    Variable,
    /// 函数符号
    Function,
    /// 命名空间符号
    Namespace,
    /// 参数符号
    Parameter,
}

impl SymbolKind {
    /// 返回符号类型的显示名称。
    pub fn display_name(&self) -> &'static str {
        match self {
            SymbolKind::Variable => "variable",
            SymbolKind::Function => "function",
            SymbolKind::Namespace => "namespace",
            SymbolKind::Parameter => "parameter",
        }
    }
}

/// 符号定义。
#[derive(Debug, Clone)]
pub struct Symbol {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub kind: SymbolKind,
    /// 定义所在行号（0-indexed）
    pub line: usize,
    /// 定义所在列号（0-indexed）
    pub column: usize,
    /// 类型信息
    pub type_info: Option<String>,
}

/// 符号表。
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    /// 所有符号列表
    pub symbols: Vec<Symbol>,
    /// 按名称索引的符号位置映射
    pub by_name: HashMap<String, Vec<usize>>,
}

impl SymbolTable {
    /// 创建空的符号表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 向符号表中添加一个符号。
    pub fn insert(&mut self, symbol: Symbol) {
        let idx = self.symbols.len();
        self.by_name.entry(symbol.name.clone()).or_default().push(idx);
        self.symbols.push(symbol);
    }

    /// 根据名称查找符号定义，返回第一个匹配项。
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.by_name.get(name).and_then(|indices| indices.first()).and_then(|&idx| self.symbols.get(idx))
    }

    /// 根据名称查找所有同名符号定义。
    pub fn get_all(&self, name: &str) -> Vec<&Symbol> {
        self.by_name
            .get(name)
            .map(|indices| indices.iter().filter_map(|&idx| self.symbols.get(idx)).collect())
            .unwrap_or_default()
    }
}

/// 悬停信息。
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// 悬停内容（Markdown 格式）
    pub contents: String,
    /// 悬停范围（起始字节偏移, 结束字节偏移）
    pub range: Option<(usize, usize)>,
}

/// 诊断严重程度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 信息
    Information,
}

/// 语义诊断信息。
#[derive(Debug, Clone)]
pub struct SemanticDiagnostic {
    /// 诊断消息
    pub message: String,
    /// 诊断所在行号（0-indexed）
    pub line: usize,
    /// 诊断严重程度
    pub severity: DiagnosticSeverity,
}

/// 语义分析结果。
#[derive(Debug, Clone)]
pub struct SemanticResult {
    /// 符号表
    pub symbol_table: SymbolTable,
    /// 诊断列表
    pub diagnostics: Vec<SemanticDiagnostic>,
}

/// 语义分析器。
///
/// 对 Valkyrie 源码进行基于文本模式匹配的语义分析，
/// 提取符号定义并收集语义诊断信息。
pub struct SemanticAnalyzer {
    /// 上一次分析的符号表
    symbol_table: SymbolTable,
    /// 上一次分析的诊断列表
    diagnostics: Vec<SemanticDiagnostic>,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
    /// 创建新的语义分析器。
    pub fn new() -> Self {
        Self { symbol_table: SymbolTable::new(), diagnostics: Vec::new() }
    }

    /// 分析源码，构建符号表并收集诊断信息。
    ///
    /// 使用简单的文本模式匹配提取以下符号：
    /// - `micro name(params)` → Function 符号
    /// - `let name` / `let mut name` → Variable 符号
    /// - `namespace Name` → Namespace 符号
    ///
    /// 同时收集以下诊断：
    /// - 未定义的变量引用（启发式：标识符不在符号表中）
    pub fn analyze(&mut self, source: &str) -> SemanticResult {
        self.symbol_table = SymbolTable::new();
        self.diagnostics = Vec::new();

        let mut param_names: Vec<String> = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("namespace ") {
                if let Some(name) = extract_identifier(rest) {
                    let column = line.find(&name).unwrap_or(0);
                    self.symbol_table.insert(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Namespace,
                        line: line_idx,
                        column,
                        type_info: None,
                    });
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("micro ") {
                if let Some(name) = extract_identifier(rest) {
                    let column = line.find(&name).unwrap_or(0);
                    let params = extract_params_from_line(rest);
                    let type_info = if params.is_empty() { None } else { Some(format!("({})", params.join(", "))) };
                    self.symbol_table.insert(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Function,
                        line: line_idx,
                        column,
                        type_info: type_info.clone(),
                    });
                    for param in &params {
                        param_names.push(param.clone());
                        let param_col = line.find(param).unwrap_or(0);
                        self.symbol_table.insert(Symbol {
                            name: param.clone(),
                            kind: SymbolKind::Parameter,
                            line: line_idx,
                            column: param_col,
                            type_info: None,
                        });
                    }
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("let ") {
                let rest = if let Some(r) = rest.strip_prefix("mut ") { r } else { rest };
                if let Some(name) = extract_identifier(rest) {
                    let column = line.find(&name).unwrap_or(0);
                    let type_info = extract_type_from_let(rest);
                    self.symbol_table.insert(Symbol { name, kind: SymbolKind::Variable, line: line_idx, column, type_info });
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("const ") {
                if let Some(name) = extract_identifier(rest) {
                    let column = line.find(&name).unwrap_or(0);
                    let type_info = extract_type_from_let(rest);
                    self.symbol_table.insert(Symbol { name, kind: SymbolKind::Variable, line: line_idx, column, type_info });
                }
            }
        }

        self.collect_undefined_references(source);

        SemanticResult { symbol_table: self.symbol_table.clone(), diagnostics: self.diagnostics.clone() }
    }

    /// 根据名称查找符号定义。
    pub fn get_definition(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.get(name)
    }

    /// 获取指定位置的悬停信息。
    ///
    /// 根据行号和列号定位源码中的标识符，
    /// 并在符号表中查找对应的符号定义以生成悬停内容。
    pub fn get_hover_info(&self, line: usize, column: usize, source: &str) -> Option<HoverInfo> {
        let line_text = source.lines().nth(line)?;
        let ident = extract_identifier_at(line_text, column)?;
        let symbol = self.symbol_table.get(&ident)?;

        let mut contents = String::new();
        match symbol.kind {
            SymbolKind::Function => {
                contents.push_str(&format!("**function** `{}`", symbol.name));
                if let Some(ref sig) = symbol.type_info {
                    contents.push_str(&format!("`{}`", sig));
                }
            }
            SymbolKind::Variable => {
                contents.push_str(&format!("**variable** `{}`", symbol.name));
                if let Some(ref ty) = symbol.type_info {
                    contents.push_str(&format!(": `{}`", ty));
                }
            }
            SymbolKind::Namespace => {
                contents.push_str(&format!("**namespace** `{}`", symbol.name));
            }
            SymbolKind::Parameter => {
                contents.push_str(&format!("**parameter** `{}`", symbol.name));
            }
        }

        let start = source.lines().take(line).map(|l| l.len() + 1).sum::<usize>() + column;
        let end = start + ident.len();

        Some(HoverInfo { contents, range: Some((start, end)) })
    }

    /// 获取上一次分析的诊断列表。
    pub fn get_diagnostics(&self) -> Vec<SemanticDiagnostic> {
        self.diagnostics.clone()
    }

    /// 收集未定义变量引用的诊断信息。
    ///
    /// 启发式规则：对于每一行中的标识符引用，
    /// 如果标识符不是关键字、内置函数或已定义符号，
    /// 则报告为未定义引用。
    fn collect_undefined_references(&mut self, source: &str) {
        let keywords = [
            "namespace",
            "micro",
            "let",
            "const",
            "fn",
            "if",
            "else",
            "while",
            "for",
            "return",
            "break",
            "continue",
            "true",
            "false",
            "null",
            "struct",
            "enum",
            "impl",
            "trait",
            "pub",
            "use",
            "mod",
            "import",
            "export",
            "from",
            "as",
            "in",
            "match",
            "loop",
            "mut",
        ];
        let builtins = ["print", "len", "push", "pop", "typeof", "to_string", "to_int", "to_float"];

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            let mut pos = 0;
            let chars: Vec<char> = trimmed.chars().collect();

            while pos < chars.len() {
                let ch = chars[pos];
                if ch.is_ascii_alphabetic() || ch == '_' {
                    let start = pos;
                    while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                        pos += 1;
                    }
                    let ident: String = chars[start..pos].iter().collect();

                    if keywords.contains(&ident.as_str()) || builtins.contains(&ident.as_str()) {
                        continue;
                    }

                    if self.symbol_table.get(&ident).is_some() {
                        continue;
                    }

                    let is_declaration = is_declaration_line(trimmed, &ident);
                    let is_type_annotation = is_type_like(&ident);
                    let is_string_content = is_inside_string(trimmed, start);

                    if is_declaration || is_type_annotation || is_string_content {
                        continue;
                    }

                    self.diagnostics.push(SemanticDiagnostic {
                        message: format!("undefined variable: `{}`", ident),
                        line: line_idx,
                        severity: DiagnosticSeverity::Warning,
                    });
                }
                else {
                    pos += 1;
                }
            }
        }
    }
}

/// 从字符串开头提取标识符。
fn extract_identifier(s: &str) -> Option<String> {
    let s = s.trim_start();
    let end = s
        .char_indices()
        .take_while(|&(_, c)| c.is_ascii_alphanumeric() || c == '_')
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    if end > 0 {
        let ident = &s[..end];
        if ident.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
            return Some(ident.to_string());
        }
    }
    None
}

/// 从 micro 声明行中提取参数名列表。
fn extract_params_from_line(rest: &str) -> Vec<String> {
    let mut params = Vec::new();
    if let Some(start) = rest.find('(') {
        if let Some(end) = rest.find(')') {
            let params_str = &rest[start + 1..end];
            for param in params_str.split(',') {
                let trimmed = param.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let name_part = trimmed.split(':').next().unwrap_or(trimmed);
                if let Some(name) = extract_identifier(name_part) {
                    params.push(name);
                }
            }
        }
    }
    params
}

/// 从 let 声明中提取类型注解。
fn extract_type_from_let(rest: &str) -> Option<String> {
    if let Some(colon_pos) = rest.find(':') {
        let type_part = rest[colon_pos + 1..].trim();
        let type_str = type_part.split(|c: char| c == '=' || c == ',').next().unwrap_or(type_part).trim();
        if !type_str.is_empty() {
            return Some(type_str.to_string());
        }
    }
    None
}

/// 从指定行文本的指定列位置提取标识符。
fn extract_identifier_at(line: &str, column: usize) -> Option<String> {
    let byte_offset = line.char_indices().take(column).map(|(i, c)| i + c.len_utf8()).last().unwrap_or(0);

    let remaining = line.get(byte_offset..)?;
    let ident = extract_identifier(remaining)?;
    Some(ident)
}

/// 判断标识符所在行是否为声明行。
fn is_declaration_line(line: &str, ident: &str) -> bool {
    let decl_prefixes = ["let ", "let mut ", "const ", "micro ", "namespace ", "fn "];
    for prefix in &decl_prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            if rest.trim_start().starts_with(ident) {
                return true;
            }
            if line.contains('(') && line.contains(')') {
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.find(')') {
                        let params = &line[start + 1..end];
                        for param in params.split(',') {
                            let trimmed = param.trim();
                            let name_part = trimmed.split(':').next().unwrap_or(trimmed).trim();
                            if name_part == ident {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// 判断标识符是否像类型名称（首字母大写）。
fn is_type_like(ident: &str) -> bool {
    ident.starts_with(|c: char| c.is_ascii_uppercase())
}

/// 判断指定位置是否在字符串字面量内。
fn is_inside_string(line: &str, char_pos: usize) -> bool {
    let mut in_string = false;
    let mut pos = 0;
    let chars: Vec<char> = line.chars().collect();
    while pos < char_pos && pos < chars.len() {
        if chars[pos] == '"' {
            if pos > 0 && chars[pos - 1] == '\\' {
                pos += 1;
                continue;
            }
            in_string = !in_string;
        }
        pos += 1;
    }
    in_string
}

/// 基于 AST 的语义分析器。
///
/// 使用 Valkyrie 解析器将源码解析为 AST，然后通过 TypeChecker
/// 进行精确的符号提取和类型推断，提供比文本模式匹配更准确的语义信息。
pub struct AstSemanticAnalyzer {
    /// 上一次分析的符号表
    symbol_table: SymbolTable,
    /// 上一次分析的诊断列表
    diagnostics: Vec<SemanticDiagnostic>,
    /// 上一次分析的类型环境
    type_env: TypeEnvironment,
}

impl Default for AstSemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl AstSemanticAnalyzer {
    /// 创建新的 AST 语义分析器。
    pub fn new() -> Self {
        Self { symbol_table: SymbolTable::new(), diagnostics: Vec::new(), type_env: TypeEnvironment::new() }
    }

    /// 分析源码，基于 AST 构建符号表并收集诊断信息。
    ///
    /// 使用 Valkyrie 解析器解析源码，然后通过 TypeChecker 进行类型检查，
    /// 提取精确的符号定义（函数签名、类字段、组件定义、枚举变体等）
    /// 和类型推断结果。
    pub fn analyze(&mut self, source: &str) -> SemanticResult {
        self.symbol_table = SymbolTable::new();
        self.diagnostics = Vec::new();

        let compiler = gg_script::ScriptCompiler::new();
        let (_, type_diagnostics) = compiler.compile_to_ir_with_diagnostics(source, "module");

        for diag in &type_diagnostics {
            self.diagnostics.push(SemanticDiagnostic {
                message: if let Some(suggestion) = &diag.suggestion {
                    format!("{} (suggestion: {})", diag.message, suggestion)
                } else {
                    diag.message.clone()
                },
                line: 0,
                severity: match diag.severity {
                    TypeDiagnosticSeverity::Error => DiagnosticSeverity::Error,
                    TypeDiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
                },
            });
        }

        let mut checker = TypeChecker::new();
        let root = match gg_script::parse_source(source) {
            Ok(r) => r,
            Err(_) => return SemanticResult { symbol_table: self.symbol_table.clone(), diagnostics: self.diagnostics.clone() },
        };
        checker.check_root(&root);

        for (name, sig) in &checker.env.function_signatures {
            let params: Vec<String> = sig.param_types.iter().map(|t| type_info_to_string(t)).collect();
            let return_type = type_info_to_string(&sig.return_type);
            self.symbol_table.insert(Symbol {
                name: name.clone(),
                kind: SymbolKind::Function,
                line: 0,
                column: 0,
                type_info: Some(format!("({}) -> {}", params.join(", "), return_type)),
            });
        }

        for (name, ty) in &checker.env.types {
            let kind = match ty {
                TypeInfo::Object(_) => SymbolKind::Namespace,
                TypeInfo::Trait(_) => SymbolKind::Namespace,
                TypeInfo::Component(_) => SymbolKind::Namespace,
                _ => SymbolKind::Variable,
            };
            self.symbol_table.insert(Symbol {
                name: name.clone(),
                kind,
                line: 0,
                column: 0,
                type_info: Some(type_info_to_string(ty)),
            });
        }

        for (name, ty) in &checker.env.variables {
            self.symbol_table.insert(Symbol {
                name: name.clone(),
                kind: SymbolKind::Variable,
                line: 0,
                column: 0,
                type_info: Some(type_info_to_string(ty)),
            });
        }

        for diag in &checker.diagnostics {
            self.diagnostics.push(SemanticDiagnostic {
                message: if let Some(suggestion) = &diag.suggestion {
                    format!("{} (suggestion: {})", diag.message, suggestion)
                } else {
                    diag.message.clone()
                },
                line: 0,
                severity: match diag.severity {
                    TypeDiagnosticSeverity::Error => DiagnosticSeverity::Error,
                    TypeDiagnosticSeverity::Warning => DiagnosticSeverity::Warning,
                },
            });
        }

        self.type_env = checker.env;

        SemanticResult { symbol_table: self.symbol_table.clone(), diagnostics: self.diagnostics.clone() }
    }

    /// 获取指定类型的字段和方法列表，用于类型感知补全。
    pub fn get_type_members(&self, type_name: &str) -> Vec<CompletionMember> {
        let mut members = Vec::new();

        if let Some(fields) = self.type_env.class_fields.get(type_name) {
            for (field_name, field_ty) in fields {
                members.push(CompletionMember {
                    name: field_name.clone(),
                    kind: CompletionMemberKind::Field,
                    type_info: type_info_to_string(field_ty),
                });
            }
        }

        if let Some(sigs) = self.type_env.function_signatures.get(type_name) {
            let params: Vec<String> = sigs.param_types.iter().map(|t| type_info_to_string(t)).collect();
            members.push(CompletionMember {
                name: type_name.to_string(),
                kind: CompletionMemberKind::Method,
                type_info: format!("({}) -> {}", params.join(", "), type_info_to_string(&sigs.return_type)),
            });
        }

        for (sig_name, sig) in &self.type_env.function_signatures {
            if sig_name.starts_with(&format!("{}_", type_name)) {
                let method_name = sig_name.strip_prefix(&format!("{}_", type_name)).unwrap_or(sig_name);
                let params: Vec<String> = sig.param_types.iter().map(|t| type_info_to_string(t)).collect();
                members.push(CompletionMember {
                    name: method_name.to_string(),
                    kind: CompletionMemberKind::Method,
                    type_info: format!("({}) -> {}", params.join(", "), type_info_to_string(&sig.return_type)),
                });
            }
        }

        members
    }

    /// 根据名称查找符号定义。
    pub fn get_definition(&self, name: &str) -> Option<&Symbol> {
        self.symbol_table.get(name)
    }

    /// 获取上一次分析的类型环境。
    pub fn type_environment(&self) -> &TypeEnvironment {
        &self.type_env
    }
}

/// 补全成员类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionMemberKind {
    /// 字段
    Field,
    /// 方法
    Method,
}

/// 补全成员
#[derive(Debug, Clone)]
pub struct CompletionMember {
    /// 成员名称
    pub name: String,
    /// 成员类型
    pub kind: CompletionMemberKind,
    /// 类型信息
    pub type_info: String,
}

/// 将 TypeInfo 转换为可读的类型字符串。
fn type_info_to_string(ty: &TypeInfo) -> String {
    match ty {
        TypeInfo::Int => "i32".to_string(),
        TypeInfo::Float => "f64".to_string(),
        TypeInfo::Bool => "Bool".to_string(),
        TypeInfo::String => "String".to_string(),
        TypeInfo::Null => "null".to_string(),
        TypeInfo::Unknown => "Unknown".to_string(),
        TypeInfo::Object(name) => name.clone(),
        TypeInfo::Trait(name) => name.clone(),
        TypeInfo::Component(name) => name.clone(),
        TypeInfo::Function { param_types, return_type } => {
            let params: Vec<String> = param_types.iter().map(type_info_to_string).collect();
            format!("({}) -> {}", params.join(", "), type_info_to_string(return_type))
        }
        TypeInfo::Array(inner) => format!("Array<{}>", type_info_to_string(inner)),
        TypeInfo::Map(key, value) => format!("Map<{}, {}>", type_info_to_string(key), type_info_to_string(value)),
        TypeInfo::Closure { param_types, return_type, .. } => {
            let params: Vec<String> = param_types.iter().map(type_info_to_string).collect();
            format!("({}) -> {}", params.join(", "), type_info_to_string(return_type))
        }
        TypeInfo::Tuple(types) => {
            let parts: Vec<String> = types.iter().map(type_info_to_string).collect();
            format!("({})", parts.join(", "))
        }
        TypeInfo::Optional(inner) => format!("Option<{}>", type_info_to_string(inner)),
        TypeInfo::Generic(name, args) => {
            let arg_strs: Vec<String> = args.iter().map(type_info_to_string).collect();
            format!("{}<{}>", name, arg_strs.join(", "))
        }
    }
}
