//! Schema 验证器模块
//! 对 SchemaIr 进行语义验证，确保类型引用、主键约束等规则

use crate::{
    error::{SchemaError, SchemaResult},
    ir::{FieldTypeIr, SchemaIr},
};

/// Schema 验证器，对 SchemaIr 执行语义检查
pub struct SchemaValidator;

impl SchemaValidator {
    /// 创建新的 Schema 验证器
    pub fn new() -> Self {
        Self
    }

    /// 验证 SchemaIr 的语义正确性
    pub fn validate(&self, ir: &SchemaIr) -> SchemaResult<()> {
        self.validate_model_primary_keys(ir)?;
        self.validate_field_types(ir)?;
        self.validate_references(ir)?;
        self.validate_enum_variants(ir)?;
        self.validate_service_methods(ir)?;
        Ok(())
    }

    fn validate_model_primary_keys(&self, ir: &SchemaIr) -> SchemaResult<()> {
        for model in &ir.models {
            let has_pk = model.annotations.iter().any(|a| a.name == "primary_key")
                || model.fields.iter().any(|f| f.annotations.iter().any(|a| a.name == "primary_key"));
            if !has_pk {
                return Err(SchemaError::ValidationError(format!("模型 '{}' 缺少主键注解 (@primary_key)", model.name)));
            }
        }
        Ok(())
    }

    fn validate_field_types(&self, ir: &SchemaIr) -> SchemaResult<()> {
        for model in &ir.models {
            for field in &model.fields {
                self.validate_single_field_type(&field.field_type, ir)?;
            }
        }
        for message in &ir.messages {
            for field in &message.fields {
                self.validate_single_field_type(&field.field_type, ir)?;
            }
        }
        Ok(())
    }

    fn validate_single_field_type(&self, field_type: &FieldTypeIr, ir: &SchemaIr) -> SchemaResult<()> {
        match field_type {
            FieldTypeIr::Custom(name) => {
                let model_exists = ir.models.iter().any(|m| m.name == *name);
                let enum_exists = ir.enums.iter().any(|e| e.name == *name);
                let message_exists = ir.messages.iter().any(|m| m.name == *name);
                if !model_exists && !enum_exists && !message_exists {
                    return Err(SchemaError::ValidationError(format!("自定义类型 '{}' 未在当前 Schema 中定义", name)));
                }
            }
            FieldTypeIr::Array(inner) | FieldTypeIr::RefArray(inner) | FieldTypeIr::Optional(inner) => {
                self.validate_single_field_type(inner, ir)?;
            }
            FieldTypeIr::Map(key, value) => {
                self.validate_single_field_type(key, ir)?;
                self.validate_single_field_type(value, ir)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_references(&self, ir: &SchemaIr) -> SchemaResult<()> {
        let model_names: Vec<&str> = ir.models.iter().map(|m| m.name.as_str()).collect();
        for model in &ir.models {
            for field in &model.fields {
                for annotation in &field.annotations {
                    if annotation.name == "references" {
                        if let Some(arg) = annotation.arguments.first() {
                            let ref_target = arg.value.split('.').next().unwrap_or(&arg.value);
                            if !model_names.contains(&ref_target) {
                                return Err(SchemaError::ValidationError(format!(
                                    "字段 '{}.{}' 的 @references 注解引用了不存在的模型 '{}'",
                                    model.name, field.name, ref_target
                                )));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_enum_variants(&self, ir: &SchemaIr) -> SchemaResult<()> {
        for enum_def in &ir.enums {
            let mut seen_values = std::collections::HashSet::new();
            for variant in &enum_def.variants {
                if !seen_values.insert(variant.value) {
                    return Err(SchemaError::ValidationError(format!(
                        "枚举 '{}' 中存在重复的变体值 {}",
                        enum_def.name, variant.value
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_service_methods(&self, ir: &SchemaIr) -> SchemaResult<()> {
        let message_names: Vec<&str> = ir.messages.iter().map(|m| m.name.as_str()).collect();
        for service in &ir.services {
            for method in &service.methods {
                if !message_names.contains(&method.request_type.as_str()) {
                    return Err(SchemaError::ValidationError(format!(
                        "服务 '{}.{}' 的请求类型 '{}' 未定义",
                        service.name, method.name, method.request_type
                    )));
                }
                if !message_names.contains(&method.response_type.as_str()) {
                    return Err(SchemaError::ValidationError(format!(
                        "服务 '{}.{}' 的响应类型 '{}' 未定义",
                        service.name, method.name, method.response_type
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}
