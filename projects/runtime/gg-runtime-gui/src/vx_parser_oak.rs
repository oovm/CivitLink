//! VX 文件语法解析器（基于 oak-voc）
//! 
//! 使用 oak-voc 解析器解析 VX 文件，支持 template/script/style 三段式结构。

use oak_voc::{VocLexer, VocParser, VxDocument, TemplateNode, ScriptAst, StyleAst, StyleRule};

/// VX 文件解析错误
#[derive(Debug, Clone)]
pub struct VxParseError {
    /// 错误消息
    pub message: String,
    /// 源码位置
    pub line: u32,
    pub column: u32,
}

impl std::fmt::Display for VxParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error: {} at {}:{}", self.message, self.line, self.column)
    }
}

impl std::error::Error for VxParseError {}

/// VX 文件语法解析器（基于 oak-voc）
/// 
/// 使用 oak-voc 解析器解析 VX 文件，构建 VxDocument AST。
pub struct VxParserOak {
    lexer: VocLexer,
    parser: VocParser,
}

impl VxParserOak {
    /// 创建新的语法解析器
    pub fn new(source: &str) -> Self {
        Self {
            lexer: VocLexer::new(source),
            parser: VocParser::new(),
        }
    }

    /// 解析 VX 文件，返回 VxDocument AST
    pub fn parse(&mut self) -> Result<oak_voc::VxDocument, VxParseError> {
        use oak_core::{LexerState, ParserState, TokenStream};

        let mut lexer_state = LexerState::default();
        let tokens: Vec<_> = std::iter::from_fn(|| {
            self.lexer.next_token(&mut lexer_state).ok()
        }).collect();

        let mut token_stream = TokenStream::from_tokens(tokens);
        let mut parser_state = ParserState::default();

        match self.parser.parse(&mut token_stream, &mut parser_state) {
            Ok(document) => Ok(document),
            Err(e) => Err(VxParseError {
                message: format!("{:?}", e),
                line: 1,
                column: 1,
            }),
        }
    }
}

/// 便捷函数：解析 VX 文件源码为 VxDocument
pub fn parse_vx(source: &str) -> Result<oak_voc::VxDocument, VxParseError> {
    let mut parser = VxParserOak::new(source);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_only() {
        let doc = parse_vx("<template><div>Hello</div></template>").unwrap();
        assert!(doc.template.is_some());
        assert!(doc.script.is_none());
        assert!(doc.style.is_none());
    }

    #[test]
    fn test_parse_full_vx_file() {
        let source = "\
<template>
  <Layout style=\"flex-1\">
    <Text class=\"title\">Hello GG Editor</Text>
  </Layout>
</template>

<script>
  using gg_editor::ui::widgets::{Layout, Text};
</script>

<style>
  .title {
    color: #fff;
  }
</style>";
        let doc = parse_vx(source).unwrap();
        assert!(doc.template.is_some());
        assert!(doc.script.is_some());
        assert!(doc.style.is_some());
    }
}