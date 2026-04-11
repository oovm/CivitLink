//! Template 代码生成模块

use crate::{
    artifact::{SerializedBinding, SerializedEventHandler, TemplateBundle},
    error::{WidgetError, WidgetResult},
    template::ir::TemplateIr,
};

/// Template 代码生成器
pub struct TemplateCodegen;

impl TemplateCodegen {
    /// 创建新的 Template 代码生成器
    pub fn new() -> Self {
        Self
    }

    /// 将 TemplateIr 编译为 TemplateBundle
    pub fn generate(ir: &TemplateIr) -> WidgetResult<TemplateBundle> {
        let node_tree = serde_json::to_vec(&ir.root)
            .map_err(|e| WidgetError::CodegenError(format!("Failed to serialize node tree: {}", e)))?;

        let bindings: Vec<SerializedBinding> = ir
            .bindings
            .iter()
            .map(|b| SerializedBinding {
                target_id: b.target.to_string(),
                property: b.property.clone(),
                expression: b.expression.clone(),
            })
            .collect();

        let event_handlers: Vec<SerializedEventHandler> = ir
            .events
            .iter()
            .map(|e| SerializedEventHandler {
                target_id: e.target.to_string(),
                event_type: e.event_type.clone(),
                handler: e.handler.clone(),
            })
            .collect();

        Ok(TemplateBundle {
            node_tree,
            component_refs: std::collections::HashMap::new(),
            bindings,
            event_handlers,
            resource_refs: vec![],
        })
    }
}

impl Default for TemplateCodegen {
    fn default() -> Self {
        Self::new()
    }
}
