use std::collections::HashMap;

use super::types::{ComponentRegistry, ComponentSchema, PropertySchema, PropertyType};

impl ComponentRegistry {
    /// 注册 Layout 组件
    pub(super) fn register_layout(&mut self) {
        self.builtins.insert("Layout".to_string());
        self.register(ComponentSchema {
            type_name: "Layout".to_string(),
            properties: HashMap::from([
                (
                    "id".to_string(),
                    PropertySchema {
                        name: "id".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "class".to_string(),
                    PropertySchema {
                        name: "class".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "style".to_string(),
                    PropertySchema {
                        name: "style".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "direction".to_string(),
                    PropertySchema {
                        name: "direction".to_string(),
                        property_type: PropertyType::Enum(vec!["row".to_string(), "column".to_string()]),
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "align".to_string(),
                    PropertySchema {
                        name: "align".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "justify".to_string(),
                    PropertySchema {
                        name: "justify".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "gap".to_string(),
                    PropertySchema {
                        name: "gap".to_string(),
                        property_type: PropertyType::Float,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "padding".to_string(),
                    PropertySchema {
                        name: "padding".to_string(),
                        property_type: PropertyType::Float,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Stack 组件
    pub(super) fn register_stack(&mut self) {
        self.builtins.insert("Stack".to_string());
        self.register(ComponentSchema {
            type_name: "Stack".to_string(),
            properties: HashMap::from([
                (
                    "id".to_string(),
                    PropertySchema {
                        name: "id".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "class".to_string(),
                    PropertySchema {
                        name: "class".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "style".to_string(),
                    PropertySchema {
                        name: "style".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "orientation".to_string(),
                    PropertySchema {
                        name: "orientation".to_string(),
                        property_type: PropertyType::Enum(vec!["horizontal".to_string(), "vertical".to_string()]),
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }
}
