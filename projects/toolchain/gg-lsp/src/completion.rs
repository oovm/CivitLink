//! Valkyrie 脚本自动补全模块

/// 补全项类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionItemKind {
    /// 关键字
    Keyword,
    /// 函数
    Function,
    /// 变量
    Variable,
}

/// 补全项
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    /// 补全文本
    pub label: String,
    /// 补全类型
    pub kind: CompletionItemKind,
    /// 简要说明
    pub detail: String,
}

/// Valkyrie 关键字列表
const KEYWORDS: &[(&str, &str)] = &[
    ("namespace", "Keyword: namespace declaration"),
    ("micro", "Keyword: micro declaration"),
    ("let", "Keyword: variable declaration"),
    ("const", "Keyword: constant declaration"),
    ("fn", "Keyword: function declaration"),
    ("if", "Keyword: conditional branch"),
    ("else", "Keyword: else branch"),
    ("while", "Keyword: while loop"),
    ("for", "Keyword: for loop"),
    ("return", "Keyword: return from function"),
    ("break", "Keyword: break out of loop"),
    ("continue", "Keyword: continue to next iteration"),
    ("true", "Keyword: boolean true"),
    ("false", "Keyword: boolean false"),
    ("null", "Keyword: null value"),
    ("struct", "Keyword: struct declaration"),
    ("enum", "Keyword: enum declaration"),
    ("impl", "Keyword: implementation block"),
    ("trait", "Keyword: trait declaration"),
    ("pub", "Keyword: public visibility"),
    ("use", "Keyword: use declaration"),
    ("mod", "Keyword: module declaration"),
    ("import", "Keyword: import declaration"),
    ("export", "Keyword: export declaration"),
    ("from", "Keyword: import from module"),
    ("as", "Keyword: alias binding"),
    ("in", "Keyword: membership operator"),
    ("match", "Keyword: pattern matching"),
    ("loop", "Keyword: infinite loop"),
];

/// Valkyrie 内置函数列表
const BUILTINS: &[(&str, &str)] = &[
    ("print", "Print a value to stdout"),
    ("len", "Get the length of a collection"),
    ("push", "Add a value to a collection"),
    ("pop", "Remove and return the last element"),
    ("typeof", "Get the type of a value"),
    ("to_string", "Convert a value to string"),
    ("to_int", "Convert a value to integer"),
    ("to_float", "Convert a value to float"),
];

/// 内置函数签名映射
const BUILTIN_SIGNATURES: &[(&str, &str)] = &[
    ("print", "print(value)"),
    ("len", "len(collection)"),
    ("push", "push(collection, value)"),
    ("pop", "pop(collection)"),
    ("typeof", "typeof(value)"),
    ("to_string", "to_string(value)"),
    ("to_int", "to_int(value)"),
    ("to_float", "to_float(value)"),
];

/// Valkyrie 自动补全提供者
pub struct CompletionProvider {}

impl CompletionProvider {
    /// 创建新的补全提供者
    pub fn new() -> Self {
        Self {}
    }

    /// 根据前缀和源码上下文提供补全项
    pub fn complete(&self, prefix: &str, source: &str) -> Vec<CompletionEntry> {
        let mut entries = Vec::new();

        self.collect_keyword_completions(prefix, &mut entries);
        self.collect_builtin_completions(prefix, &mut entries);
        self.collect_variable_completions(prefix, source, &mut entries);

        entries
    }

    /// 收集关键字补全项
    fn collect_keyword_completions(&self, prefix: &str, entries: &mut Vec<CompletionEntry>) {
        for &(keyword, detail) in KEYWORDS {
            if keyword.starts_with(prefix) {
                entries.push(CompletionEntry {
                    label: keyword.to_string(),
                    kind: CompletionItemKind::Keyword,
                    detail: detail.to_string(),
                });
            }
        }
    }

    /// 收集内置函数补全项
    fn collect_builtin_completions(&self, prefix: &str, entries: &mut Vec<CompletionEntry>) {
        for &(name, description) in BUILTINS {
            if name.starts_with(prefix) {
                let signature = BUILTIN_SIGNATURES.iter().find(|&&(n, _)| n == name).map(|&(_, sig)| sig).unwrap_or(name);
                entries.push(CompletionEntry {
                    label: signature.to_string(),
                    kind: CompletionItemKind::Function,
                    detail: description.to_string(),
                });
            }
        }
    }

    /// 收集作用域变量补全项
    fn collect_variable_completions(&self, prefix: &str, source: &str, entries: &mut Vec<CompletionEntry>) {
        let variables = Self::extract_variables(source);
        for var in variables {
            if var.starts_with(prefix) {
                entries.push(CompletionEntry {
                    label: var,
                    kind: CompletionItemKind::Variable,
                    detail: "Variable: declared in current scope".to_string(),
                });
            }
        }
    }

    /// 从源码中提取 let/const 声明的变量名
    fn extract_variables(source: &str) -> Vec<String> {
        let mut vars = Vec::new();
        for line in source.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("let ") {
                if let Some(ident) = Self::extract_identifier(rest) {
                    vars.push(ident);
                }
            }
            else if let Some(rest) = trimmed.strip_prefix("const ") {
                if let Some(ident) = Self::extract_identifier(rest) {
                    vars.push(ident);
                }
            }
        }
        vars
    }

    /// 从字符串开头提取标识符
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
}

impl Default for CompletionProvider {
    /// 创建默认补全提供者
    fn default() -> Self {
        Self::new()
    }
}
