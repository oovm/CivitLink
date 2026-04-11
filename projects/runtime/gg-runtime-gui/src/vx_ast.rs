//! VX 文件抽象语法树
//!
//! 定义 VX 单文件组件的 AST 结构，
//! 包含模板、脚本和样式三个部分。

use crate::TemplateNode;

/// VX 文档，表示一个完整的 .vx 文件
#[derive(Debug, Clone, PartialEq)]
pub struct VxDocument {
    /// 模板 AST
    pub template: Option<TemplateAst>,
    /// 脚本 AST
    pub script: Option<ScriptAst>,
    /// 样式 AST
    pub style: Option<StyleAst>,
}

impl VxDocument {
    /// 将 VxDocument 转换为 DynamicVxComponent
    pub fn to_component(&self) -> crate::DynamicVxComponent {
        let template = self.template.clone();

        let style_string = self.style.as_ref().map(|style_ast| {
            style_ast
                .rules
                .iter()
                .map(|rule| {
                    let props = rule
                        .properties
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("; ");
                    if props.is_empty() {
                        format!("{} {{ }}", rule.selector)
                    } else {
                        format!("{} {{ {}; }}", rule.selector, props)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        });

        let script_source = self.script.as_ref().map(|s| s.raw_source.clone());

        crate::DynamicVxComponent::from_document("vx-component", template, style_string, script_source)
    }
}

/// 模板 AST，基于 TemplateNode
pub type TemplateAst = TemplateNode;

/// 样式规则
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRule {
    /// 选择器
    pub selector: String,
    /// 属性列表
    pub properties: Vec<(String, String)>,
}

/// 样式 AST
#[derive(Debug, Clone, PartialEq)]
pub struct StyleAst {
    /// 样式规则列表
    pub rules: Vec<StyleRule>,
}

/// 脚本 AST
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptAst {
    /// 原始脚本源码
    pub raw_source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vx_document_default() {
        let doc = VxDocument {
            template: None,
            script: None,
            style: None,
        };
        assert!(doc.template.is_none());
        assert!(doc.script.is_none());
        assert!(doc.style.is_none());
    }

    #[test]
    fn test_vx_document_with_template() {
        let doc = VxDocument {
            template: Some(TemplateAst::Element {
                tag: "div".to_string(),
                attributes: vec![("class".to_string(), "container".to_string())],
                children: vec![TemplateNode::Text("Hello".to_string())],
            }),
            script: None,
            style: None,
        };
        assert!(doc.template.is_some());
        let template = doc.template.unwrap();
        match template {
            TemplateNode::Element {
                tag,
                attributes,
                children,
            } => {
                assert_eq!(tag, "div");
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].0, "class");
                assert_eq!(attributes[0].1, "container");
                assert_eq!(children.len(), 1);
            }
            TemplateNode::Text(_) => panic!("expected Element"),
        }
    }

    #[test]
    fn test_vx_document_full() {
        let doc = VxDocument {
            template: Some(TemplateAst::Element {
                tag: "div".to_string(),
                attributes: vec![],
                children: vec![TemplateNode::Text("Hello".to_string())],
            }),
            script: Some(ScriptAst {
                raw_source: "let x = 1;".to_string(),
            }),
            style: Some(StyleAst {
                rules: vec![StyleRule {
                    selector: ".container".to_string(),
                    properties: vec![("color".to_string(), "red".to_string())],
                }],
            }),
        };
        assert!(doc.template.is_some());
        assert!(doc.script.is_some());
        assert!(doc.style.is_some());
    }

    #[test]
    fn test_style_rule() {
        let rule = StyleRule {
            selector: ".box".to_string(),
            properties: vec![
                ("color".to_string(), "red".to_string()),
                ("margin".to_string(), "10px".to_string()),
            ],
        };
        assert_eq!(rule.selector, ".box");
        assert_eq!(rule.properties.len(), 2);
        assert_eq!(rule.properties[0].0, "color");
        assert_eq!(rule.properties[0].1, "red");
        assert_eq!(rule.properties[1].0, "margin");
        assert_eq!(rule.properties[1].1, "10px");
    }

    #[test]
    fn test_style_ast() {
        let ast = StyleAst {
            rules: vec![
                StyleRule {
                    selector: ".a".to_string(),
                    properties: vec![("color".to_string(), "blue".to_string())],
                },
                StyleRule {
                    selector: ".b".to_string(),
                    properties: vec![("margin".to_string(), "5px".to_string())],
                },
            ],
        };
        assert_eq!(ast.rules.len(), 2);
    }

    #[test]
    fn test_script_ast() {
        let ast = ScriptAst {
            raw_source: "function hello() { return 42; }".to_string(),
        };
        assert_eq!(ast.raw_source, "function hello() { return 42; }");
    }

    #[test]
    fn test_template_ast_is_template_node() {
        let node = TemplateAst::Text("hello".to_string());
        match node {
            TemplateNode::Text(content) => assert_eq!(content, "hello"),
            TemplateNode::Element { .. } => panic!("expected Text"),
        }
    }

    #[test]
    fn test_vx_document_equality() {
        let doc1 = VxDocument {
            template: Some(TemplateAst::Text("hello".to_string())),
            script: None,
            style: None,
        };
        let doc2 = VxDocument {
            template: Some(TemplateAst::Text("hello".to_string())),
            script: None,
            style: None,
        };
        assert_eq!(doc1, doc2);
    }

    #[test]
    fn test_style_rule_equality() {
        let rule1 = StyleRule {
            selector: ".a".to_string(),
            properties: vec![("color".to_string(), "red".to_string())],
        };
        let rule2 = StyleRule {
            selector: ".a".to_string(),
            properties: vec![("color".to_string(), "red".to_string())],
        };
        assert_eq!(rule1, rule2);
    }

    #[test]
    fn test_script_ast_equality() {
        let s1 = ScriptAst {
            raw_source: "let x = 1;".to_string(),
        };
        let s2 = ScriptAst {
            raw_source: "let x = 1;".to_string(),
        };
        assert_eq!(s1, s2);
    }
}
