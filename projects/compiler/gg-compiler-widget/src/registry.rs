//! 组件注册系统模块

use std::collections::{HashMap, HashSet};

/// 属性类型
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    /// 字符串
    String,
    /// 整数
    Int,
    /// 浮点数
    Float,
    /// 布尔值
    Bool,
    /// 颜色
    Color,
    /// 长度
    Length,
    /// 枚举值
    Enum(Vec<String>),
    /// 二维向量
    Vec2,
    /// 三维向量
    Vec3,
    /// 四维向量
    Vec4,
    /// 自定义类型
    Custom(String),
}

/// 属性 Schema
#[derive(Debug, Clone)]
pub struct PropertySchema {
    /// 属性名
    pub name: String,
    /// 属性类型
    pub property_type: PropertyType,
    /// 默认值
    pub default_value: Option<String>,
    /// 是否必需
    pub required: bool,
    /// 是否支持绑定
    pub bindable: bool,
}

/// 事件 Schema
#[derive(Debug, Clone)]
pub struct EventSchema {
    /// 事件名
    pub name: String,
    /// 事件参数类型
    pub parameters: Vec<PropertyType>,
}

/// 组件 Schema
#[derive(Debug, Clone)]
pub struct ComponentSchema {
    /// 组件类型名
    pub type_name: String,
    /// 组件属性列表
    pub properties: HashMap<String, PropertySchema>,
    /// 组件事件列表
    pub events: HashMap<String, EventSchema>,
    /// 是否为容器组件
    pub is_container: bool,
    /// 允许的子组件类型
    pub allowed_children: Vec<String>,
    /// 组件来源模块
    pub source_module: String,
}

/// 组件注册表
pub struct ComponentRegistry {
    /// 已注册的组件
    components: HashMap<String, ComponentSchema>,
    /// 内置组件名称集合
    builtins: HashSet<String>,
}

impl ComponentRegistry {
    /// 创建空的组件注册表
    pub fn new() -> Self {
        Self { components: HashMap::new(), builtins: HashSet::new() }
    }

    /// 创建包含所有内置组件的注册表
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_builtins();
        registry
    }

    /// 注册一个组件
    pub fn register(&mut self, schema: ComponentSchema) {
        self.components.insert(schema.type_name.clone(), schema);
    }

    /// 查询指定名称的组件
    pub fn lookup(&self, type_name: &str) -> Option<&ComponentSchema> {
        self.components.get(type_name)
    }

    /// 注册所有内置组件
    pub fn register_builtins(&mut self) {
        self.register_layout();
        self.register_stack();
        self.register_button();
        self.register_text();
        self.register_input();
        self.register_image();
        self.register_panel();
        self.register_scroll_view();
        self.register_editor_components();
        self.register_widget_components();
    }

    /// 注册 Layout 组件
    fn register_layout(&mut self) {
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
    fn register_stack(&mut self) {
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

    /// 注册 Button 组件
    fn register_button(&mut self) {
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
    fn register_text(&mut self) {
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
    fn register_input(&mut self) {
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
    fn register_image(&mut self) {
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
    fn register_panel(&mut self) {
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
    fn register_scroll_view(&mut self) {
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

    /// 注册编辑器专用组件
    fn register_editor_components(&mut self) {
        self.register_inspector_panel();
        self.register_hierarchy_view();
        self.register_asset_browser();
        self.register_scene_view();
        self.register_toolbar();
        self.register_menu_bar();
        self.register_tab_container();
        self.register_split_view();
        self.register_tree_view();
        self.register_property_field();
    }

    /// 注册 InspectorPanel 组件
    fn register_inspector_panel(&mut self) {
        self.register(ComponentSchema {
            type_name: "InspectorPanel".to_string(),
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
                    "target".to_string(),
                    PropertySchema {
                        name: "target".to_string(),
                        property_type: PropertyType::Custom("EntityRef".to_string()),
                        default_value: None,
                        required: true,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 HierarchyView 组件
    fn register_hierarchy_view(&mut self) {
        self.register(ComponentSchema {
            type_name: "HierarchyView".to_string(),
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
                    "root".to_string(),
                    PropertySchema {
                        name: "root".to_string(),
                        property_type: PropertyType::Custom("EntityRef".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 AssetBrowser 组件
    fn register_asset_browser(&mut self) {
        self.register(ComponentSchema {
            type_name: "AssetBrowser".to_string(),
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
                    "path".to_string(),
                    PropertySchema {
                        name: "path".to_string(),
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
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 SceneView 组件
    fn register_scene_view(&mut self) {
        self.register(ComponentSchema {
            type_name: "SceneView".to_string(),
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
                    "scene".to_string(),
                    PropertySchema {
                        name: "scene".to_string(),
                        property_type: PropertyType::Custom("SceneRef".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 Toolbar 组件
    fn register_toolbar(&mut self) {
        self.register(ComponentSchema {
            type_name: "Toolbar".to_string(),
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
                    "items".to_string(),
                    PropertySchema {
                        name: "items".to_string(),
                        property_type: PropertyType::Custom("ToolbarItems".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 MenuBar 组件
    fn register_menu_bar(&mut self) {
        self.register(ComponentSchema {
            type_name: "MenuBar".to_string(),
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
                    "menus".to_string(),
                    PropertySchema {
                        name: "menus".to_string(),
                        property_type: PropertyType::Custom("MenuItems".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 TabContainer 组件
    fn register_tab_container(&mut self) {
        self.register(ComponentSchema {
            type_name: "TabContainer".to_string(),
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
                    "active_tab".to_string(),
                    PropertySchema {
                        name: "active_tab".to_string(),
                        property_type: PropertyType::Int,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 SplitView 组件
    fn register_split_view(&mut self) {
        self.register(ComponentSchema {
            type_name: "SplitView".to_string(),
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
                (
                    "ratio".to_string(),
                    PropertySchema {
                        name: "ratio".to_string(),
                        property_type: PropertyType::Float,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 TreeView 组件
    fn register_tree_view(&mut self) {
        self.register(ComponentSchema {
            type_name: "TreeView".to_string(),
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
                    "data".to_string(),
                    PropertySchema {
                        name: "data".to_string(),
                        property_type: PropertyType::Custom("TreeNode".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "on_select".to_string(),
                    PropertySchema {
                        name: "on_select".to_string(),
                        property_type: PropertyType::Custom("EventHandler".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::from([(
                "select".to_string(),
                EventSchema { name: "select".to_string(), parameters: vec![PropertyType::Custom("TreeNode".to_string())] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册 PropertyField 组件
    fn register_property_field(&mut self) {
        self.register(ComponentSchema {
            type_name: "PropertyField".to_string(),
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
                    "label".to_string(),
                    PropertySchema {
                        name: "label".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "type".to_string(),
                    PropertySchema {
                        name: "type".to_string(),
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
            ]),
            events: HashMap::from([(
                "change".to_string(),
                EventSchema { name: "change".to_string(), parameters: vec![PropertyType::String] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_editor_ui::widgets".to_string(),
        });
    }

    /// 注册控件组件
    fn register_widget_components(&mut self) {
        self.register_checkbox();
        self.register_combo_box();
        self.register_context_menu();
        self.register_dialog();
        self.register_dropdown();
        self.register_progress_bar();
        self.register_slider();
        self.register_spinner();
        self.register_tab_bar();
        self.register_text_box();
        self.register_tooltip();
    }

    /// 注册 Checkbox 组件
    fn register_checkbox(&mut self) {
        self.builtins.insert("Checkbox".to_string());
        self.register(ComponentSchema {
            type_name: "Checkbox".to_string(),
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
                    "checked".to_string(),
                    PropertySchema {
                        name: "checked".to_string(),
                        property_type: PropertyType::Bool,
                        default_value: Some("false".to_string()),
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "label".to_string(),
                    PropertySchema {
                        name: "label".to_string(),
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
            ]),
            events: HashMap::from([(
                "change".to_string(),
                EventSchema { name: "change".to_string(), parameters: vec![PropertyType::Bool] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 ComboBox 组件
    fn register_combo_box(&mut self) {
        self.builtins.insert("ComboBox".to_string());
        self.register(ComponentSchema {
            type_name: "ComboBox".to_string(),
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
                    "options".to_string(),
                    PropertySchema {
                        name: "options".to_string(),
                        property_type: PropertyType::Custom("StringList".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "selected_index".to_string(),
                    PropertySchema {
                        name: "selected_index".to_string(),
                        property_type: PropertyType::Int,
                        default_value: Some("0".to_string()),
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
            ]),
            events: HashMap::from([(
                "change".to_string(),
                EventSchema { name: "change".to_string(), parameters: vec![PropertyType::Int] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 ContextMenu 组件
    fn register_context_menu(&mut self) {
        self.builtins.insert("ContextMenu".to_string());
        self.register(ComponentSchema {
            type_name: "ContextMenu".to_string(),
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
                    "items".to_string(),
                    PropertySchema {
                        name: "items".to_string(),
                        property_type: PropertyType::Custom("MenuItems".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "onSelect".to_string(),
                    PropertySchema {
                        name: "onSelect".to_string(),
                        property_type: PropertyType::Custom("EventHandler".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::from([(
                "select".to_string(),
                EventSchema { name: "select".to_string(), parameters: vec![PropertyType::Custom("MenuItem".to_string())] },
            )]),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Dialog 组件
    fn register_dialog(&mut self) {
        self.builtins.insert("Dialog".to_string());
        self.register(ComponentSchema {
            type_name: "Dialog".to_string(),
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
                    "title".to_string(),
                    PropertySchema {
                        name: "title".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "buttons".to_string(),
                    PropertySchema {
                        name: "buttons".to_string(),
                        property_type: PropertyType::Custom("DialogButtons".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "modal".to_string(),
                    PropertySchema {
                        name: "modal".to_string(),
                        property_type: PropertyType::Bool,
                        default_value: Some("true".to_string()),
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "onClose".to_string(),
                    PropertySchema {
                        name: "onClose".to_string(),
                        property_type: PropertyType::Custom("EventHandler".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
            ]),
            events: HashMap::from([("close".to_string(), EventSchema { name: "close".to_string(), parameters: vec![] })]),
            is_container: true,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Dropdown 组件
    fn register_dropdown(&mut self) {
        self.builtins.insert("Dropdown".to_string());
        self.register(ComponentSchema {
            type_name: "Dropdown".to_string(),
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
                    "options".to_string(),
                    PropertySchema {
                        name: "options".to_string(),
                        property_type: PropertyType::Custom("StringList".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "selected".to_string(),
                    PropertySchema {
                        name: "selected".to_string(),
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

    /// 注册 ProgressBar 组件
    fn register_progress_bar(&mut self) {
        self.builtins.insert("ProgressBar".to_string());
        self.register(ComponentSchema {
            type_name: "ProgressBar".to_string(),
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
                        property_type: PropertyType::Float,
                        default_value: Some("0.0".to_string()),
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "max_value".to_string(),
                    PropertySchema {
                        name: "max_value".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("100.0".to_string()),
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

    /// 注册 Slider 组件
    fn register_slider(&mut self) {
        self.builtins.insert("Slider".to_string());
        self.register(ComponentSchema {
            type_name: "Slider".to_string(),
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
                        property_type: PropertyType::Float,
                        default_value: Some("0.0".to_string()),
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "min".to_string(),
                    PropertySchema {
                        name: "min".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("0.0".to_string()),
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "max".to_string(),
                    PropertySchema {
                        name: "max".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("1.0".to_string()),
                        required: false,
                        bindable: false,
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
            ]),
            events: HashMap::from([(
                "change".to_string(),
                EventSchema { name: "change".to_string(), parameters: vec![PropertyType::Float] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 Spinner 组件
    fn register_spinner(&mut self) {
        self.builtins.insert("Spinner".to_string());
        self.register(ComponentSchema {
            type_name: "Spinner".to_string(),
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
                    "active".to_string(),
                    PropertySchema {
                        name: "active".to_string(),
                        property_type: PropertyType::Bool,
                        default_value: Some("false".to_string()),
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "size".to_string(),
                    PropertySchema {
                        name: "size".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("24.0".to_string()),
                        required: false,
                        bindable: false,
                    },
                ),
            ]),
            events: HashMap::new(),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 TabBar 组件
    fn register_tab_bar(&mut self) {
        self.builtins.insert("TabBar".to_string());
        self.register(ComponentSchema {
            type_name: "TabBar".to_string(),
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
                    "tabs".to_string(),
                    PropertySchema {
                        name: "tabs".to_string(),
                        property_type: PropertyType::Custom("TabList".to_string()),
                        default_value: None,
                        required: false,
                        bindable: true,
                    },
                ),
                (
                    "active_index".to_string(),
                    PropertySchema {
                        name: "active_index".to_string(),
                        property_type: PropertyType::Int,
                        default_value: Some("0".to_string()),
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
            ]),
            events: HashMap::from([(
                "change".to_string(),
                EventSchema { name: "change".to_string(), parameters: vec![PropertyType::Int] },
            )]),
            is_container: false,
            allowed_children: vec![],
            source_module: "gg_ui::vx_components".to_string(),
        });
    }

    /// 注册 TextBox 组件
    fn register_text_box(&mut self) {
        self.builtins.insert("TextBox".to_string());
        self.register(ComponentSchema {
            type_name: "TextBox".to_string(),
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
                    "placeholder".to_string(),
                    PropertySchema {
                        name: "placeholder".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "readonly".to_string(),
                    PropertySchema {
                        name: "readonly".to_string(),
                        property_type: PropertyType::Bool,
                        default_value: Some("false".to_string()),
                        required: false,
                        bindable: false,
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

    /// 注册 Tooltip 组件
    fn register_tooltip(&mut self) {
        self.builtins.insert("Tooltip".to_string());
        self.register(ComponentSchema {
            type_name: "Tooltip".to_string(),
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
                    "position".to_string(),
                    PropertySchema {
                        name: "position".to_string(),
                        property_type: PropertyType::Enum(vec![
                            "top".to_string(),
                            "bottom".to_string(),
                            "left".to_string(),
                            "right".to_string(),
                            "auto".to_string(),
                        ]),
                        default_value: Some("auto".to_string()),
                        required: false,
                        bindable: false,
                    },
                ),
                (
                    "delay".to_string(),
                    PropertySchema {
                        name: "delay".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("500.0".to_string()),
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

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
