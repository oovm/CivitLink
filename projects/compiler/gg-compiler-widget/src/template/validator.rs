//! Template 验证器模块

use std::collections::HashMap;

use crate::{
    error::{WidgetError, WidgetResult},
    registry::ComponentRegistry,
    template::ir::{BindingDirection, DataBinding, ElementId, ElementIr, EventBinding, PropertyValue, TemplateIr},
};
use oak_voc::ast::VxDocument;

/// 验证错误
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// 错误消息
    pub message: String,
}

/// Template 验证器
pub struct TemplateValidator<'a> {
    /// 组件注册表
    registry: &'a ComponentRegistry,
    /// 下一个元素 ID
    next_id: ElementId,
    /// 收集的验证错误
    errors: Vec<ValidationError>,
    /// 收集的数据绑定
    bindings: Vec<DataBinding>,
    /// 收集的事件绑定
    events: Vec<EventBinding>,
}

impl<'a> TemplateValidator<'a> {
    /// 创建新的 Template 验证器
    pub fn new(registry: &'a ComponentRegistry) -> Self {
        Self { registry, next_id: 1, errors: Vec::new(), bindings: Vec::new(), events: Vec::new() }
    }

    /// 分配下一个元素 ID
    fn alloc_id(&mut self) -> ElementId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 验证 Template 内容并生成 TemplateIr
    pub fn validate(&mut self, template_content: &str) -> WidgetResult<TemplateIr> {
        self.errors.clear();
        self.bindings.clear();
        self.events.clear();
        self.next_id = 1;

        let root = self.parse_template(template_content)?;

        if !self.errors.is_empty() {
            let messages: Vec<String> = self.errors.iter().map(|e| e.message.clone()).collect();
            return Err(WidgetError::SemanticError(format!("Template validation failed: {}", messages.join("; "))));
        }

        let mut dependencies = HashMap::new();
        self.collect_dependencies(&root, &mut dependencies);

        Ok(TemplateIr {
            root,
            dependencies,
            bindings: std::mem::take(&mut self.bindings),
            events: std::mem::take(&mut self.events),
        })
    }

    /// 解析整个模板内容，返回根元素
    fn parse_template(&mut self, content: &str) -> WidgetResult<ElementIr> {
        let trimmed = content.trim();

        let inner = if trimmed.starts_with("<template>") && trimmed.ends_with("</template>") {
            &trimmed["<template>".len()..trimmed.len() - "</template>".len()]
        }
        else {
            trimmed
        };

        let inner = inner.trim();

        if inner.is_empty() {
            let id = self.alloc_id();
            return Ok(ElementIr {
                id,
                component_type: "Layout".to_string(),
                properties: HashMap::new(),
                children: Vec::new(),
            });
        }

        let mut pos = 0;
        self.skip_whitespace(inner, &mut pos);

        if pos >= inner.len() {
            let id = self.alloc_id();
            return Ok(ElementIr {
                id,
                component_type: "Layout".to_string(),
                properties: HashMap::new(),
                children: Vec::new(),
            });
        }

        let root = self.parse_element_recursive(inner, &mut pos)?;

        self.validate_element(&root);

        Ok(root)
    }

    /// 递归解析元素
    fn parse_element_recursive(&mut self, content: &str, pos: &mut usize) -> WidgetResult<ElementIr> {
        self.skip_whitespace(content, pos);

        if *pos >= content.len() {
            return Err(WidgetError::ParseError("Unexpected end of template".to_string()));
        }

        if !content[*pos..].starts_with('<') {
            return Err(WidgetError::ParseError(format!("Expected '<' at position {}", pos)));
        }

        *pos += 1;

        self.skip_whitespace(content, pos);

        let tag_name = self.parse_identifier(content, pos);
        if tag_name.is_empty() {
            return Err(WidgetError::ParseError("Empty tag name".to_string()));
        }

        let element_id = self.alloc_id();

        let mut properties: HashMap<String, PropertyValue> = HashMap::new();
        let mut children: Vec<ElementIr> = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_whitespace(content, pos);

            if *pos >= content.len() {
                return Err(WidgetError::ParseError(format!("Unexpected end while parsing tag '{}'", tag_name)));
            }

            if content[*pos..].starts_with("/>") {
                *pos += 2;
                self_closing = true;
                break;
            }

            if content[*pos..].starts_with('>') {
                *pos += 1;
                break;
            }

            if content[*pos..].starts_with('@') {
                *pos += 1;
                let event_name = self.parse_identifier(content, pos);
                self.skip_whitespace(content, pos);
                if *pos < content.len() && content[*pos..].starts_with('=') {
                    *pos += 1;
                }
                self.skip_whitespace(content, pos);
                let handler = self.parse_attribute_value(content, pos);

                if event_name.is_empty() {
                    self.errors.push(ValidationError { message: format!("Empty event name on element '{}'", tag_name) });
                }
                else if handler.is_empty() {
                    self.errors.push(ValidationError {
                        message: format!("Empty handler for event '@{}' on element '{}'", event_name, tag_name),
                    });
                }
                else {
                    self.events.push(EventBinding {
                        target: element_id,
                        event_type: event_name.clone(),
                        handler: handler.clone(),
                    });
                }
                continue;
            }

            let is_binding = content[*pos..].starts_with(':');
            if is_binding {
                *pos += 1;
            }

            let attr_name = self.parse_identifier(content, pos);
            if attr_name.is_empty() {
                *pos += 1;
                continue;
            }

            self.skip_whitespace(content, pos);

            if *pos < content.len() && content[*pos..].starts_with('=') {
                *pos += 1;
            }
            else {
                properties.insert(attr_name, PropertyValue::Bool(true));
                continue;
            }

            self.skip_whitespace(content, pos);
            let attr_value = self.parse_attribute_value(content, pos);

            if is_binding {
                if attr_value.is_empty() {
                    self.errors.push(ValidationError {
                        message: format!("Empty binding expression for ':{}' on element '{}'", attr_name, tag_name),
                    });
                }
                else {
                    properties.insert(attr_name.clone(), PropertyValue::Binding(attr_value.clone()));
                    self.bindings.push(DataBinding {
                        target: element_id,
                        property: attr_name,
                        expression: attr_value,
                        direction: BindingDirection::OneWay,
                    });
                }
            }
            else {
                let value = Self::infer_property_value(&attr_value);
                properties.insert(attr_name, value);
            }
        }

        if !self_closing {
            loop {
                self.skip_whitespace(content, pos);

                if *pos >= content.len() {
                    break;
                }

                if content[*pos..].starts_with("</") {
                    *pos += 2;
                    self.skip_whitespace(content, pos);
                    let closing_tag = self.parse_identifier(content, pos);
                    self.skip_whitespace(content, pos);
                    if *pos < content.len() && content[*pos..].starts_with('>') {
                        *pos += 1;
                    }
                    let _ = closing_tag;
                    break;
                }

                if content[*pos..].starts_with('<') {
                    let child = self.parse_element_recursive(content, pos)?;
                    children.push(child);
                }
                else {
                    let text = self.parse_text_content(content, pos);
                    if !text.trim().is_empty() {
                        let text_id = self.alloc_id();
                        let mut text_props = HashMap::new();
                        text_props.insert("value".to_string(), PropertyValue::String(text.trim().to_string()));
                        children.push(ElementIr {
                            id: text_id,
                            component_type: "Text".to_string(),
                            properties: text_props,
                            children: Vec::new(),
                        });
                    }
                }
            }
        }

        Ok(ElementIr { id: element_id, component_type: tag_name, properties, children })
    }

    /// 解析标识符
    fn parse_identifier(&self, content: &str, pos: &mut usize) -> String {
        let start = *pos;
        while *pos < content.len() {
            let ch = content.as_bytes()[*pos];
            if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-' {
                *pos += 1;
            }
            else {
                break;
            }
        }
        content[start..*pos].to_string()
    }

    /// 解析属性值（引号内的字符串）
    fn parse_attribute_value(&self, content: &str, pos: &mut usize) -> String {
        if *pos >= content.len() {
            return String::new();
        }

        let quote = content.as_bytes()[*pos];
        if quote != b'"' && quote != b'\'' {
            let start = *pos;
            while *pos < content.len() {
                let ch = content.as_bytes()[*pos];
                if ch.is_ascii_whitespace() || ch == b'>' || ch == b'/' {
                    break;
                }
                *pos += 1;
            }
            return content[start..*pos].to_string();
        }

        *pos += 1;
        let start = *pos;
        while *pos < content.len() && content.as_bytes()[*pos] != quote {
            *pos += 1;
        }
        let value = content[start..*pos].to_string();
        if *pos < content.len() {
            *pos += 1;
        }
        value
    }

    /// 解析文本内容
    fn parse_text_content(&self, content: &str, pos: &mut usize) -> String {
        let start = *pos;
        while *pos < content.len() && !content[*pos..].starts_with('<') {
            *pos += 1;
        }
        content[start..*pos].to_string()
    }

    /// 跳过空白字符
    fn skip_whitespace(&self, content: &str, pos: &mut usize) {
        while *pos < content.len() && content.as_bytes()[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
    }

    /// 推断属性值类型
    fn infer_property_value(value: &str) -> PropertyValue {
        if value == "true" {
            PropertyValue::Bool(true)
        }
        else if value == "false" {
            PropertyValue::Bool(false)
        }
        else if let Ok(i) = value.parse::<i64>() {
            PropertyValue::Int(i)
        }
        else if let Ok(f) = value.parse::<f64>() {
            PropertyValue::Float(f)
        }
        else {
            PropertyValue::String(value.to_string())
        }
    }

    /// 验证元素树
    fn validate_element(&mut self, element: &ElementIr) {
        if self.registry.lookup(&element.component_type).is_none() {
            self.errors.push(ValidationError { message: format!("Unknown component type: {}", element.component_type) });
            for child in &element.children {
                self.validate_element(child);
            }
            return;
        }

        if let Some(schema) = self.registry.lookup(&element.component_type) {
            for (prop_name, prop_value) in &element.properties {
                self.validate_property(&element.component_type, schema, prop_name, prop_value);
            }

            let element_id = element.id;
            let binding_props: Vec<String> =
                self.bindings.iter().filter(|b| b.target == element_id).map(|b| b.property.clone()).collect();
            for prop in &binding_props {
                self.validate_binding(schema, prop);
            }

            let event_types: Vec<String> =
                self.events.iter().filter(|e| e.target == element_id).map(|e| e.event_type.clone()).collect();
            for evt in &event_types {
                self.validate_event(schema, evt);
            }
        }

        for child in &element.children {
            self.validate_element(child);
        }
    }

    /// 验证组件类型是否已注册
    pub fn validate_component_type(&self, type_name: &str) -> WidgetResult<()> {
        if self.registry.lookup(type_name).is_none() {
            return Err(WidgetError::SemanticError(format!("Unknown component type: {}", type_name)));
        }
        Ok(())
    }

    /// 验证属性是否存在于组件 schema 中
    fn validate_property(
        &mut self,
        component_type: &str,
        schema: &crate::registry::ComponentSchema,
        prop_name: &str,
        _prop_value: &PropertyValue,
    ) {
        if !schema.properties.contains_key(prop_name) {
            self.errors.push(ValidationError {
                message: format!("Unknown property '{}' on component '{}'", prop_name, component_type),
            });
        }
    }

    /// 验证数据绑定属性是否可绑定
    fn validate_binding(&mut self, schema: &crate::registry::ComponentSchema, property_name: &str) {
        if let Some(prop_schema) = schema.properties.get(property_name) {
            if !prop_schema.bindable {
                self.errors.push(ValidationError {
                    message: format!("Property '{}' on '{}' is not bindable", property_name, schema.type_name),
                });
            }
        }
    }

    /// 验证事件是否存在于组件 schema 中
    fn validate_event(&mut self, schema: &crate::registry::ComponentSchema, event_type: &str) {
        if !schema.events.contains_key(event_type) {
            self.errors.push(ValidationError {
                message: format!("Unknown event '{}' on component '{}'", event_type, schema.type_name),
            });
        }
    }

    /// 收集组件依赖
    fn collect_dependencies(&self, element: &ElementIr, deps: &mut HashMap<String, crate::template::ir::ComponentDependency>) {
        if let Some(schema) = self.registry.lookup(&element.component_type) {
            deps.entry(element.component_type.clone()).or_insert_with(|| crate::template::ir::ComponentDependency {
                type_id: element.component_type.clone(),
                source_module: schema.source_module.clone(),
            });
        }
        for child in &element.children {
            self.collect_dependencies(child, deps);
        }
    }

    /// 从 oak-voc AST 直接验证并生成 TemplateIr
    pub fn validate_from_ast(&mut self, ast: &VxDocument) -> WidgetResult<TemplateIr> {
        self.errors.clear();
        self.bindings.clear();
        self.events.clear();
        self.next_id = 1;

        let root = match &ast.template {
            Some(template_node) => self.convert_template_node(template_node)?,
            None => {
                let id = self.alloc_id();
                ElementIr { id, component_type: "Layout".to_string(), properties: HashMap::new(), children: Vec::new() }
            }
        };

        self.validate_element(&root);

        if !self.errors.is_empty() {
            let messages: Vec<String> = self.errors.iter().map(|e| e.message.clone()).collect();
            return Err(WidgetError::SemanticError(format!("Template validation failed: {}", messages.join("; "))));
        }

        let mut dependencies = HashMap::new();
        self.collect_dependencies(&root, &mut dependencies);

        Ok(TemplateIr {
            root,
            dependencies,
            bindings: std::mem::take(&mut self.bindings),
            events: std::mem::take(&mut self.events),
        })
    }

    /// 将 oak-voc TemplateNode 转换为 ElementIr
    fn convert_template_node(&mut self, node: &oak_voc::ast::TemplateNode) -> WidgetResult<ElementIr> {
        match node {
            oak_voc::ast::TemplateNode::Text(text_node) => {
                let id = self.alloc_id();
                let mut props = HashMap::new();
                if !text_node.value.is_empty() {
                    props.insert("value".to_string(), PropertyValue::String(text_node.value.clone()));
                }
                Ok(ElementIr { id, component_type: "Text".to_string(), properties: props, children: Vec::new() })
            }
            oak_voc::ast::TemplateNode::Element { tag, attributes, children } => {
                let element_id = self.alloc_id();
                let mut properties: HashMap<String, PropertyValue> = HashMap::new();
                let mut child_elements: Vec<ElementIr> = Vec::new();

                for attr in attributes {
                    let name = &attr.name.value;
                    let value = &attr.value.value;

                    if name.starts_with('@') {
                        let event_name = &name[1..];
                        if event_name.is_empty() {
                            self.errors
                                .push(ValidationError { message: format!("Empty event name on element '{}'", tag.value) });
                        }
                        else if value.is_empty() {
                            self.errors.push(ValidationError {
                                message: format!("Empty handler for event '@{}' on element '{}'", event_name, tag.value),
                            });
                        }
                        else {
                            self.events.push(EventBinding {
                                target: element_id,
                                event_type: event_name.to_string(),
                                handler: value.clone(),
                            });
                        }
                    }
                    else if name.starts_with(':') {
                        let prop_name = &name[1..];
                        if value.is_empty() {
                            self.errors.push(ValidationError {
                                message: format!("Empty binding expression for ':{}' on element '{}'", prop_name, tag.value),
                            });
                        }
                        else {
                            properties.insert(prop_name.to_string(), PropertyValue::Binding(value.clone()));
                            self.bindings.push(DataBinding {
                                target: element_id,
                                property: prop_name.to_string(),
                                expression: value.clone(),
                                direction: BindingDirection::OneWay,
                            });
                        }
                    }
                    else {
                        let pv = Self::infer_property_value(value);
                        properties.insert(name.clone(), pv);
                    }
                }

                for child in children {
                    let child_ir = self.convert_template_node(child)?;
                    child_elements.push(child_ir);
                }

                Ok(ElementIr { id: element_id, component_type: tag.value.clone(), properties, children: child_elements })
            }
        }
    }
}
