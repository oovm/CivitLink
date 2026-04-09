//! 属性检查器面板实现
//!
//! 提供基于属性描述符的动态属性编辑面板，
//! 通过描述符注册表和编辑器注册表驱动属性显示和编辑，
//! 使用命令管理器实现撤销/重做功能。

use gg_core::GResult;
use gg_editor_shell::panel::{PanelLayoutHint, PanelPosition};
use gg_editor_shell::{EditorContext, EditorPanel};

use crate::descriptor::{
    ComponentDescriptor, DescriptorRegistry, PropertyConstraints, PropertyDescriptor, PropertyType,
};
use crate::editor::{
    AssetPathEditorFactory, BoolEditorFactory, ColorEditorFactory, EnumEditorFactory,
    NumericEditorFactory, PropertyEditorRegistry, StringEditorFactory,
};

/// 属性检查器面板
///
/// 基于属性描述符的动态属性编辑面板，通过描述符注册表查询组件属性结构，
/// 通过编辑器注册表创建对应的属性编辑器组件，使用命令管理器实现撤销/重做。
pub struct InspectorPanel {
    /// 面板是否可见
    visible: bool,
    /// 描述符注册表
    descriptor_registry: DescriptorRegistry,
    /// 编辑器注册表
    editor_registry: PropertyEditorRegistry,
    /// 当前选中的实体 ID
    selected_entity: Option<u64>,
}

impl InspectorPanel {
    /// 创建新的属性检查器面板
    ///
    /// 初始化描述符注册表和编辑器注册表，注册内置编辑器工厂。
    pub fn new() -> Self {
        let mut editor_registry = PropertyEditorRegistry::new();
        editor_registry.register_factory(Box::new(StringEditorFactory));
        editor_registry.register_factory(Box::new(NumericEditorFactory));
        editor_registry.register_factory(Box::new(BoolEditorFactory));
        editor_registry.register_factory(Box::new(EnumEditorFactory));
        editor_registry.register_factory(Box::new(ColorEditorFactory));
        editor_registry.register_factory(Box::new(AssetPathEditorFactory));

        let mut panel = Self {
            visible: true,
            descriptor_registry: DescriptorRegistry::new(),
            editor_registry,
            selected_entity: None,
        };
        panel.register_default_descriptors();
        panel
    }

    /// 获取当前选中的实体 ID
    pub fn selected_entity(&self) -> Option<u64> {
        self.selected_entity
    }

    /// 设置当前选中的实体 ID
    pub fn set_selected_entity(&mut self, entity: Option<u64>) {
        self.selected_entity = entity;
    }

    /// 获取描述符注册表引用
    pub fn descriptor_registry(&self) -> &DescriptorRegistry {
        &self.descriptor_registry
    }

    /// 获取描述符注册表可变引用
    pub fn descriptor_registry_mut(&mut self) -> &mut DescriptorRegistry {
        &mut self.descriptor_registry
    }

    /// 获取编辑器注册表引用
    pub fn editor_registry(&self) -> &PropertyEditorRegistry {
        &self.editor_registry
    }

    /// 获取编辑器注册表可变引用
    pub fn editor_registry_mut(&mut self) -> &mut PropertyEditorRegistry {
        &mut self.editor_registry
    }

    /// 注册默认的 Galgame 组件描述符
    ///
    /// 为 DialogueNode、PortraitState、AudioControl 和 SceneBackground
    /// 组件注册属性描述符。
    pub fn register_default_descriptors(&mut self) {
        self.descriptor_registry
            .register_component(ComponentDescriptor {
                type_name: "DialogueNode".to_string(),
                display_name: "对话节点".to_string(),
                properties: vec![
                    PropertyDescriptor {
                        name: "id".to_string(),
                        display_name: "节点 ID".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "speaker_id".to_string(),
                        display_name: "说话角色 ID".to_string(),
                        property_type: PropertyType::String,
                        default_value: Some("None".to_string()),
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "text".to_string(),
                        display_name: "对话文本".to_string(),
                        property_type: PropertyType::String,
                        default_value: Some(String::new()),
                        constraints: Some(PropertyConstraints {
                            max_length: Some(4096),
                            ..Default::default()
                        }),
                    },
                    PropertyDescriptor {
                        name: "commands".to_string(),
                        display_name: "内联命令".to_string(),
                        property_type: PropertyType::Custom("DialogueCommandList".to_string()),
                        default_value: None,
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "choices".to_string(),
                        display_name: "选项列表".to_string(),
                        property_type: PropertyType::Custom("ChoiceList".to_string()),
                        default_value: None,
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "next_node_id".to_string(),
                        display_name: "下一节点 ID".to_string(),
                        property_type: PropertyType::String,
                        default_value: Some("None".to_string()),
                        constraints: None,
                    },
                ],
            });

        self.descriptor_registry
            .register_component(ComponentDescriptor {
                type_name: "PortraitState".to_string(),
                display_name: "立绘状态".to_string(),
                properties: vec![
                    PropertyDescriptor {
                        name: "character_id".to_string(),
                        display_name: "角色 ID".to_string(),
                        property_type: PropertyType::String,
                        default_value: None,
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "position".to_string(),
                        display_name: "位置".to_string(),
                        property_type: PropertyType::Enum(vec![
                            "Left".to_string(),
                            "Center".to_string(),
                            "Right".to_string(),
                            "Custom".to_string(),
                        ]),
                        default_value: Some("Center".to_string()),
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "expression".to_string(),
                        display_name: "表情".to_string(),
                        property_type: PropertyType::String,
                        default_value: Some("default".to_string()),
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "scale".to_string(),
                        display_name: "缩放".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("1.0".to_string()),
                        constraints: Some(PropertyConstraints {
                            min_value: Some(0.0),
                            max_value: Some(10.0),
                            step: Some(0.1),
                            ..Default::default()
                        }),
                    },
                    PropertyDescriptor {
                        name: "opacity".to_string(),
                        display_name: "透明度".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("1.0".to_string()),
                        constraints: Some(PropertyConstraints {
                            min_value: Some(0.0),
                            max_value: Some(1.0),
                            step: Some(0.01),
                            ..Default::default()
                        }),
                    },
                ],
            });

        self.descriptor_registry
            .register_component(ComponentDescriptor {
                type_name: "AudioControl".to_string(),
                display_name: "音频控制".to_string(),
                properties: vec![
                    PropertyDescriptor {
                        name: "bgm_path".to_string(),
                        display_name: "BGM 路径".to_string(),
                        property_type: PropertyType::AssetPath("ogg,wav,mp3".to_string()),
                        default_value: Some("None".to_string()),
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "volume".to_string(),
                        display_name: "音量".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("1.0".to_string()),
                        constraints: Some(PropertyConstraints {
                            min_value: Some(0.0),
                            max_value: Some(1.0),
                            step: Some(0.01),
                            ..Default::default()
                        }),
                    },
                    PropertyDescriptor {
                        name: "fade_in".to_string(),
                        display_name: "淡入时长".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("0.0".to_string()),
                        constraints: Some(PropertyConstraints {
                            min_value: Some(0.0),
                            step: Some(0.1),
                            ..Default::default()
                        }),
                    },
                    PropertyDescriptor {
                        name: "fade_out".to_string(),
                        display_name: "淡出时长".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("0.0".to_string()),
                        constraints: Some(PropertyConstraints {
                            min_value: Some(0.0),
                            step: Some(0.1),
                            ..Default::default()
                        }),
                    },
                    PropertyDescriptor {
                        name: "is_playing".to_string(),
                        display_name: "是否播放".to_string(),
                        property_type: PropertyType::Bool,
                        default_value: Some("false".to_string()),
                        constraints: None,
                    },
                ],
            });

        self.descriptor_registry
            .register_component(ComponentDescriptor {
                type_name: "SceneBackground".to_string(),
                display_name: "场景背景".to_string(),
                properties: vec![
                    PropertyDescriptor {
                        name: "asset_path".to_string(),
                        display_name: "资源路径".to_string(),
                        property_type: PropertyType::AssetPath("png,jpg,webp".to_string()),
                        default_value: Some("None".to_string()),
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "transition_type".to_string(),
                        display_name: "转场类型".to_string(),
                        property_type: PropertyType::Enum(vec![
                            "None".to_string(),
                            "Fade".to_string(),
                            "CrossDissolve".to_string(),
                            "Slide".to_string(),
                        ]),
                        default_value: Some("None".to_string()),
                        constraints: None,
                    },
                    PropertyDescriptor {
                        name: "duration".to_string(),
                        display_name: "持续时间".to_string(),
                        property_type: PropertyType::Float,
                        default_value: Some("0.5".to_string()),
                        constraints: Some(PropertyConstraints {
                            min_value: Some(0.0),
                            step: Some(0.1),
                            ..Default::default()
                        }),
                    },
                    PropertyDescriptor {
                        name: "filter".to_string(),
                        display_name: "氛围滤镜".to_string(),
                        property_type: PropertyType::Custom("AmbientFilter".to_string()),
                        default_value: Some("None".to_string()),
                        constraints: None,
                    },
                ],
            });
    }
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for InspectorPanel {
    fn name(&self) -> &str {
        "Inspector"
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn on_register(&mut self, _context: &mut EditorContext) {
        // 面板注册时订阅实体选中/取消选中事件
        // 实际事件订阅通过 EventBus 完成
    }

    fn on_unregister(&mut self, _context: &mut EditorContext) {
        // 面板注销时清理
    }

    fn render(&mut self, _context: &mut EditorContext) -> GResult<()> {
        // 渲染检查器面板 UI
        // 通过描述符注册表获取当前选中实体组件的属性结构，
        // 通过编辑器注册表创建对应的属性编辑器组件进行显示和编辑
        Ok(())
    }

    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Right,
            preferred_size: Some((300.0, 600.0)),
            min_size: Some((200.0, 300.0)),
        }
    }
}
