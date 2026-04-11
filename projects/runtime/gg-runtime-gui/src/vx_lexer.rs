//! VX 文件词法分析器
//!
//! 将 VX 单文件组件源码解析为词法单元（Token）流，
//! 支持 template/script/style 三段式结构的识别。

use std::collections::VecDeque;
use std::fmt;

/// VX 文件中的段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VxSection {
    /// 模板段
    Template,
    /// 脚本段
    Script,
    /// 样式段
    Style,
}

/// VX 词法单元
#[derive(Debug, Clone, PartialEq)]
pub enum VxToken {
    /// 段开始标签，如 `<template>`
    SectionOpen(VxSection),
    /// 段结束标签，如 `</template>`
    SectionClose(VxSection),
    /// 开标签，如 `<div>`
    TagOpen(String),
    /// 闭标签，如 `</div>`
    TagClose(String),
    /// 自闭合标签，如 `<br />`
    SelfCloseTag(String),
    /// 属性，如 `class="container"`
    Attribute {
        /// 属性名
        name: String,
        /// 属性值
        value: String,
    },
    /// 文本内容
    Text(String),
    /// CSS 选择器
    Selector(String),
    /// CSS 块开始 `{`
    BlockOpen,
    /// CSS 块结束 `}`
    BlockClose,
    /// CSS 属性名
    Property(String),
    /// CSS 属性值
    Value(String),
    /// CSS 变量名，如 `$primary`
    Variable(String),
    /// 输入结束
    EndOfInput,
}

/// 源码位置跨度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// 起始行号（从 1 开始）
    pub start_line: usize,
    /// 起始列号（从 1 开始）
    pub start_col: usize,
    /// 结束行号
    pub end_line: usize,
    /// 结束列号
    pub end_col: usize,
}

/// 带位置信息的词法单元
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    /// 词法单元
    pub token: VxToken,
    /// 位置跨度
    pub span: Span,
}

/// 词法分析错误
#[derive(Debug, Clone)]
pub struct VxLexerError {
    /// 错误消息
    pub message: String,
    /// 错误所在行号
    pub line: usize,
    /// 错误所在列号
    pub col: usize,
}

impl fmt::Display for VxLexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "lexer error at {}:{}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for VxLexerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexerState {
    Initial,
    Template,
    Script,
    Style,
}

/// VX 词法分析器
///
/// 将 VX 单文件组件源码逐个解析为带位置信息的词法单元。
/// 支持三种段：template（XML 标签）、script（原始文本）、style（CSS/SCSS）。
pub struct VxLexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    state: LexerState,
    pending: VecDeque<SpannedToken>,
    finished: bool,
}

impl VxLexer {
    /// 创建新的词法分析器
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            state: LexerState::Initial,
            pending: VecDeque::new(),
            finished: false,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.source.len() {
            return None;
        }
        let ch = self.source[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn make_span(&self, start_line: usize, start_col: usize) -> Span {
        Span {
            start_line,
            start_col,
            end_line: self.line,
            end_col: self.col,
        }
    }

    fn error_at(&self, message: &str, line: usize, col: usize) -> VxLexerError {
        VxLexerError {
            message: message.to_string(),
            line,
            col,
        }
    }

    fn peek_string(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() > self.source.len() {
            return false;
        }
        for (i, &c) in chars.iter().enumerate() {
            if self.source[self.pos + i] != c {
                return false;
            }
        }
        true
    }

    fn consume_string(&mut self, s: &str) {
        for _ in s.chars() {
            self.advance();
        }
    }

    fn read_until_close_tag(&mut self, tag: &str) -> String {
        let close_tag = format!("</{}>", tag);
        let mut content = String::new();
        while !self.is_eof() && !self.peek_string(&close_tag) {
            content.push(self.advance().unwrap());
        }
        content
    }

    fn read_until_char(&mut self, stop: char) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == stop {
                break;
            }
            result.push(ch);
            self.advance();
        }
        result
    }

    fn make_end_of_input(&self) -> SpannedToken {
        SpannedToken {
            token: VxToken::EndOfInput,
            span: Span {
                start_line: self.line,
                start_col: self.col,
                end_line: self.line,
                end_col: self.col,
            },
        }
    }

    fn lex_initial(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        self.skip_whitespace();
        if self.is_eof() {
            return Ok(Some(self.make_end_of_input()));
        }

        if self.peek() != Some('<') {
            return Err(self.error_at(
                "expected section tag (<template>, <script>, or <style>)",
                self.line,
                self.col,
            ));
        }

        if self.peek_string("</") {
            return Err(self.error_at(
                "unexpected closing tag outside of section",
                self.line,
                self.col,
            ));
        }

        if self.peek_string("<template>") {
            let start_line = self.line;
            let start_col = self.col;
            self.consume_string("<template>");
            self.state = LexerState::Template;
            let span = self.make_span(start_line, start_col);
            return Ok(Some(SpannedToken {
                token: VxToken::SectionOpen(VxSection::Template),
                span,
            }));
        }

        if self.peek_string("<script>") {
            let start_line = self.line;
            let start_col = self.col;
            self.consume_string("<script>");
            self.state = LexerState::Script;
            let span = self.make_span(start_line, start_col);
            return Ok(Some(SpannedToken {
                token: VxToken::SectionOpen(VxSection::Script),
                span,
            }));
        }

        if self.peek_string("<style>") {
            let start_line = self.line;
            let start_col = self.col;
            self.consume_string("<style>");
            self.state = LexerState::Style;
            let span = self.make_span(start_line, start_col);
            return Ok(Some(SpannedToken {
                token: VxToken::SectionOpen(VxSection::Style),
                span,
            }));
        }

        Err(self.error_at(
            "expected section tag (<template>, <script>, or <style>)",
            self.line,
            self.col,
        ))
    }

    fn lex_template(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        loop {
            self.skip_whitespace();

            if self.is_eof() {
                return Ok(Some(self.make_end_of_input()));
            }

            if self.peek_string("</template>") {
                let start_line = self.line;
                let start_col = self.col;
                self.consume_string("</template>");
                self.state = LexerState::Initial;
                let span = self.make_span(start_line, start_col);
                return Ok(Some(SpannedToken {
                    token: VxToken::SectionClose(VxSection::Template),
                    span,
                }));
            }

            if self.peek() == Some('<') {
                return self.lex_template_tag();
            }

            let result = self.lex_template_text()?;
            if result.is_some() {
                return Ok(result);
            }
        }
    }

    fn lex_template_tag(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        let start_line = self.line;
        let start_col = self.col;

        self.advance();

        if self.peek() == Some('/') {
            self.advance();
            let tag_name = self.read_identifier();
            if tag_name.is_empty() {
                return Err(self.error_at(
                    "expected tag name after '</'",
                    self.line,
                    self.col,
                ));
            }
            self.skip_whitespace();
            if self.peek() != Some('>') {
                return Err(self.error_at(
                    "expected '>' after close tag name",
                    self.line,
                    self.col,
                ));
            }
            self.advance();
            let span = self.make_span(start_line, start_col);
            Ok(Some(SpannedToken {
                token: VxToken::TagClose(tag_name),
                span,
            }))
        } else {
            let tag_name = self.read_identifier();
            if tag_name.is_empty() {
                return Err(self.error_at(
                    "expected tag name after '<'",
                    self.line,
                    self.col,
                ));
            }

            let mut attributes = Vec::new();
            let mut is_self_closing = false;

            loop {
                self.skip_whitespace();
                if self.peek() == Some('>') {
                    self.advance();
                    break;
                }
                if self.peek_string("/>") {
                    self.advance();
                    self.advance();
                    is_self_closing = true;
                    break;
                }
                if self.is_eof() {
                    return Err(self.error_at(
                        "unexpected end of input in tag",
                        self.line,
                        self.col,
                    ));
                }

                let attr = self.lex_attribute()?;
                attributes.push(attr);
            }

            let span = self.make_span(start_line, start_col);
            let main_token = if is_self_closing {
                SpannedToken {
                    token: VxToken::SelfCloseTag(tag_name),
                    span,
                }
            } else {
                SpannedToken {
                    token: VxToken::TagOpen(tag_name),
                    span,
                }
            };

            for (name, value, attr_span) in attributes {
                self.pending.push_back(SpannedToken {
                    token: VxToken::Attribute { name, value },
                    span: attr_span,
                });
            }

            Ok(Some(main_token))
        }
    }

    fn lex_attribute(&mut self) -> Result<(String, String, Span), VxLexerError> {
        let start_line = self.line;
        let start_col = self.col;

        let name = self.read_identifier();
        if name.is_empty() {
            return Err(self.error_at("expected attribute name", self.line, self.col));
        }

        self.skip_whitespace();

        if self.peek() != Some('=') {
            return Err(self.error_at(
                "expected '=' after attribute name",
                self.line,
                self.col,
            ));
        }
        self.advance();

        self.skip_whitespace();

        if self.peek() != Some('"') {
            return Err(self.error_at(
                "expected '\"' for attribute value",
                self.line,
                self.col,
            ));
        }
        self.advance();

        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                break;
            }
            value.push(ch);
            self.advance();
        }

        let span = self.make_span(start_line, start_col);
        Ok((name, value, span))
    }

    fn lex_template_text(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        let start_line = self.line;
        let start_col = self.col;
        let mut text = String::new();
        let mut last_was_space = false;

        while let Some(ch) = self.peek() {
            if ch == '<' {
                break;
            }
            if ch.is_whitespace() {
                if !last_was_space {
                    text.push(' ');
                    last_was_space = true;
                }
                self.advance();
            } else {
                text.push(ch);
                last_was_space = false;
                self.advance();
            }
        }

        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let span = self.make_span(start_line, start_col);
        Ok(Some(SpannedToken {
            token: VxToken::Text(trimmed),
            span,
        }))
    }

    fn lex_script(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        let start_line = self.line;
        let start_col = self.col;

        let content = self.read_until_close_tag("script");

        if self.peek_string("</script>") {
            let text_token = if content.trim().is_empty() {
                None
            } else {
                Some(SpannedToken {
                    token: VxToken::Text(content),
                    span: self.make_span(start_line, start_col),
                })
            };

            let close_start_line = self.line;
            let close_start_col = self.col;
            self.consume_string("</script>");
            self.state = LexerState::Initial;
            let close_token = SpannedToken {
                token: VxToken::SectionClose(VxSection::Script),
                span: self.make_span(close_start_line, close_start_col),
            };

            if let Some(tt) = text_token {
                self.pending.push_back(close_token);
                Ok(Some(tt))
            } else {
                Ok(Some(close_token))
            }
        } else {
            Err(self.error_at(
                "unexpected end of input in script section",
                self.line,
                self.col,
            ))
        }
    }

    fn lex_style(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        self.skip_whitespace();

        if self.is_eof() {
            return Ok(Some(self.make_end_of_input()));
        }

        if self.peek_string("</style>") {
            let start_line = self.line;
            let start_col = self.col;
            self.consume_string("</style>");
            self.state = LexerState::Initial;
            let span = self.make_span(start_line, start_col);
            return Ok(Some(SpannedToken {
                token: VxToken::SectionClose(VxSection::Style),
                span,
            }));
        }

        if self.peek() == Some('}') {
            let start_line = self.line;
            let start_col = self.col;
            self.advance();
            let span = self.make_span(start_line, start_col);
            return Ok(Some(SpannedToken {
                token: VxToken::BlockClose,
                span,
            }));
        }

        if self.peek() == Some('{') {
            let start_line = self.line;
            let start_col = self.col;
            self.advance();
            let span = self.make_span(start_line, start_col);
            return Ok(Some(SpannedToken {
                token: VxToken::BlockOpen,
                span,
            }));
        }

        if self.peek() == Some('$') {
            return self.lex_style_variable();
        }

        self.lex_style_selector_or_property()
    }

    fn lex_style_variable(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        let start_line = self.line;
        let start_col = self.col;

        self.advance();
        let name = self.read_identifier();
        if name.is_empty() {
            return Err(self.error_at(
                "expected variable name after '$'",
                self.line,
                self.col,
            ));
        }

        self.skip_whitespace();

        if self.peek() == Some(':') {
            self.advance();
            self.skip_whitespace();

            let value_start_line = self.line;
            let value_start_col = self.col;
            let value = self.read_until_char(';');
            if self.peek() == Some(';') {
                self.advance();
            }
            let value_span = self.make_span(value_start_line, value_start_col);

            self.pending.push_back(SpannedToken {
                token: VxToken::Value(value.trim().to_string()),
                span: value_span,
            });
        }

        let span = self.make_span(start_line, start_col);
        Ok(Some(SpannedToken {
            token: VxToken::Variable(name),
            span,
        }))
    }

    fn lex_style_selector_or_property(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        let start_line = self.line;
        let start_col = self.col;

        let mut content = String::new();
        while !self.is_eof() {
            let ch = self.peek().unwrap();
            if ch == '{' || ch == ':' || ch == '}' || ch == ';' {
                break;
            }
            content.push(ch);
            self.advance();
        }

        if self.is_eof() {
            return Err(self.error_at(
                "unexpected end of input in style section",
                start_line,
                start_col,
            ));
        }

        let ch = self.peek().unwrap();

        if ch == '{' {
            let span = self.make_span(start_line, start_col);
            Ok(Some(SpannedToken {
                token: VxToken::Selector(content.trim().to_string()),
                span,
            }))
        } else if ch == ':' {
            self.advance();
            self.skip_whitespace();

            let value_start_line = self.line;
            let value_start_col = self.col;
            let value = self.read_until_char(';');
            if self.peek() == Some(';') {
                self.advance();
            }
            let value_span = self.make_span(value_start_line, value_start_col);

            self.pending.push_back(SpannedToken {
                token: VxToken::Value(value.trim().to_string()),
                span: value_span,
            });

            let span = self.make_span(start_line, start_col);
            Ok(Some(SpannedToken {
                token: VxToken::Property(content.trim().to_string()),
                span,
            }))
        } else {
            Err(self.error_at(
                "unexpected character in style section",
                start_line,
                start_col,
            ))
        }
    }

    /// 获取下一个词法单元
    pub fn next_token(&mut self) -> Result<Option<SpannedToken>, VxLexerError> {
        if self.finished {
            return Ok(None);
        }

        if let Some(token) = self.pending.pop_front() {
            return Ok(Some(token));
        }

        let result = match self.state {
            LexerState::Initial => self.lex_initial()?,
            LexerState::Template => self.lex_template()?,
            LexerState::Script => self.lex_script()?,
            LexerState::Style => self.lex_style()?,
        };

        if let Some(spanned) = &result {
            if spanned.token == VxToken::EndOfInput {
                self.finished = true;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_tokens(source: &str) -> Result<Vec<VxToken>, VxLexerError> {
        let mut lexer = VxLexer::new(source);
        let mut tokens = Vec::new();
        loop {
            match lexer.next_token()? {
                Some(spanned) => {
                    if spanned.token == VxToken::EndOfInput {
                        break;
                    }
                    tokens.push(spanned.token);
                }
                None => break,
            }
        }
        Ok(tokens)
    }

    #[test]
    fn test_section_open_template() {
        let tokens = collect_tokens("<template></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_section_open_script() {
        let tokens = collect_tokens("<script></script>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Script),
                VxToken::SectionClose(VxSection::Script),
            ]
        );
    }

    #[test]
    fn test_section_open_style() {
        let tokens = collect_tokens("<style></style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_template_tag_open() {
        let tokens = collect_tokens("<template><div></div></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::TagOpen("div".to_string()),
                VxToken::TagClose("div".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_self_closing_tag() {
        let tokens = collect_tokens("<template><br /></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::SelfCloseTag("br".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_attribute() {
        let tokens = collect_tokens("<template><div class=\"container\"></div></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::TagOpen("div".to_string()),
                VxToken::Attribute {
                    name: "class".to_string(),
                    value: "container".to_string(),
                },
                VxToken::TagClose("div".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_multiple_attributes() {
        let tokens = collect_tokens(
            "<template><div class=\"a\" id=\"b\"></div></template>",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::TagOpen("div".to_string()),
                VxToken::Attribute {
                    name: "class".to_string(),
                    value: "a".to_string(),
                },
                VxToken::Attribute {
                    name: "id".to_string(),
                    value: "b".to_string(),
                },
                VxToken::TagClose("div".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_self_closing_with_attribute() {
        let tokens = collect_tokens("<template><input type=\"text\" /></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::SelfCloseTag("input".to_string()),
                VxToken::Attribute {
                    name: "type".to_string(),
                    value: "text".to_string(),
                },
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_text() {
        let tokens = collect_tokens("<template>Hello World</template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::Text("Hello World".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_text_whitespace_collapse() {
        let tokens = collect_tokens("<template>  Hello   World  </template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::Text("Hello World".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_template_nested_tags_with_text() {
        let tokens = collect_tokens(
            "<template><div><span>Hello</span></div></template>",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::TagOpen("div".to_string()),
                VxToken::TagOpen("span".to_string()),
                VxToken::Text("Hello".to_string()),
                VxToken::TagClose("span".to_string()),
                VxToken::TagClose("div".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_script_raw_text() {
        let tokens = collect_tokens("<script>let x = 1;</script>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Script),
                VxToken::Text("let x = 1;".to_string()),
                VxToken::SectionClose(VxSection::Script),
            ]
        );
    }

    #[test]
    fn test_script_multiline() {
        let tokens = collect_tokens("<script>\n  let x = 1;\n  let y = 2;\n</script>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Script),
                VxToken::Text("\n  let x = 1;\n  let y = 2;\n".to_string()),
                VxToken::SectionClose(VxSection::Script),
            ]
        );
    }

    #[test]
    fn test_script_empty() {
        let tokens = collect_tokens("<script></script>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Script),
                VxToken::SectionClose(VxSection::Script),
            ]
        );
    }

    #[test]
    fn test_style_selector_and_property() {
        let tokens = collect_tokens("<style>.container { color: red; }</style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector(".container".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("color".to_string()),
                VxToken::Value("red".to_string()),
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_style_multiple_properties() {
        let tokens = collect_tokens(
            "<style>.box { color: red; margin: 10px; }</style>",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector(".box".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("color".to_string()),
                VxToken::Value("red".to_string()),
                VxToken::Property("margin".to_string()),
                VxToken::Value("10px".to_string()),
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_style_id_selector() {
        let tokens = collect_tokens("<style>#main { width: 100%; }</style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector("#main".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("width".to_string()),
                VxToken::Value("100%".to_string()),
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_style_tag_selector() {
        let tokens = collect_tokens("<style>div { padding: 5px; }</style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector("div".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("padding".to_string()),
                VxToken::Value("5px".to_string()),
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_style_variable() {
        let tokens = collect_tokens("<style>$primary: blue;</style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Variable("primary".to_string()),
                VxToken::Value("blue".to_string()),
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_style_nested_blocks() {
        let tokens = collect_tokens(
            "<style>.outer { .inner { color: red; } }</style>",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector(".outer".to_string()),
                VxToken::BlockOpen,
                VxToken::Selector(".inner".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("color".to_string()),
                VxToken::Value("red".to_string()),
                VxToken::BlockClose,
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_full_vx_file() {
        let source = "\
<template>
  <div class=\"container\">
    <span text=\"hello\" />
    Hello World
  </div>
</template>

<script>
  let x = 1;
</script>

<style>
  .container {
    color: red;
  }
</style>";
        let tokens = collect_tokens(source).unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::TagOpen("div".to_string()),
                VxToken::Attribute {
                    name: "class".to_string(),
                    value: "container".to_string(),
                },
                VxToken::SelfCloseTag("span".to_string()),
                VxToken::Attribute {
                    name: "text".to_string(),
                    value: "hello".to_string(),
                },
                VxToken::Text("Hello World".to_string()),
                VxToken::TagClose("div".to_string()),
                VxToken::SectionClose(VxSection::Template),
                VxToken::SectionOpen(VxSection::Script),
                VxToken::Text("\n  let x = 1;\n".to_string()),
                VxToken::SectionClose(VxSection::Script),
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector(".container".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("color".to_string()),
                VxToken::Value("red".to_string()),
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_error_unexpected_text_outside_section() {
        let result = collect_tokens("hello");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("section tag"));
    }

    #[test]
    fn test_error_unexpected_close_tag_outside_section() {
        let result = collect_tokens("</template>");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("closing tag"));
    }

    #[test]
    fn test_error_missing_tag_name() {
        let result = collect_tokens("<template><></template>");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_attribute_equals() {
        let result = collect_tokens("<template><div class\"x\"></div></template>");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("'='"));
    }

    #[test]
    fn test_error_missing_attribute_quote() {
        let result = collect_tokens("<template><div class=x></div></template>");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("'\"'"));
    }

    #[test]
    fn test_error_unclosed_script() {
        let result = collect_tokens("<script>let x = 1;");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("script"));
    }

    #[test]
    fn test_span_tracking() {
        let source = "<template><div></div></template>";
        let mut lexer = VxLexer::new(source);
        let token = lexer.next_token().unwrap().unwrap();
        assert_eq!(token.span.start_line, 1);
        assert_eq!(token.span.start_col, 1);
        assert_eq!(token.token, VxToken::SectionOpen(VxSection::Template));
    }

    #[test]
    fn test_span_multiline() {
        let source = "<template>\n  <div>\n  </div>\n</template>";
        let mut lexer = VxLexer::new(source);
        let mut tokens = Vec::new();
        loop {
            match lexer.next_token().unwrap() {
                Some(spanned) => {
                    if spanned.token == VxToken::EndOfInput {
                        break;
                    }
                    tokens.push(spanned);
                }
                None => break,
            }
        }
        let div_open = tokens.iter().find(|t| t.token == VxToken::TagOpen("div".to_string())).unwrap();
        assert_eq!(div_open.span.start_line, 2);
    }

    #[test]
    fn test_end_of_input() {
        let mut lexer = VxLexer::new("");
        let token = lexer.next_token().unwrap().unwrap();
        assert_eq!(token.token, VxToken::EndOfInput);
        let next = lexer.next_token().unwrap();
        assert!(next.is_none());
    }

    #[test]
    fn test_lexer_error_display() {
        let err = VxLexerError {
            message: "test error".to_string(),
            line: 5,
            col: 10,
        };
        let display = format!("{}", err);
        assert!(display.contains("5:10"));
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_style_multiple_rules() {
        let tokens = collect_tokens(
            "<style>.a { color: red; } .b { margin: 10px; }</style>",
        )
        .unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Selector(".a".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("color".to_string()),
                VxToken::Value("red".to_string()),
                VxToken::BlockClose,
                VxToken::Selector(".b".to_string()),
                VxToken::BlockOpen,
                VxToken::Property("margin".to_string()),
                VxToken::Value("10px".to_string()),
                VxToken::BlockClose,
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_style_variable_without_value() {
        let tokens = collect_tokens("<style>$var</style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::Variable("var".to_string()),
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_template_empty_text_between_tags() {
        let tokens = collect_tokens("<template><div>  </div></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::TagOpen("div".to_string()),
                VxToken::TagClose("div".to_string()),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_sections_in_different_order() {
        let tokens = collect_tokens("<style></style><template></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::SectionClose(VxSection::Style),
                VxToken::SectionOpen(VxSection::Template),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }

    #[test]
    fn test_style_empty() {
        let tokens = collect_tokens("<style></style>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Style),
                VxToken::SectionClose(VxSection::Style),
            ]
        );
    }

    #[test]
    fn test_template_empty() {
        let tokens = collect_tokens("<template></template>").unwrap();
        assert_eq!(
            tokens,
            vec![
                VxToken::SectionOpen(VxSection::Template),
                VxToken::SectionClose(VxSection::Template),
            ]
        );
    }
}
