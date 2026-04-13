//! Galgame MDX 解析器模块
//! 将 .galgame 文件（基于 MDX 格式）解析为 GalgameIr

use oak_core::{Builder, ParseSession, source::SourceText};
use oak_markdown::{
    MarkdownBuilder, MarkdownLanguage,
    ast::{Block, MarkdownRoot},
};

use crate::{
    error::{GalgameError, GalgameResult},
    ir::{CharacterDefIr, ChoiceIr, CommandIr, DialogueIr, FrontMatterIr, GalgameIr, VariableValueIr},
};

/// Galgame MDX 解析器
pub struct GalgameParser {
    /// 自动 ID 计数器
    auto_id_counter: usize,
}

impl GalgameParser {
    /// 创建新的 Galgame 解析器
    pub fn new() -> Self {
        Self { auto_id_counter: 0 }
    }

    /// 解析 .galgame 源码为 GalgameIr
    pub fn parse(&mut self, source: &str) -> GalgameResult<GalgameIr> {
        let (front_matter, remaining_source) = Self::parse_front_matter(source);

        let config = MarkdownLanguage { allow_mdx: true, allow_xml: true, ..MarkdownLanguage::default() };

        let builder = MarkdownBuilder::new(&config);
        let source_text = SourceText::new(remaining_source.to_string());
        let mut cache = ParseSession::default();
        let result = builder.build(&source_text, &[], &mut cache);

        let root = result.result.map_err(|e| GalgameError::ParseError(format!("Failed to parse galgame script: {}", e)))?;

        let mut ir = self.parse_from_ast(&root)?;
        ir.front_matter = front_matter;
        Ok(ir)
    }

    /// 解析 YAML front matter
    fn parse_front_matter(source: &str) -> (FrontMatterIr, &str) {
        if !source.starts_with("---") {
            return (FrontMatterIr::default(), source);
        }

        let after_first = &source[3..];
        let newline_pos = match after_first.find('\n') {
            Some(pos) => pos + 3,
            None => return (FrontMatterIr::default(), source),
        };

        let rest = &source[newline_pos..];
        let end_pos = match rest.find("\n---") {
            Some(pos) => pos,
            None => return (FrontMatterIr::default(), source),
        };

        let yaml_content = &rest[..end_pos];
        let remaining = &rest[end_pos + 4..];

        let mut front_matter = FrontMatterIr::default();

        for line in yaml_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                match key {
                    "title" => front_matter.title = Some(value.to_string()),
                    "author" => front_matter.author = Some(value.to_string()),
                    "version" => front_matter.version = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        (front_matter, remaining)
    }

    /// 从预解析的 oak-markdown AST 构建 GalgameIr
    pub fn parse_from_ast(&mut self, ast: &MarkdownRoot) -> GalgameResult<GalgameIr> {
        let front_matter = FrontMatterIr::default();
        let mut imports = Vec::new();
        let mut dialogues = Vec::new();

        for block in &ast.blocks {
            match block {
                Block::MdxImport(import_stmt) => {
                    imports.push(import_stmt.content.clone());
                }
                Block::MdxExport(_) => {}
                Block::Heading(h) => {
                    let id = self.next_auto_id();
                    dialogues.push(DialogueIr {
                        id,
                        speaker_id: None,
                        text: h.content.clone(),
                        commands: Vec::new(),
                        choices: Vec::new(),
                        next_node_id: None,
                    });
                }
                Block::Paragraph(p) => {
                    if p.content.trim().is_empty() {
                        continue;
                    }
                    let id = self.next_auto_id();
                    dialogues.push(DialogueIr {
                        id,
                        speaker_id: None,
                        text: p.content.clone(),
                        commands: Vec::new(),
                        choices: Vec::new(),
                        next_node_id: None,
                    });
                }
                Block::MdxComponent(component) => {
                    self.parse_mdx_component_into(component, &mut dialogues);
                }
                _ => {}
            }
        }

        Self::link_dialogues(&mut dialogues);

        Ok(GalgameIr { front_matter, imports, dialogues })
    }

    /// 解析 MDX 组件并追加到对话列表
    fn parse_mdx_component_into(&mut self, component: &oak_markdown::ast::MdxComponent, dialogues: &mut Vec<DialogueIr>) {
        match component.name.as_str() {
            "If" | "if" => {
                let condition = component
                    .attributes
                    .iter()
                    .find_map(|attr| if attr.name == "condition" { attr.value.clone() } else { None });

                let condition_expr = condition.unwrap_or_default();

                let mut true_branch = Vec::new();
                let mut false_branch = Vec::new();
                let mut in_else = false;

                for child in &component.children {
                    if let Block::MdxComponent(child_component) = child {
                        if child_component.name == "Else" || child_component.name == "else" {
                            in_else = true;
                            continue;
                        }
                        self.parse_mdx_component_into(
                            child_component,
                            if in_else { &mut false_branch } else { &mut true_branch },
                        );
                    }
                }

                let id = self.next_auto_id();
                let command = CommandIr::CallFunction { name: "check_condition".to_string(), args: vec![condition_expr] };
                dialogues.push(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: vec![command],
                    choices: Vec::new(),
                    next_node_id: None,
                });

                dialogues.extend(true_branch);
                dialogues.extend(false_branch);
            }
            "Loop" | "loop" => {
                let times = component.attributes.iter().find_map(|attr| {
                    if attr.name == "times" || attr.name == "count" {
                        attr.value.as_ref().and_then(|v| v.parse::<u32>().ok())
                    }
                    else {
                        None
                    }
                });

                let loop_times = times.unwrap_or(1);

                let mut body = Vec::new();
                for child in &component.children {
                    if let Block::MdxComponent(child_component) = child {
                        self.parse_mdx_component_into(child_component, &mut body);
                    }
                }

                for _ in 0..loop_times {
                    let body_clone = body.clone();
                    dialogues.extend(body_clone);
                }
            }
            _ => {
                if let Some(dialogue) = self.parse_mdx_component(component) {
                    dialogues.push(dialogue);
                }
            }
        }
    }

    /// 解析 MDX 组件为 DialogueIr
    fn parse_mdx_component(&mut self, component: &oak_markdown::ast::MdxComponent) -> Option<DialogueIr> {
        let id = self.next_auto_id();

        match component.name.as_str() {
            "Dialogue" | "dialogue" => {
                let mut speaker_id = None;
                let mut text = String::new();

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "character" | "speaker" => {
                            if let Some(ref value) = attr.value {
                                speaker_id = Some(value.clone());
                            }
                        }
                        "text" => {
                            if let Some(ref value) = attr.value {
                                text = value.clone();
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr { id, speaker_id, text, commands: Vec::new(), choices: Vec::new(), next_node_id: None })
            }
            "Scene" | "scene" => {
                let mut name = String::new();
                let mut bg = String::new();

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "name" => {
                            if let Some(ref value) = attr.value {
                                name = value.clone();
                            }
                        }
                        "bg" => {
                            if let Some(ref value) = attr.value {
                                bg = value.clone();
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: vec![CommandIr::ChangeScene { name, bg }],
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            "Character" | "character" => {
                let mut char_name = String::new();
                let mut sprite = String::new();
                let mut position = String::from("center");

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "name" => {
                            if let Some(ref value) = attr.value {
                                char_name = value.clone();
                            }
                        }
                        "sprite" | "expression" => {
                            if let Some(ref value) = attr.value {
                                sprite = value.clone();
                            }
                        }
                        "position" => {
                            if let Some(ref value) = attr.value {
                                position = value.clone();
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: vec![CommandIr::ShowCharacter { name: char_name, sprite, position }],
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            "Choice" | "choice" => {
                let mut choices = Vec::new();

                for child in &component.children {
                    if let Block::MdxComponent(child_component) = child {
                        if child_component.name == "Option" || child_component.name == "option" {
                            let mut choice_text = String::new();
                            let mut next_node_id = String::new();
                            let mut condition = None;

                            for attr in &child_component.attributes {
                                match attr.name.as_str() {
                                    "text" => {
                                        if let Some(ref value) = attr.value {
                                            choice_text = value.clone();
                                        }
                                    }
                                    "next" => {
                                        if let Some(ref value) = attr.value {
                                            next_node_id = value.clone();
                                        }
                                    }
                                    "condition" => {
                                        if let Some(ref value) = attr.value {
                                            condition = Some(value.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            choices.push(ChoiceIr { text: choice_text, next_node_id, condition });
                        }
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: Vec::new(),
                    choices,
                    next_node_id: None,
                })
            }
            "Audio" | "audio" => {
                let mut commands = Vec::new();

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "bgm" => {
                            if let Some(ref value) = attr.value {
                                commands.push(CommandIr::PlayBgm { path: value.clone() });
                            }
                        }
                        "se" => {
                            if let Some(ref value) = attr.value {
                                commands.push(CommandIr::PlaySe { path: value.clone() });
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands,
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            "Effect" | "effect" => {
                let mut commands = Vec::new();

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "fadeIn" | "fade_in" => {
                            if let Some(ref value) = attr.value {
                                if let Ok(duration) = value.parse::<f32>() {
                                    commands.push(CommandIr::FadeIn { duration });
                                }
                            }
                        }
                        "fadeOut" | "fade_out" => {
                            if let Some(ref value) = attr.value {
                                if let Ok(duration) = value.parse::<f32>() {
                                    commands.push(CommandIr::FadeOut { duration });
                                }
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands,
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            "SetVariable" | "setvariable" | "set_variable" => {
                let mut var_name = String::new();
                let mut var_value = String::new();

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "name" => {
                            if let Some(ref value) = attr.value {
                                var_name = value.clone();
                            }
                        }
                        "value" => {
                            if let Some(ref value) = attr.value {
                                var_value = value.clone();
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: vec![CommandIr::SetVariable { name: var_name, value: var_value }],
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            "Goto" | "goto" => {
                let mut target = String::new();

                for attr in &component.attributes {
                    if attr.name == "target" || attr.name == "label" {
                        if let Some(ref value) = attr.value {
                            target = value.clone();
                        }
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: vec![CommandIr::Goto { target }],
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            "Call" | "call" => {
                let mut func_name = String::new();
                let mut args = Vec::new();

                for attr in &component.attributes {
                    match attr.name.as_str() {
                        "name" | "function" => {
                            if let Some(ref value) = attr.value {
                                func_name = value.clone();
                            }
                        }
                        "args" => {
                            if let Some(ref value) = attr.value {
                                args = value.split(',').map(|s| s.trim().to_string()).collect();
                            }
                        }
                        _ => {}
                    }
                }

                Some(DialogueIr {
                    id,
                    speaker_id: None,
                    text: String::new(),
                    commands: vec![CommandIr::CallFunction { name: func_name, args }],
                    choices: Vec::new(),
                    next_node_id: None,
                })
            }
            _ => None,
        }
    }

    /// 生成下一个自动 ID
    fn next_auto_id(&mut self) -> String {
        let id = format!("_auto_{}", self.auto_id_counter);
        self.auto_id_counter += 1;
        id
    }

    /// 将对话节点按顺序链接
    fn link_dialogues(dialogues: &mut [DialogueIr]) {
        if dialogues.is_empty() {
            return;
        }

        for i in 0..dialogues.len() {
            if dialogues[i].choices.is_empty() && dialogues[i].next_node_id.is_none() {
                if i + 1 < dialogues.len() {
                    dialogues[i].next_node_id = Some(dialogues[i + 1].id.clone());
                }
            }
        }
    }
}

impl Default for GalgameParser {
    fn default() -> Self {
        Self::new()
    }
}
