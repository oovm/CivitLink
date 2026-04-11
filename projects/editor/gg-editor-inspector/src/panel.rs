//! 属性检查器面板实现
//!
//! 提供基于属性描述符的动态属性编辑面板，
//! 通过描述符注册表和编辑器注册表驱动属性显示和编辑，
//! 使用命令管理器实现撤销/重做功能。
//! 支持实体选中/取消选中事件驱动的属性面板更新，
//! 通过 `PropertyStore` 资源和 `ReflectionPropertyBinding` 实现 ECS 世界与检查器面板的双向数据绑定。

use std::{cell::RefCell, rc::Rc};

use gg_core::GResult;
use gg_ecs::World;
use gg_editor_shell::{
    EditorContext, EditorEvent, EditorPanel,
    panel::{PanelLayoutHint, PanelPosition},
};
use gg_ui::{Style, UiNodeData, UiTree};
use gg_render::Color;

use crate::{
    binding::{PropertyBinding, ReflectionPropertyBinding},
    descriptor::{ComponentDescriptor, DescriptorRegistry, PropertyConstraints, PropertyDescriptor, PropertyType},
    editor::{
        AssetPathEditorFactory, BoolEditorFactory, ColorEditorFactory, EnumEditorFactory, NumericEditorFactory,
        PropertyEditorRegistry, PropertyEditorWidget, StringEditorFactory,
    },
};

/// 活跃属性条目
///
/// 存储当前选中实体的单个属性编辑状态，
/// 包括组件类型名、属性名、属性绑定和编辑器组件，
/// 用于双向数据绑定和命令系统集成。
pub struct ActivePropertyEntry {
    /// 所属组件类型名称
    pub component_type: String,
    /// 属性名称
    pub property_name: String,
    /// 属性绑定，用于读写 ECS 世界中的属性值
    pub binding: Box<dyn PropertyBinding>,
    /// 属性编辑器组件
    pub editor: Box<dyn PropertyEditorWidget>,
}

/// 属性检查器面板
///
/// 基于属性描述符的动态属性编辑面板，通过描述符注册表查询组件属性结构，
/// 通过编辑器注册表创建对应的属性编辑器组件，使用命令管理器实现撤销/重做。
/// 订阅实体选中/取消选中事件，动态更新面板内容。
/// 通过 `PropertyStore` 资源和 `ReflectionPropertyBinding` 实现 ECS 世界与检查器面板的双向数据绑定。
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
    /// 当前实体的活跃属性条目列表
    active_entries: Vec<ActivePropertyEntry>,
}

impl InspectorPanel {
    /// 创建新的属性检查器面板
    ///
    /// 初始化描述符注册表和编辑器注册表，注册内置编辑器工厂，
    /// 初始化事件接收单元格和活跃属性条目列表。
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
            active_entries: Vec::new(),
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

    /// 获取活跃属性条目列表引用
    pub fn active_entries(&self) -> &[ActivePropertyEntry] {
        &self.active_entries
    }

    /// 提交待处理的属性变更
    ///
    /// 遍历所有活跃属性条目，检查编辑器是否被修改，
    /// 若已修改则创建 `SetPropertyCommand` 并通过 `EditorContext` 执行，
    /// 实现属性变更的撤销/重做支持。
    pub fn apply_pending_changes(&mut self, context: &mut EditorContext) {
        let entity = match self.selected_entity {
            Some(e) => e,
            None => return,
        };

        let pending: Vec<(String, String, String)> = self
            .active_entries
            .iter()
            .filter(|entry| entry.editor.is_modified())
            .map(|entry| {
                (
                    entry.component_type.clone(),
                    entry.property_name.clone(),
                    entry.editor.get_value(),
                )
            })
            .collect();

        for (component_type, property_name, new_value) in pending {
            let binding = Box::new(ReflectionPropertyBinding::new(component_type.clone(), property_name.clone()));
            let description = format!("修改 {}.{}", component_type, property_name);
            let command = crate::binding::SetPropertyCommand::new(binding, entity, new_value, description);
            context.execute_command(Box::new(command));
        }
    }

    /// 设置指定属性的值
    ///
    /// 通过命令系统修改指定实体上某组件属性的值，
    /// 支持撤销/重做。同时更新对应的编辑器组件。
    ///
    /// # 参数
    ///
    /// - `context` - 编辑器上下文
    /// - `component_type` - 组件类型名称
    /// - `property_name` - 属性名称
    /// - `new_value` - 新的属性值（字符串形式）
    pub fn set_property_value(
        &mut self,
        context: &mut EditorContext,
        component_type: &str,
        property_name: &str,
        new_value: String,
    ) {
        let entity = match self.selected_entity {
            Some(e) => e,
            None => return,
        };

        let binding = Box::new(ReflectionPropertyBinding::new(component_type.to_string(), property_name.to_string()));
        let description = format!("修改 {}.{}", component_type, property_name);
        let command = crate::binding::SetPropertyCommand::new(binding, entity, new_value.clone(), description);
        context.execute_command(Box::new(command));

        if let Some(entry) = self
            .active_entries
            .iter_mut()
            .find(|e| e.component_type == component_type && e.property_name == property_name)
        {
            entry.editor.set_value(&new_value);
        }
    }

    /// 从 ECS 世界读取属性值并设置到编辑器
    ///
    /// 通过 `ReflectionPropertyBinding` 从 `PropertyStore` 资源中读取当前属性值，
    /// 若 `PropertyStore` 中无对应值则使用描述符中的默认值，
    /// 然后调用编辑器的 `set_value()` 方法设置初始值。
    fn read_property_values(&mut self, world: &mut World) {
        let entity = match self.selected_entity {
            Some(e) => e,
            None => return,
        };

        for entry in &mut self.active_entries {
            let value = entry.binding.read(world, entity).or_else(|| {
                self.descriptor_registry
                    .get_component(&entry.component_type)
                    .and_then(|desc| {
                        desc.properties
                            .iter()
                            .find(|p| p.name == entry.property_name)
                            .and_then(|p| p.default_value.clone())
                    })
            });
            if let Some(v) = value {
                entry.editor.set_value(&v);
            }
        }
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
        self.active_entries.clear();
        self.selected_entity = None;
    }

    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::PropertyChanged { entity, component, property } => {
                if self.selected_entity == Some(*entity) {
                    if let Some(entry) = self
                        .active_entries
                        .iter_mut()
                        .find(|e| e.component_type == *component && e.property_name == *property)
                    {
                        let world = &mut context.world_mut().ecs_world;
                        if let Some(value) = entry.binding.read(world, *entity) {
                            entry.editor.set_value(&value);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn build_ui(&mut self, context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        self.apply_pending_changes(context);

        let mut selection_changed = false;

        if *self.incoming_deselected.borrow() {
            *self.incoming_deselected.borrow_mut() = false;
            self.selected_entity = None;
            self.active_entries.clear();
            selection_changed = true;
        }

        if let Some(entity) = self.incoming_selected.borrow_mut().take() {
            self.selected_entity = Some(entity);
            self.active_entries.clear();
            selection_changed = true;
        }

        let root_id = ui_tree.create_node("inspector_root", Style::new().with_background_color(Color::new(0.12, 0.12, 0.14, 1.0)), UiNodeData::Container);
        ui_tree.set_root(root_id);

        if let Some(_entity) = self.selected_entity {
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
                        let prop_id = crate::controls::create_property_control(
                            &property.property_type,
                            &property.display_name,
                            property.constraints.as_ref(),
                            ui_tree,
                        );
                        ui_tree.add_child(section_id, prop_id);

                        let binding = Box::new(ReflectionPropertyBinding::new(
                            component_desc.type_name.clone(),
                            property.name.clone(),
                        ));

                        self.active_entries.push(ActivePropertyEntry {
                            component_type: component_desc.type_name.clone(),
                            property_name: property.name.clone(),
                            binding,
                            editor,
                        });
                    }
                }
            }

            let world = &mut context.world_mut().ecs_world;
            self.read_property_values(world);
        } else {
            let no_selection_id = ui_tree.create_node(
                "no_selection",
                Style::default(),
                UiNodeData::Text { content: "No selection".to_string() },
            );
            ui_tree.add_child(root_id, no_selection_id);
        }

        Ok(())
    }

    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint { position: PanelPosition::Right, preferred_size: Some((300.0, 600.0)), min_size: Some((200.0, 300.0)) }
    }
}
