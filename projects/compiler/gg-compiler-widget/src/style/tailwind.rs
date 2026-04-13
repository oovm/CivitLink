//! Tailwind CSS 子集处理器模块

use crate::{
    error::{WidgetError, WidgetResult},
    style::ir::StyleValue,
};

/// Tailwind 配置
#[derive(Debug, Clone)]
pub struct TailwindConfig {
    /// 是否启用响应式前缀
    pub responsive: bool,
    /// 是否启用状态变体
    pub state_variants: bool,
}

/// Tailwind 处理器
pub struct TailwindProcessor {
    /// Tailwind 配置
    config: TailwindConfig,
}

impl TailwindProcessor {
    /// 创建新的 Tailwind 处理器
    pub fn new(config: TailwindConfig) -> Self {
        Self { config }
    }

    /// 解析 Tailwind 类名并生成样式值
    pub fn process_class(&self, class_name: &str) -> WidgetResult<StyleValue> {
        if class_name.starts_with("flex-") {
            Ok(StyleValue::Enum("flex".to_string()))
        }
        else if class_name.starts_with("text-") {
            Ok(StyleValue::String(class_name.to_string()))
        }
        else if class_name.starts_with("bg-") {
            Ok(StyleValue::Color(class_name.to_string()))
        }
        else {
            Err(WidgetError::SemanticError(format!("Unknown Tailwind class: {}", class_name)))
        }
    }

    /// 检查是否为响应式前缀
    pub fn is_responsive_prefix(&self, prefix: &str) -> bool {
        self.config.responsive && matches!(prefix, "sm" | "md" | "lg" | "xl")
    }

    /// 检查是否为状态变体
    pub fn is_state_variant(&self, variant: &str) -> bool {
        self.config.state_variants && matches!(variant, "hover" | "focus" | "active")
    }
}

impl Default for TailwindProcessor {
    fn default() -> Self {
        Self::new(TailwindConfig { responsive: true, state_variants: true })
    }
}
