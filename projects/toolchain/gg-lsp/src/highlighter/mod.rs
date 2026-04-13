//! Valkyrie 脚本语法高亮模块。

use std::{borrow::Cow, ops::Range};

use oak_highlight::{
    HighlightResult, HighlightSegment, HighlightSpan, HighlightTheme, highlighter::Highlighter, themes::Theme,
};

use crate::kind::SyntaxKind;

const VALKYRIE_KEYWORDS: &[&str] = &[
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
];

/// Valkyrie 脚本语法高亮器。
///
/// 实现逐字符扫描的词法分析器，识别关键字、字符串、数字、
/// 注释、标识符、标点符号和空白字符等 token 类型，
/// 并将结果映射为 `HighlightResult` 用于语法高亮渲染。
pub struct GgHighlighter {
    /// 高亮主题配置。
    pub theme: HighlightTheme,
}

impl Default for GgHighlighter {
    fn default() -> Self {
        Self { theme: HighlightTheme::default() }
    }
}

impl GgHighlighter {
    /// 创建一个新的 Valkyrie 高亮器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用指定主题创建高亮器。
    pub fn with_theme(mut self, theme: HighlightTheme) -> Self {
        self.theme = theme;
        self
    }

    /// 使用预定义主题枚举创建高亮器。
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme.get_theme();
        self
    }

    /// 对 Valkyrie 源码执行词法分析，返回 token 列表。
    ///
    /// 每个 token 由其语法类型和字节范围组成。
    pub fn highlight_valkyrie(source: &str) -> Vec<(SyntaxKind, Range<usize>)> {
        let chars: Vec<char> = source.chars().collect();
        let mut pos: usize = 0;
        let mut tokens = Vec::new();
        let len = chars.len();

        while pos < len {
            let start = pos;
            let ch = chars[pos];

            if ch.is_whitespace() {
                pos += 1;
                while pos < len && chars[pos].is_whitespace() {
                    pos += 1;
                }
                tokens.push((SyntaxKind::Whitespace, byte_range(source, start, pos, &chars)));
            }
            else if ch == '/' && pos + 1 < len {
                if chars[pos + 1] == '/' {
                    pos += 2;
                    while pos < len && chars[pos] != '\n' {
                        pos += 1;
                    }
                    tokens.push((SyntaxKind::Comment, byte_range(source, start, pos, &chars)));
                }
                else if chars[pos + 1] == '*' {
                    pos += 2;
                    while pos + 1 < len && !(chars[pos] == '*' && chars[pos + 1] == '/') {
                        pos += 1;
                    }
                    if pos + 1 < len {
                        pos += 2;
                    }
                    else {
                        pos = len;
                    }
                    tokens.push((SyntaxKind::Comment, byte_range(source, start, pos, &chars)));
                }
                else {
                    pos += 1;
                    tokens.push((SyntaxKind::Punct, byte_range(source, start, pos, &chars)));
                }
            }
            else if ch == '"' {
                pos += 1;
                while pos < len {
                    if chars[pos] == '\\' && pos + 1 < len {
                        pos += 2;
                    }
                    else if chars[pos] == '"' {
                        pos += 1;
                        break;
                    }
                    else {
                        pos += 1;
                    }
                }
                tokens.push((SyntaxKind::String, byte_range(source, start, pos, &chars)));
            }
            else if ch.is_ascii_digit() {
                if ch == '0' && pos + 1 < len && (chars[pos + 1] == 'x' || chars[pos + 1] == 'X') {
                    pos += 2;
                    while pos < len && (chars[pos].is_ascii_hexdigit() || chars[pos] == '_') {
                        pos += 1;
                    }
                }
                else {
                    while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '_') {
                        pos += 1;
                    }
                    if pos < len && chars[pos] == '.' && pos + 1 < len && chars[pos + 1].is_ascii_digit() {
                        pos += 1;
                        while pos < len && (chars[pos].is_ascii_digit() || chars[pos] == '_') {
                            pos += 1;
                        }
                    }
                }
                tokens.push((SyntaxKind::Number, byte_range(source, start, pos, &chars)));
            }
            else if ch.is_ascii_alphabetic() || ch == '_' {
                pos += 1;
                while pos < len && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                    pos += 1;
                }
                let word: String = chars[start..pos].iter().collect();
                if VALKYRIE_KEYWORDS.contains(&word.as_str()) {
                    tokens.push((SyntaxKind::Keyword, byte_range(source, start, pos, &chars)));
                }
                else {
                    tokens.push((SyntaxKind::Ident, byte_range(source, start, pos, &chars)));
                }
            }
            else {
                let range = scan_punct(&chars, pos);
                pos = range.end;
                let byte_start = chars[..start].iter().map(|c| c.len_utf8()).sum();
                let byte_end = chars[..range.end].iter().map(|c| c.len_utf8()).sum();
                tokens.push((SyntaxKind::Punct, byte_start..byte_end));
            }
        }

        tokens
    }

    /// 将 `SyntaxKind` 映射为 TextMate 风格的 scope 字符串。
    pub fn kind_to_scope(kind: SyntaxKind) -> &'static str {
        match kind {
            SyntaxKind::Keyword => "keyword",
            SyntaxKind::String => "string",
            SyntaxKind::Number => "constant",
            SyntaxKind::Comment => "comment",
            SyntaxKind::Ident => "variable.other",
            SyntaxKind::Punct => "punctuation",
            SyntaxKind::Whitespace => "punctuation.whitespace",
            SyntaxKind::Unknown => "none",
        }
    }

    /// 将 token 列表转换为高亮结果。
    pub fn tokens_to_result<'a>(
        source: &'a str,
        tokens: &[(SyntaxKind, Range<usize>)],
        theme: &HighlightTheme,
    ) -> HighlightResult<'a> {
        let mut segments = Vec::with_capacity(tokens.len());

        for (kind, range) in tokens {
            let scope = Self::kind_to_scope(*kind);
            let style = theme.resolve_style(scope);
            let text = &source[range.clone()];
            segments.push(HighlightSegment {
                span: HighlightSpan { start: range.start, end: range.end },
                style,
                text: Cow::Borrowed(text),
            });
        }

        HighlightResult { segments, source: Cow::Borrowed(source) }
    }
}

impl Highlighter for GgHighlighter {
    fn highlight<'a>(
        &self,
        source: &'a str,
        _language: &str,
        theme: Theme,
    ) -> Result<HighlightResult<'a>, oak_core::errors::OakError> {
        let theme_config = theme.get_theme();
        let tokens = Self::highlight_valkyrie(source);
        Ok(Self::tokens_to_result(source, &tokens, &theme_config))
    }
}

/// 将字符位置范围转换为字节偏移范围。
fn byte_range(_source: &str, char_start: usize, char_end: usize, chars: &[char]) -> Range<usize> {
    let byte_start = if char_start == 0 { 0 } else { chars[..char_start].iter().map(|c| c.len_utf8()).sum() };
    let byte_end = if char_end == 0 { 0 } else { chars[..char_end].iter().map(|c| c.len_utf8()).sum() };
    byte_start..byte_end
}

/// 从当前位置扫描标点符号，返回其字符位置范围。
fn scan_punct(chars: &[char], pos: usize) -> Range<usize> {
    let len = chars.len();
    let start = pos;
    let ch = chars[pos];

    let two_char = if pos + 1 < len {
        let s: String = chars[pos..pos + 2].iter().collect();
        Some(s)
    }
    else {
        None
    };

    let three_char = if pos + 2 < len {
        let s: String = chars[pos..pos + 3].iter().collect();
        Some(s)
    }
    else {
        None
    };

    if let Some(ref tc) = three_char {
        if matches!(tc.as_str(), "<<=" | ">>=") {
            return start..start + 3;
        }
    }

    if let Some(ref tc) = two_char {
        if matches!(
            tc.as_str(),
            "==" | "!=" | "<=" | ">=" | "&&" | "||" | "<<" | ">>" | "+=" | "-=" | "*=" | "/=" | "::" | "->" | "=>"
        ) {
            return start..start + 2;
        }
    }

    if matches!(
        ch,
        '+' | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '<'
            | '>'
            | '!'
            | '&'
            | '|'
            | '('
            | ')'
            | '{'
            | '}'
            | '['
            | ']'
            | ','
            | '.'
            | ':'
            | ';'
            | '@'
            | '#'
    ) {
        return start..start + 1;
    }

    start..start + 1
}
