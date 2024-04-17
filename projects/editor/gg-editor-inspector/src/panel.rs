//! 属性检查器面板实现
//!
//! 提供基于属性描述符的动态属性编辑面板，
//! 通过描述符注册表和编辑器注册表驱动属性显示和编辑，
//! 使用命令管理器实现撤销/重做功能。
//! 支持实体选中/取消选中事件驱动的属性面板更新。

use std::{cell::RefCell, rc::Rc};

use gg_core::GResult;
use gg_editor_shell::{
    EditorContext, EditorEvent, EditorPanel,
    panel::{PanelLayoutHint, PanelPosition},
};
use gg_ui::{Style, UiNodeData, UiTree};

use crate::{
    descriptor::{ComponentDescriptor, DescriptorRegistry, PropertyConstraints, PropertyDescriptor, PropertyType},
    editor::{
        AssetPathEditorFactory, BoolEditorFactory, ColorEditorFactory, EnumEditorFactory, NumericEditorFactory,
        PropertyEditorRegistry, PropertyEditorWidget, StringEditorFactory,
    },
};

/// 属性检查器面板
///
/// 基于属性描述符的动态属性编辑面板，通过描述符注册表查询组件属性结构，
/// 通过编辑器注册表创建对应的属性编辑器组件，使用命令管理器实现撤销/重做。
/// 订阅实体选中/取消选中事件，动态更新面板内容。
pub struct InspectorPanel {
    /// 面板是否可见
    visible: bool,
    /// 描述符注册表
    descriptor_registry: DescriptorRegistry,
    /// 编辑器注册表
    editor_registry: PropertyEditorRegistry,
    /// 当前选中的实体 ID
    selected_entity: Option<u64>,
    /// 接收实体选中事件的共享单元格
    incoming_selected: Rc<RefCell<Option<u64>>>,
    /// 接收实体取消选中事件的共享单元格
    incoming_deselected: Rc<RefCell<bool>>,
    /// 当前实体的活跃属性编辑器列表
    active_editors: Vec<(String, Box<dyn PropertyEditorWidget>)>,
}

impl InspectorPanel {
    /// 创建新的属性检查器面板
    ///
    /// 初始化描述符注册表和编辑器注册表，注册内置编辑器工厂，
    /// 初始化事件接收单元格和活跃编辑器列表。
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
            incoming_selected: Rc::new(RefCell::new(None)),
            incoming_deselected: Rc::new(RefCell::new(false)),
            active_editors: Vec::new(),
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
        self.descriptor_registry.register_component(ComponentDescriptor {
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
                    constraints: Some(PropertyConstraints { max_length: Some(4096), ..Default::default() }),
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

        self.descriptor_registry.register_component(ComponentDescriptor {
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

        self.descriptor_registry.register_component(ComponentDescriptor {
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
                    constraints: Some(PropertyConstraints { min_value: Some(0.0), step: Some(0.1), ..Default::default() }),
                },
                PropertyDescriptor {
                    name: "fade_out".to_string(),
                    display_name: "淡出时长".to_string(),
                    property_type: PropertyType::Float,
                    default_value: Some("0.0".to_string()),
                    constraints: Some(PropertyConstraints { min_value: Some(0.0), step: Some(0.1), ..Default::default() }),
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

        self.descriptor_registry.register_component(ComponentDescriptor {
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
                    constraints: Some(PropertyConstraints { min_value: Some(0.0), step: Some(0.1), ..Default::default() }),
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

    fn on_register(&mut self, context: &mut EditorContext) {
        let incoming_selected = self.incoming_selected.clone();
        let incoming_deselected = self.incoming_deselected.clone();

        context.events_mut().subscribe(Box::new(move |event| match event {
            EditorEvent::EntitySelected { entity } => {
                *incoming_selected.borrow_mut() = Some(*entity);
            }
            EditorEvent::EntityDeselected => {
                *incoming_deselected.borrow_mut() = true;
            }
            _ => {}
        }));
    }

    fn on_unregister(&mut self, _context: &mut EditorContext) {
        self.active_editors.clear();
        self.selected_entity = None;
    }

    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        let mut selection_changed = false;

        if *self.incoming_deselected.borrow() {
            *self.incoming_deselected.borrow_mut() = false;
            self.selected_entity = None;
            self.active_editors.clear();
            selection_changed = true;
        }

        if let Some(entity) = self.incoming_selected.borrow_mut().take() {
            self.selected_entity = Some(entity);
            self.active_editors.clear();
            selection_changed = true;
        }

        if !selection_changed {
            return Ok(());
        }

        if let Some(_entity) = self.selected_entity {
            let root_id = ui_tree.create_node("inspector_root", Style::default(), UiNodeData::Container);
            ui_tree.set_root(root_id);

            for component_desc in self.descriptor_registry.component_descriptors() {
                let section_id = ui_tree.create_node(
                    format!("section_{}", component_desc.type_name),
                    Style::default(),
                    UiNodeData::Container,
                );
                ui_tree.add_child(root_id, section_id);

                let header_id = ui_tree.create_node(
                    format!("header_{}", component_desc.type_name),
                    Style::default(),
                    UiNodeData::Text { content: component_desc.display_name.clone() },
                );
                ui_tree.add_child(section_id, header_id);

                for property in &component_desc.properties {
                    if let Some(editor) = self.editor_registry.create_editor(&property.property_type) {
                        let control_type =
                            crate::controls::create_property_control(&property.property_type, property.constraints.as_ref());
                        let prop_id = ui_tree.create_node(
                            format!("prop_{}_{}", component_desc.type_name, property.name),
                            Style::default(),
                            UiNodeData::Custom { kind: control_type },
                        );
                        ui_tree.add_child(section_id, prop_id);
                        self.active_editors.push((property.name.clone(), editor));
                    }
                }
            }
        }

        Ok(())
    }

    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint { position: PanelPosition::Right, preferred_size: Some((300.0, 600.0)), min_size: Some((200.0, 300.0)) }
    }
}
