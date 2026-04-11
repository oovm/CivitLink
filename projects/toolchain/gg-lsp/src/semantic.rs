//! Valkyrie 脚本语义分析模块。
//!
//! 提供符号表构建、悬停信息查询和语义诊断等功能，
//! 基于简单的文本模式匹配实现，无需完整 AST。

use std::collections::HashMap;

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
        self.by_name
            .get(name)
            .and_then(|indices| indices.first())
            .and_then(|&idx| self.symbols.get(idx))
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
        Self {
            symbol_table: SymbolTable::new(),
            diagnostics: Vec::new(),
        }
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
                    let type_info = if params.is_empty() {
                        None
                    }
                    else {
                        Some(format!("({})", params.join(", ")))
                    };
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
                let rest = if let Some(r) = rest.strip_prefix("mut ") {
                    r
                }
                else {
                    rest
                };
                if let Some(name) = extract_identifier(rest) {
                    let column = line.find(&name).unwrap_or(0);
                    let type_info = extract_type_from_let(rest);
                    self.symbol_table.insert(Symbol {
                        name,
                        kind: SymbolKind::Variable,
                        line: line_idx,
                        column,
                        type_info,
                    });
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("const ") {
                if let Some(name) = extract_identifier(rest) {
                    let column = line.find(&name).unwrap_or(0);
                    let type_info = extract_type_from_let(rest);
                    self.symbol_table.insert(Symbol {
                        name,
                        kind: SymbolKind::Variable,
                        line: line_idx,
                        column,
                        type_info,
                    });
                }
            }
        }

        self.collect_undefined_references(source);

        SemanticResult {
            symbol_table: self.symbol_table.clone(),
            diagnostics: self.diagnostics.clone(),
        }
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

        Some(HoverInfo {
            contents,
            range: Some((start, end)),
        })
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
            "namespace", "micro", "let", "const", "fn", "if", "else", "while", "for",
            "return", "break", "continue", "true", "false", "null", "struct", "enum",
            "impl", "trait", "pub", "use", "mod", "import", "export", "from", "as",
            "in", "match", "loop", "mut",
        ];
        let builtins = [
            "print", "len", "push", "pop", "typeof", "to_string", "to_int", "to_float",
        ];

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
        let type_str = type_part
            .split(|c: char| c == '=' || c == ',')
            .next()
            .unwrap_or(type_part)
            .trim();
        if !type_str.is_empty() {
            return Some(type_str.to_string());
        }
    }
    None
}

/// 从指定行文本的指定列位置提取标识符。
fn extract_identifier_at(line: &str, column: usize) -> Option<String> {
    let byte_offset = line
        .char_indices()
        .take(column)
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_identifier() {
        assert_eq!(extract_identifier("hello world"), Some("hello".to_string()));
        assert_eq!(extract_identifier("  name: Type"), Some("name".to_string()));
        assert_eq!(extract_identifier("123abc"), None);
        assert_eq!(extract_identifier("_private"), Some("_private".to_string()));
    }

    #[test]
    fn test_extract_params_from_line() {
        let result = extract_params_from_line("main(x: int, y: float)");
        assert_eq!(result, vec!["x".to_string(), "y".to_string()]);

        let result = extract_params_from_line("main()");
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_type_from_let() {
        assert_eq!(extract_type_from_let("x: int = 42"), Some("int".to_string()));
        assert_eq!(extract_type_from_let("x = 42"), None);
    }

    #[test]
    fn test_symbol_table_insert_and_get() {
        let mut table = SymbolTable::new();
        table.insert(Symbol {
            name: "foo".to_string(),
            kind: SymbolKind::Function,
            line: 0,
            column: 6,
            type_info: None,
        });
        table.insert(Symbol {
            name: "bar".to_string(),
            kind: SymbolKind::Variable,
            line: 1,
            column: 4,
            type_info: None,
        });

        assert!(table.get("foo").is_some());
        assert!(table.get("bar").is_some());
        assert!(table.get("baz").is_none());
        assert_eq!(table.symbols.len(), 2);
    }

    #[test]
    fn test_analyze_namespace() {
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze("namespace Game\n");

        let sym = result.symbol_table.get("Game").unwrap();
        assert_eq!(sym.name, "Game");
        assert_eq!(sym.kind, SymbolKind::Namespace);
    }

    #[test]
    fn test_analyze_micro() {
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze("micro main(x: int)\n");

        let sym = result.symbol_table.get("main").unwrap();
        assert_eq!(sym.name, "main");
        assert_eq!(sym.kind, SymbolKind::Function);
        assert_eq!(sym.type_info, Some("(x)".to_string()));
    }

    #[test]
    fn test_analyze_let() {
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze("let score: int = 0\n");

        let sym = result.symbol_table.get("score").unwrap();
        assert_eq!(sym.name, "score");
        assert_eq!(sym.kind, SymbolKind::Variable);
        assert_eq!(sym.type_info, Some("int".to_string()));
    }

    #[test]
    fn test_analyze_let_mut() {
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze("let mut count = 0\n");

        let sym = result.symbol_table.get("count").unwrap();
        assert_eq!(sym.name, "count");
        assert_eq!(sym.kind, SymbolKind::Variable);
        assert_eq!(sym.type_info, None);
    }

    #[test]
    fn test_get_hover_info() {
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze("let score: int = 0\n");

        let hover = analyzer.get_hover_info(0, 4, "let score: int = 0\n").unwrap();
        assert!(hover.contents.contains("variable"));
        assert!(hover.contents.contains("score"));
    }

    #[test]
    fn test_diagnostics_undefined() {
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze("let x = 1\nprint(y)\n");

        let undef_diags: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.message.contains("undefined"))
            .collect();
        assert!(!undef_diags.is_empty());
    }
}
