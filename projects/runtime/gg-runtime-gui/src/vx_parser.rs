//! VX 文件语法解析器
//!
//! 将 VxLexer 产生的词法单元流解析为 VxDocument AST，
//! 支持 template/script/style 三段式结构的递归下降解析。

use std::fmt;

use crate::vx_ast::{ScriptAst, StyleAst, StyleRule, VxDocument};
use crate::vx_lexer::{Span, SpannedToken, VxLexer, VxSection, VxToken};
use crate::TemplateNode;

/// 语法解析错误
#[derive(Debug, Clone)]
pub struct VxParseError {
    /// 错误消息
    pub message: String,
    /// 期望的词法单元描述
    pub expected: Option<String>,
    /// 实际遇到的词法单元描述
    pub found: Option<String>,
    /// 源码位置
    pub span: Option<Span>,
}

impl fmt::Display for VxParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error: {}", self.message)?;
        if let Some(expected) = &self.expected {
            write!(f, ", expected {}", expected)?;
        }
        if let Some(found) = &self.found {
            write!(f, ", found {}", found)?;
        }
        if let Some(span) = &self.span {
            write!(f, " at {}:{}", span.start_line, span.start_col)?;
        }
        Ok(())
    }
}

impl std::error::Error for VxParseError {}

/// VX 文件语法解析器
///
/// 消费 VxLexer 产生的词法单元流，构建 VxDocument AST。
/// 支持三种段（template/script/style）以任意顺序出现。
pub struct VxParser {
    lexer: VxLexer,
    current: Option<SpannedToken>,
}

impl VxParser {
    /// 创建新的语法解析器
    pub fn new(lexer: VxLexer) -> Self {
        Self {
            lexer,
            current: None,
        }
    }

    /// 解析 VX 文件，返回 VxDocument AST
    pub fn parse(&mut self) -> Result<VxDocument, VxParseError> {
        let mut template = None;
        let mut script = None;
        let mut style = None;

        self.advance()?;

        while !self.is_at_end() {
            match &self.current {
                Some(spanned) => match &spanned.token {
                    VxToken::SectionOpen(VxSection::Template) => {
                        template = Some(self.parse_template_section()?);
                    }
                    VxToken::SectionOpen(VxSection::Script) => {
                        script = Some(self.parse_script_section()?);
                    }
                    VxToken::SectionOpen(VxSection::Style) => {
                        style = Some(self.parse_style_section()?);
                    }
                    VxToken::EndOfInput => break,
                    _ => {
                        return Err(VxParseError {
                            message: "unexpected token outside of section".to_string(),
                            expected: Some("section tag".to_string()),
                            found: Some(format!("{:?}", spanned.token)),
                            span: Some(spanned.span),
                        });
                    }
                },
                None => break,
            }
        }

        Ok(VxDocument {
            template,
            script,
            style,
        })
    }

    fn advance(&mut self) -> Result<(), VxParseError> {
        match self.lexer.next_token() {
            Ok(Some(spanned)) => {
                self.current = Some(spanned);
                Ok(())
            }
            Ok(None) => {
                self.current = None;
                Ok(())
            }
            Err(e) => Err(VxParseError {
                message: e.message,
                expected: None,
                found: None,
                span: Some(Span {
                    start_line: e.line,
                    start_col: e.col,
                    end_line: e.line,
                    end_col: e.col,
                }),
            }),
        }
    }

    fn is_at_end(&self) -> bool {
        match &self.current {
            Some(spanned) => spanned.token == VxToken::EndOfInput,
            None => true,
        }
    }

    fn expect_section_close(&mut self, section: VxSection) -> Result<(), VxParseError> {
        match &self.current {
            Some(spanned) => match &spanned.token {
                VxToken::SectionClose(s) if *s == section => {
                    self.advance()?;
                    Ok(())
                }
                _ => Err(VxParseError {
                    message: format!("expected </{}> closing tag", match section {
                        VxSection::Template => "template",
                        VxSection::Script => "script",
                        VxSection::Style => "style",
                    }),
                    expected: Some(format!("SectionClose({:?})", section)),
                    found: Some(format!("{:?}", spanned.token)),
                    span: Some(spanned.span),
                }),
            },
            None => Err(VxParseError {
                message: format!("unexpected end of input, expected </{}>", match section {
                    VxSection::Template => "template",
                    VxSection::Script => "script",
                    VxSection::Style => "style",
                }),
                expected: Some(format!("SectionClose({:?})", section)),
                found: Some("end of input".to_string()),
                span: None,
            }),
        }
    }

    fn parse_template_section(&mut self) -> Result<TemplateNode, VxParseError> {
        self.advance()?;
        let nodes = self.parse_template_nodes(VxSection::Template)?;
        self.expect_section_close(VxSection::Template)?;

        if nodes.len() == 1 {
            Ok(nodes.into_iter().next().unwrap())
        } else if nodes.is_empty() {
            Ok(TemplateNode::Text(String::new()))
        } else {
            Ok(TemplateNode::Element {
                tag: "fragment".to_string(),
                attributes: vec![],
                children: nodes,
            })
        }
    }

    fn parse_template_nodes(&mut self, _parent_section: VxSection) -> Result<Vec<TemplateNode>, VxParseError> {
        let mut children = Vec::new();

        loop {
            match &self.current {
                Some(spanned) => match &spanned.token {
                    VxToken::SectionClose(_) | VxToken::TagClose(_) => break,
                    VxToken::EndOfInput => {
                        return Err(VxParseError {
                            message: "unexpected end of input in template section".to_string(),
                            expected: Some("template content or </template>".to_string()),
                            found: Some("end of input".to_string()),
                            span: Some(spanned.span),
                        });
                    }
                    VxToken::TagOpen(tag) => {
                        let tag_name = tag.clone();
                        let span = spanned.span;
                        self.advance()?;
                        let mut attributes = Vec::new();
                        while let Some(s) = &self.current {
                            if let VxToken::Attribute { name, value } = &s.token {
                                attributes.push((name.clone(), value.clone()));
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                        let sub_children = self.parse_template_nodes(_parent_section)?;
                        match &self.current {
                            Some(s) => match &s.token {
                                VxToken::TagClose(close_tag) => {
                                    if close_tag != &tag_name {
                                        return Err(VxParseError {
                                            message: format!(
                                                "mismatched closing tag: expected </{}>, found </{}>",
                                                tag_name, close_tag
                                            ),
                                            expected: Some(format!("</{}>", tag_name)),
                                            found: Some(format!("</{}>", close_tag)),
                                            span: Some(s.span),
                                        });
                                    }
                                    self.advance()?;
                                }
                                _ => {
                                    return Err(VxParseError {
                                        message: format!("expected closing tag </{}>", tag_name),
                                        expected: Some(format!("</{}>", tag_name)),
                                        found: Some(format!("{:?}", s.token)),
                                        span: Some(s.span),
                                    });
                                }
                            },
                            None => {
                                return Err(VxParseError {
                                    message: format!("unexpected end of input, expected </{}>", tag_name),
                                    expected: Some(format!("</{}>", tag_name)),
                                    found: Some("end of input".to_string()),
                                    span: Some(span),
                                });
                            }
                        }
                        children.push(TemplateNode::Element {
                            tag: tag_name,
                            attributes,
                            children: sub_children,
                        });
                    }
                    VxToken::SelfCloseTag(tag) => {
                        let tag_name = tag.clone();
                        self.advance()?;
                        let mut attributes = Vec::new();
                        while let Some(s) = &self.current {
                            if let VxToken::Attribute { name, value } = &s.token {
                                attributes.push((name.clone(), value.clone()));
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                        children.push(TemplateNode::Element {
                            tag: tag_name,
                            attributes,
                            children: vec![],
                        });
                    }
                    VxToken::Text(text) => {
                        let content = text.clone();
                        self.advance()?;
                        children.push(TemplateNode::Text(content));
                    }
                    _ => {
                        return Err(VxParseError {
                            message: "unexpected token in template section".to_string(),
                            expected: Some("tag or text".to_string()),
                            found: Some(format!("{:?}", spanned.token)),
                            span: Some(spanned.span),
                        });
                    }
                },
                None => break,
            }
        }

        Ok(children)
    }

    fn parse_script_section(&mut self) -> Result<ScriptAst, VxParseError> {
        self.advance()?;
        let raw_source = match &self.current {
            Some(spanned) => match &spanned.token {
                VxToken::Text(content) => {
                    let source = content.clone();
                    self.advance()?;
                    source
                }
                VxToken::SectionClose(VxSection::Script) => String::new(),
                _ => {
                    return Err(VxParseError {
                        message: "unexpected token in script section".to_string(),
                        expected: Some("script content or </script>".to_string()),
                        found: Some(format!("{:?}", spanned.token)),
                        span: Some(spanned.span),
                    });
                }
            },
            None => {
                return Err(VxParseError {
                    message: "unexpected end of input in script section".to_string(),
                    expected: Some("script content or </script>".to_string()),
                    found: Some("end of input".to_string()),
                    span: None,
                });
            }
        };

        self.expect_section_close(VxSection::Script)?;
        Ok(ScriptAst { raw_source })
    }

    fn parse_style_section(&mut self) -> Result<StyleAst, VxParseError> {
        self.advance()?;
        let rules = self.parse_style_rules()?;
        self.expect_section_close(VxSection::Style)?;
        Ok(StyleAst { rules })
    }

    fn parse_style_rules(&mut self) -> Result<Vec<StyleRule>, VxParseError> {
        let mut rules = Vec::new();

        loop {
            match &self.current {
                Some(spanned) => match &spanned.token {
                    VxToken::SectionClose(_) | VxToken::EndOfInput => break,
                    VxToken::Selector(selector) => {
                        let selector_str = selector.clone();
                        self.advance()?;
                        let properties = self.parse_style_block()?;
                        rules.push(StyleRule {
                            selector: selector_str,
                            properties,
                        });
                    }
                    VxToken::Variable(name) => {
                        let var_name = format!("${}", name);
                        self.advance()?;
                        let mut properties = Vec::new();
                        if let Some(s) = &self.current {
                            if let VxToken::Value(value) = &s.token {
                                properties.push(("value".to_string(), value.clone()));
                                self.advance()?;
                            }
                        }
                        rules.push(StyleRule {
                            selector: var_name,
                            properties,
                        });
                    }
                    VxToken::BlockClose => {
                        self.advance()?;
                        break;
                    }
                    _ => {
                        return Err(VxParseError {
                            message: "unexpected token in style section".to_string(),
                            expected: Some("selector or variable".to_string()),
                            found: Some(format!("{:?}", spanned.token)),
                            span: Some(spanned.span),
                        });
                    }
                },
                None => break,
            }
        }

        Ok(rules)
    }

    fn parse_style_block(&mut self) -> Result<Vec<(String, String)>, VxParseError> {
        match &self.current {
            Some(spanned) => match &spanned.token {
                VxToken::BlockOpen => {
                    self.advance()?;
                }
                _ => {
                    return Err(VxParseError {
                        message: "expected '{' after selector".to_string(),
                        expected: Some("'{'".to_string()),
                        found: Some(format!("{:?}", spanned.token)),
                        span: Some(spanned.span),
                    });
                }
            },
            None => {
                return Err(VxParseError {
                    message: "unexpected end of input, expected '{'".to_string(),
                    expected: Some("'{'".to_string()),
                    found: Some("end of input".to_string()),
                    span: None,
                });
            }
        }

        let mut properties = Vec::new();
        let mut nested_rules = Vec::new();

        loop {
            match &self.current {
                Some(spanned) => match &spanned.token {
                    VxToken::BlockClose => {
                        self.advance()?;
                        break;
                    }
                    VxToken::SectionClose(_) | VxToken::EndOfInput => {
                        return Err(VxParseError {
                            message: "unexpected end of style block, expected '}'".to_string(),
                            expected: Some("'}'".to_string()),
                            found: Some(format!("{:?}", spanned.token)),
                            span: Some(spanned.span),
                        });
                    }
                    VxToken::Property(name) => {
                        let prop_name = name.clone();
                        self.advance()?;
                        let prop_value = match &self.current {
                            Some(s) => match &s.token {
                                VxToken::Value(value) => {
                                    let v = value.clone();
                                    self.advance()?;
                                    v
                                }
                                _ => {
                                    return Err(VxParseError {
                                        message: format!("expected value after property '{}'", prop_name),
                                        expected: Some("value".to_string()),
                                        found: Some(format!("{:?}", s.token)),
                                        span: Some(s.span),
                                    });
                                }
                            },
                            None => {
                                return Err(VxParseError {
                                    message: format!("unexpected end of input after property '{}'", prop_name),
                                    expected: Some("value".to_string()),
                                    found: Some("end of input".to_string()),
                                    span: None,
                                });
                            }
                        };
                        properties.push((prop_name, prop_value));
                    }
                    VxToken::Selector(selector) => {
                        let selector_str = selector.clone();
                        self.advance()?;
                        let nested_props = self.parse_style_block()?;
                        nested_rules.push(StyleRule {
                            selector: selector_str,
                            properties: nested_props,
                        });
                    }
                    VxToken::Variable(name) => {
                        let var_name = format!("${}", name);
                        self.advance()?;
                        if let Some(s) = &self.current {
                            if let VxToken::Value(value) = &s.token {
                                properties.push((var_name, value.clone()));
                                self.advance()?;
                            }
                        }
                    }
                    _ => {
                        return Err(VxParseError {
                            message: "unexpected token in style block".to_string(),
                            expected: Some("property, selector, or '}'".to_string()),
                            found: Some(format!("{:?}", spanned.token)),
                            span: Some(spanned.span),
                        });
                    }
                },
                None => break,
            }
        }

        for rule in nested_rules {
            properties.push((format!("nested:{}", rule.selector), format!("{:?}", rule.properties)));
        }

        Ok(properties)
    }
}

/// 便捷函数：解析 VX 文件源码为 VxDocument
pub fn parse_vx(source: &str) -> Result<VxDocument, VxParseError> {
    let lexer = VxLexer::new(source);
    let mut parser = VxParser::new(lexer);
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
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element { tag, children, .. } => {
                assert_eq!(tag, "div");
                assert_eq!(children.len(), 1);
                match &children[0] {
                    TemplateNode::Text(content) => assert_eq!(content, "Hello"),
                    _ => panic!("expected Text node"),
                }
            }
            _ => panic!("expected Element node"),
        }
    }

    #[test]
    fn test_parse_template_with_attributes() {
        let doc = parse_vx(
            "<template><div class=\"container\" id=\"main\">Hello</div></template>",
        )
        .unwrap();
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element { attributes, .. } => {
                assert_eq!(attributes.len(), 2);
                assert_eq!(attributes[0], ("class".to_string(), "container".to_string()));
                assert_eq!(attributes[1], ("id".to_string(), "main".to_string()));
            }
            _ => panic!("expected Element node"),
        }
    }

    #[test]
    fn test_parse_nested_template() {
        let doc = parse_vx(
            "<template><Layout><Text>Hello</Text></Layout></template>",
        )
        .unwrap();
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element { tag, children, .. } => {
                assert_eq!(tag, "Layout");
                assert_eq!(children.len(), 1);
                match &children[0] {
                    TemplateNode::Element { tag, children, .. } => {
                        assert_eq!(tag, "Text");
                        match &children[0] {
                            TemplateNode::Text(content) => assert_eq!(content, "Hello"),
                            _ => panic!("expected Text node"),
                        }
                    }
                    _ => panic!("expected Element node"),
                }
            }
            _ => panic!("expected Element node"),
        }
    }

    #[test]
    fn test_parse_self_closing_tag() {
        let doc = parse_vx("<template><br /></template>").unwrap();
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element { tag, children, .. } => {
                assert_eq!(tag, "br");
                assert!(children.is_empty());
            }
            _ => panic!("expected Element node"),
        }
    }

    #[test]
    fn test_parse_self_closing_with_attribute() {
        let doc = parse_vx("<template><input type=\"text\" /></template>").unwrap();
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element { tag, attributes, children, .. } => {
                assert_eq!(tag, "input");
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0], ("type".to_string(), "text".to_string()));
                assert!(children.is_empty());
            }
            _ => panic!("expected Element node"),
        }
    }

    #[test]
    fn test_parse_script_section() {
        let doc = parse_vx("<script>let x = 1;</script>").unwrap();
        assert!(doc.script.is_some());
        assert_eq!(doc.script.unwrap().raw_source, "let x = 1;");
    }

    #[test]
    fn test_parse_empty_script() {
        let doc = parse_vx("<script></script>").unwrap();
        assert!(doc.script.is_some());
        assert_eq!(doc.script.unwrap().raw_source, "");
    }

    #[test]
    fn test_parse_style_section() {
        let doc = parse_vx("<style>.title { color: #fff; }</style>").unwrap();
        assert!(doc.style.is_some());
        let style = doc.style.unwrap();
        assert_eq!(style.rules.len(), 1);
        assert_eq!(style.rules[0].selector, ".title");
        assert_eq!(style.rules[0].properties.len(), 1);
        assert_eq!(style.rules[0].properties[0], ("color".to_string(), "#fff".to_string()));
    }

    #[test]
    fn test_parse_style_multiple_properties() {
        let doc = parse_vx(
            "<style>.box { color: red; margin: 10px; }</style>",
        )
        .unwrap();
        let style = doc.style.unwrap();
        assert_eq!(style.rules.len(), 1);
        assert_eq!(style.rules[0].properties.len(), 2);
        assert_eq!(style.rules[0].properties[0], ("color".to_string(), "red".to_string()));
        assert_eq!(style.rules[0].properties[1], ("margin".to_string(), "10px".to_string()));
    }

    #[test]
    fn test_parse_style_multiple_rules() {
        let doc = parse_vx(
            "<style>.a { color: red; } .b { margin: 10px; }</style>",
        )
        .unwrap();
        let style = doc.style.unwrap();
        assert_eq!(style.rules.len(), 2);
        assert_eq!(style.rules[0].selector, ".a");
        assert_eq!(style.rules[1].selector, ".b");
    }

    #[test]
    fn test_parse_style_variable() {
        let doc = parse_vx("<style>$primary: blue;</style>").unwrap();
        let style = doc.style.unwrap();
        assert_eq!(style.rules.len(), 1);
        assert_eq!(style.rules[0].selector, "$primary");
        assert_eq!(style.rules[0].properties.len(), 1);
        assert_eq!(style.rules[0].properties[0], ("value".to_string(), "blue".to_string()));
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

    #[test]
    fn test_parse_sections_in_different_order() {
        let doc = parse_vx(
            "<style>.a { color: red; }</style><template><div /></template><script>let x = 1;</script>",
        )
        .unwrap();
        assert!(doc.template.is_some());
        assert!(doc.script.is_some());
        assert!(doc.style.is_some());
    }

    #[test]
    fn test_error_mismatched_tags() {
        let result = parse_vx("<template><div><span></div></span></template>");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("mismatched"));
    }

    #[test]
    fn test_error_unexpected_close_tag() {
        let result = parse_vx("<template></div></template>");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_section_close() {
        let result = parse_vx("<template><div>Hello</div>");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("end of input") || err.message.contains("expected"));
    }

    #[test]
    fn test_error_unexpected_token_outside_section() {
        let result = parse_vx("<div>Hello</div>");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_display() {
        let err = VxParseError {
            message: "test error".to_string(),
            expected: Some("something".to_string()),
            found: Some("other".to_string()),
            span: Some(Span {
                start_line: 5,
                start_col: 10,
                end_line: 5,
                end_col: 15,
            }),
        };
        let display = format!("{}", err);
        assert!(display.contains("test error"));
        assert!(display.contains("something"));
        assert!(display.contains("other"));
        assert!(display.contains("5:10"));
    }

    #[test]
    fn test_parse_empty_template() {
        let doc = parse_vx("<template></template>").unwrap();
        assert!(doc.template.is_some());
    }

    #[test]
    fn test_parse_empty_style() {
        let doc = parse_vx("<style></style>").unwrap();
        assert!(doc.style.is_some());
        let style = doc.style.unwrap();
        assert!(style.rules.is_empty());
    }

    #[test]
    fn test_parse_template_text_only() {
        let doc = parse_vx("<template>Hello World</template>").unwrap();
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Text(content) => assert_eq!(content, "Hello World"),
            _ => panic!("expected Text node"),
        }
    }

    #[test]
    fn test_parse_template_multiple_children() {
        let doc = parse_vx(
            "<template><div /><span /></template>",
        )
        .unwrap();
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element { tag, children, .. } => {
                assert_eq!(tag, "fragment");
                assert_eq!(children.len(), 2);
            }
            _ => panic!("expected Element node"),
        }
    }
}
