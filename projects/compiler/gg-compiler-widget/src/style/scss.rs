//! SCSS 子集处理器模块

use std::collections::HashMap;

use crate::{
    artifact::RuleId,
    error::WidgetResult,
    style::ir::{ResolvedValue, SelectorIr, SelectorType, StyleIr, StyleRuleIr, StyleValue},
};
use oak_voc::ast::VxDocument;

/// SCSS 处理器
pub struct ScssProcessor {
    /// 变量表
    variables: HashMap<String, StyleValue>,
}

impl ScssProcessor {
    /// 创建新的 SCSS 处理器
    pub fn new() -> Self {
        Self { variables: HashMap::new() }
    }

    /// 处理 SCSS 内容并生成 StyleIr
    pub fn process(&mut self, style_content: &str) -> WidgetResult<StyleIr> {
        self.variables.clear();

        let after_vars = self.resolve_variables(style_content);

        let after_mixins = self.expand_mixins(&after_vars);

        let flattened = self.flatten_nested_rules(&after_mixins);

        let rules = self.parse_rules_to_ir(&flattened);

        let mut specificity_map = HashMap::new();
        for rule in &rules {
            specificity_map.insert(rule.selector.text.clone(), rule.specificity);
        }

        Ok(StyleIr { rules, specificity_map, resolved_vars: self.variables.clone() })
    }

    /// 解析变量定义并替换引用
    fn resolve_variables(&mut self, content: &str) -> String {
        let mut var_defs: HashMap<String, String> = HashMap::new();
        let mut result_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with('$') && trimmed.contains(':') {
                let colon_pos = trimmed.find(':').unwrap();
                let name = trimmed[1..colon_pos].trim().to_string();
                let rest = &trimmed[colon_pos + 1..];

                let value = if let Some(semi_pos) = rest.find(';') {
                    rest[..semi_pos].trim().to_string()
                }
                else {
                    rest.trim().to_string()
                };

                var_defs.insert(name, value);
                continue;
            }

            let mut resolved_line = line.to_string();
            for (var_name, var_value) in &var_defs {
                let pattern = format!("${}", var_name);
                resolved_line = resolved_line.replace(&pattern, var_value);
            }

            result_lines.push(resolved_line);
        }

        for (name, value) in &var_defs {
            self.variables.insert(name.clone(), Self::parse_style_value(value));
        }

        result_lines.join("\n")
    }

    /// 展开 @mixin 和 @include
    fn expand_mixins(&self, content: &str) -> String {
        let mut mixins: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let content = self.extract_mixins(content, &mut mixins);
        self.replace_includes(&content, &mixins)
    }

    /// 提取 @mixin 定义并从内容中移除
    fn extract_mixins<'a>(&self, content: &'a str, mixins: &mut HashMap<String, Vec<(String, String)>>) -> String {
        let mut result = String::new();
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            if content[pos..].starts_with("@mixin") {
                pos += 6;
                while pos < content.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }

                let name_start = pos;
                while pos < content.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'-' || bytes[pos] == b'_') {
                    pos += 1;
                }
                let mixin_name = content[name_start..pos].to_string();

                while pos < content.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }

                if pos < content.len() && bytes[pos] == b'{' {
                    pos += 1;
                }

                let mut depth = 1;
                let body_start = pos;
                while pos < content.len() && depth > 0 {
                    if bytes[pos] == b'{' {
                        depth += 1;
                    }
                    else if bytes[pos] == b'}' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        pos += 1;
                    }
                }
                let body = content[body_start..pos].trim();
                if pos < content.len() {
                    pos += 1;
                }

                let properties = Self::parse_mixin_body(body);
                mixins.insert(mixin_name, properties);
            }
            else {
                result.push(bytes[pos] as char);
                pos += 1;
            }
        }

        result
    }

    /// 解析 mixin 体内的属性
    fn parse_mixin_body(body: &str) -> Vec<(String, String)> {
        let mut props = Vec::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(colon_pos) = trimmed.find(':') {
                let name = trimmed[..colon_pos].trim().to_string();
                let rest = &trimmed[colon_pos + 1..];
                let value = if let Some(semi_pos) = rest.find(';') {
                    rest[..semi_pos].trim().to_string()
                }
                else {
                    rest.trim().to_string()
                };
                if !name.is_empty() && !value.is_empty() {
                    props.push((name, value));
                }
            }
        }
        props
    }

    /// 替换 @include 引用
    fn replace_includes(&self, content: &str, mixins: &HashMap<String, Vec<(String, String)>>) -> String {
        let mut result = String::new();
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            if content[pos..].starts_with("@include") {
                pos += 8;
                while pos < content.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }

                let name_start = pos;
                while pos < content.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'-' || bytes[pos] == b'_') {
                    pos += 1;
                }
                let include_name = content[name_start..pos].to_string();

                while pos < content.len() && bytes[pos].is_ascii_whitespace() {
                    pos += 1;
                }
                if pos < content.len() && bytes[pos] == b';' {
                    pos += 1;
                }

                if let Some(properties) = mixins.get(&include_name) {
                    for (name, value) in properties {
                        result.push_str(&format!("  {}: {};", name, value));
                        result.push('\n');
                    }
                }
            }
            else {
                result.push(bytes[pos] as char);
                pos += 1;
            }
        }

        result
    }

    /// 展开嵌套规则
    fn flatten_nested_rules(&self, content: &str) -> String {
        let (rules, _) = self.parse_nested_block(content, "");
        let mut result = String::new();
        for (selector, properties) in &rules {
            if properties.is_empty() {
                continue;
            }
            result.push_str(selector);
            result.push_str(" {\n");
            for (name, value) in properties {
                result.push_str(&format!("  {}: {};\n", name, value));
            }
            result.push_str("}\n");
        }
        result
    }

    /// 递归解析嵌套块
    fn parse_nested_block(&self, content: &str, parent_selector: &str) -> (Vec<(String, Vec<(String, String)>)>, usize) {
        let mut rules: Vec<(String, Vec<(String, String)>)> = Vec::new();
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            self.skip_ws_and_newlines(content, &mut pos);

            if pos >= content.len() {
                break;
            }

            if bytes[pos] == b'}' {
                pos += 1;
                return (rules, pos);
            }

            let selector = self.read_selector(content, &mut pos);
            if selector.is_empty() {
                pos += 1;
                continue;
            }

            self.skip_ws_and_newlines(content, &mut pos);

            if pos >= content.len() || bytes[pos] != b'{' {
                pos += 1;
                continue;
            }
            pos += 1;

            let full_selector = if parent_selector.is_empty() {
                selector.clone()
            }
            else if selector.starts_with('&') {
                let replaced = selector.replacen('&', parent_selector, 1);
                replaced
            }
            else {
                format!("{} {}", parent_selector, selector)
            };

            let (nested_rules, consumed) = self.parse_nested_block(content, &full_selector);
            pos += consumed;

            let mut own_properties: Vec<(String, String)> = Vec::new();
            for (sel, props) in &nested_rules {
                if sel == &full_selector {
                    own_properties = props.clone();
                }
                else {
                    rules.push((sel.clone(), props.clone()));
                }
            }

            if !own_properties.is_empty() {
                rules.insert(0, (full_selector.clone(), own_properties));
            }
        }

        (rules, pos)
    }

    /// 读取选择器
    fn read_selector(&self, content: &str, pos: &mut usize) -> String {
        let bytes = content.as_bytes();
        let start = *pos;
        while *pos < content.len() {
            let ch = bytes[*pos];
            if ch == b'{' || ch == b'}' {
                break;
            }
            *pos += 1;
        }
        content[start..*pos].trim().to_string()
    }

    /// 跳过空白和换行
    fn skip_ws_and_newlines(&self, content: &str, pos: &mut usize) {
        let bytes = content.as_bytes();
        while *pos < content.len() && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
    }

    /// 解析样式值
    fn parse_style_value(value: &str) -> StyleValue {
        if value.starts_with('#') {
            StyleValue::Color(value.to_string())
        }
        else if value.ends_with('%') {
            if let Ok(p) = value.trim_end_matches('%').parse::<f32>() {
                return StyleValue::Percentage(p);
            }
            StyleValue::String(value.to_string())
        }
        else if value.ends_with("px") || value.ends_with("em") || value.ends_with("rem") {
            if let Ok(l) = value.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-').parse::<f32>() {
                return StyleValue::Length(l);
            }
            StyleValue::String(value.to_string())
        }
        else if let Ok(n) = value.parse::<f32>() {
            StyleValue::Number(n)
        }
        else {
            StyleValue::String(value.to_string())
        }
    }

    /// 从扁平化 CSS 解析规则为 StyleRuleIr
    fn parse_rules_to_ir(&self, content: &str) -> Vec<StyleRuleIr> {
        let mut rules = Vec::new();
        let mut rule_id: RuleId = 0;
        let mut pos = 0;
        let bytes = content.as_bytes();

        while pos < content.len() {
            self.skip_ws_and_newlines(content, &mut pos);
            if pos >= content.len() {
                break;
            }

            let selector_text = self.read_selector(content, &mut pos);
            if selector_text.is_empty() {
                pos += 1;
                continue;
            }

            self.skip_ws_and_newlines(content, &mut pos);
            if pos >= content.len() || bytes[pos] != b'{' {
                continue;
            }
            pos += 1;

            let mut declarations: HashMap<String, ResolvedValue> = HashMap::new();
            loop {
                self.skip_ws_and_newlines(content, &mut pos);
                if pos >= content.len() || bytes[pos] == b'}' {
                    if pos < content.len() {
                        pos += 1;
                    }
                    break;
                }

                let prop_line = self.read_property_line(content, &mut pos);
                if let Some((name, value)) = Self::parse_declaration(&prop_line) {
                    declarations.insert(name, ResolvedValue { data: value.as_bytes().to_vec() });
                }
            }

            if !declarations.is_empty() {
                let selector_type = Self::classify_selector(&selector_text);
                let specificity = Self::calc_specificity(&selector_text);

                rules.push(StyleRuleIr {
                    id: rule_id,
                    selector: SelectorIr { selector_type, text: selector_text },
                    declarations,
                    specificity,
                });
                rule_id += 1;
            }
        }

        rules
    }

    /// 读取属性行
    fn read_property_line(&self, content: &str, pos: &mut usize) -> String {
        let bytes = content.as_bytes();
        let start = *pos;
        while *pos < content.len() && bytes[*pos] != b';' && bytes[*pos] != b'}' {
            *pos += 1;
        }
        if *pos < content.len() && bytes[*pos] == b';' {
            *pos += 1;
        }
        content[start..*pos].trim().to_string()
    }

    /// 解析单条声明
    fn parse_declaration(line: &str) -> Option<(String, String)> {
        let colon_pos = line.find(':')?;
        let name = line[..colon_pos].trim().to_string();
        let value = line[colon_pos + 1..].trim().trim_end_matches(';').trim().to_string();
        if name.is_empty() || value.is_empty() {
            return None;
        }
        Some((name, value))
    }

    /// 分类选择器类型
    fn classify_selector(selector: &str) -> SelectorType {
        if selector.starts_with('#') {
            SelectorType::Id
        }
        else if selector.starts_with('.') {
            SelectorType::Class
        }
        else if selector.contains(' ') || selector.contains(':') || selector.contains('>') {
            SelectorType::Compound
        }
        else {
            SelectorType::Tag
        }
    }

    /// 计算选择器优先级
    fn calc_specificity(selector: &str) -> u32 {
        let mut specificity = 0u32;
        for ch in selector.chars() {
            match ch {
                '#' => specificity += 100,
                '.' => specificity += 10,
                ':' => specificity += 10,
                c if c.is_ascii_alphabetic() => specificity += 1,
                _ => {}
            }
        }
        specificity
    }

    /// 从 oak-voc AST 直接处理样式并生成 StyleIr
    pub fn process_from_ast(&mut self, ast: &VxDocument) -> WidgetResult<StyleIr> {
        self.variables.clear();

        let style_content = match &ast.style {
            Some(style_ast) => {
                let mut content = String::new();
                for rule in &style_ast.rules {
                    content.push_str(&rule.selector);
                    content.push_str(" {\n");
                    for (name, value) in &rule.properties {
                        content.push_str(&format!("  {}: {};\n", name, value));
                    }
                    content.push_str("}\n");
                }
                content
            }
            None => {
                return Ok(StyleIr { rules: Vec::new(), specificity_map: HashMap::new(), resolved_vars: HashMap::new() });
            }
        };

        self.process(&style_content)
    }
}

impl Default for ScssProcessor {
    fn default() -> Self {
        Self::new()
    }
}
