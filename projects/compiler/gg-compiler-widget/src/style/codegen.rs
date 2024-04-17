//! Style 代码生成模块

use std::collections::HashMap;

use crate::{
    artifact::{RuleId, StyleBundle},
    error::{WidgetError, WidgetResult},
    style::ir::StyleIr,
};

/// Style 代码生成器
pub struct StyleCodegen;

impl StyleCodegen {
    /// 创建新的 Style 代码生成器
    pub fn new() -> Self {
        Self
    }

    /// 将 StyleIr 编译为 StyleBundle
    pub fn generate(ir: &StyleIr) -> WidgetResult<StyleBundle> {
        let rules = serde_json::to_vec(&ir.rules)
            .map_err(|e| WidgetError::CodegenError(format!("Failed to serialize style rules: {}", e)))?;

        let mut selector_index: HashMap<String, Vec<RuleId>> = HashMap::new();
        for rule in &ir.rules {
            let key = rule.selector.text.clone();
            selector_index.entry(key).or_default().push(rule.id);
        }

        let theme_values =
            ir.resolved_vars.iter().map(|(k, v)| (k.clone(), serde_json::to_vec(v).unwrap_or_default())).collect();

        Ok(StyleBundle { rules, selector_index, theme_values })
    }
}

impl Default for StyleCodegen {
    fn default() -> Self {
        Self::new()
    }
}
