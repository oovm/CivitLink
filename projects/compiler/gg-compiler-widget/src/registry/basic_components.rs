use std::collections::HashMap;

use super::types::{ComponentRegistry, ComponentSchema, EventSchema, PropertySchema, PropertyType};

impl ComponentRegistry {
    /// 注册 Button 组件
    pub(super) fn register_button(&mut self) {
        self.builtins.insert("Button".to_string());
        self.register(ComponentSchema {
            type_name: "Button".to_string(),
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
                    "text".to_string(),
                    PropertySchema {
                        name: "text".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "onClick".to_string(),
                    PropertySchema {
                        name: "onClick".to_string(),
                        property_type: PropertyType::Custom("EventHandler".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::from([("click".to_string(), EventSchema { name: "click".to_string(), parameters: vec![] })]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Text 组件
    pub(super) fn register_text(&mut self) {
        self.builtins.insert("Text".to_string());
        self.register(ComponentSchema {
            type_name: "Text".to_string(),
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
                    "value".to_string(),
                    PropertySchema {
                        name: "value".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Input 组件
    pub(super) fn register_input(&mut self) {
        self.builtins.insert("Input".to_string());
        self.register(ComponentSchema {
            type_name: "Input".to_string(),
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
                    "value".to_string(),
                    PropertySchema {
                        name: "value".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "onChange".to_string(),
                    PropertySchema {
                        name: "onChange".to_string(),
                        property_type: PropertyType::Custom("EventHandler".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "placeholder".to_string(),
                    PropertySchema {
                        name: "placeholder".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
            ]),
            events: HashMap::from([(
                "change".to_string(),
                EventSchema { name: "change".to_string(), parameters: vec![PropertyType::String] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Image 组件
    pub(super) fn register_image(&mut self) {
        self.builtins.insert("Image".to_string());
        self.register(ComponentSchema {
            type_name: "Image".to_string(),
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
                    "src".to_string(),
                    PropertySchema {
                        name: "src".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Panel 组件
    pub(super) fn register_panel(&mut self) {
        self.builtins.insert("Panel".to_string());
        self.register(ComponentSchema {
            type_name: "Panel".to_string(),
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
            ]),
            events: HashMap::new(),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 ScrollView 组件
    pub(super) fn register_scroll_view(&mut self) {
        self.builtins.insert("ScrollView".to_string());
        self.register(ComponentSchema {
            type_name: "ScrollView".to_string(),
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
                    "scroll_direction".to_string(),
                    PropertySchema {
                        name: "scroll_direction".to_string(),
                        property_type: PropertyType::Enum(vec![
                            "horizontal".to_string(),
                            "vertical".to_string(),
                            "both".to_string(),
                        ]),
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
