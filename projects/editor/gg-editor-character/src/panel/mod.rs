//! 角色管理器面板实现
//! 提供角色的 CRUD 操作、表情映射管理、批量导入和骨骼绑定功能

use crate::{
    animation::{AnimationLoader, AnimationPreviewState, TimelineControl},
    skeleton::{SkeletonBinding, SkeletonHierarchy},
};
use gg_core::GResult;
use gg_ecs::Entity;
use gg_editor_shell::{EditorContext, EditorPanel, PanelLayoutHint, PanelPosition, event::EditorEvent};
use gg_galgame_schema::components::{CharacterDef, PortraitPosition};
use gg_ui::{FlexDirection, LayoutStyle, SizeValue, Style, UiNodeData, UiNodeId, UiTree};
use std::collections::HashMap;

/// 角色摘要信息
///
/// 缓存角色列表中的单条记录，用于 UI 显示。
pub struct CharacterInfo {
    /// 角色唯一标识
    pub id: String,
    /// 角色显示名称
    pub name: String,
    /// 表情映射数量
    pub expression_count: usize,
}

/// 角色创建向导
///
/// 收集创建角色所需的参数，提供验证和创建功能。
pub struct CharacterCreateWizard {
    /// 角色名称
    pub name: String,
    /// 默认立绘资源路径
    pub portrait_path: Option<String>,
    /// 默认位置锚点
    pub default_position: PortraitPosition,
}

impl CharacterCreateWizard {
    /// 创建新的角色创建向导
    ///
    /// 默认名称为空字符串，无立绘路径，位置为中央。
    pub fn new() -> Self {
        Self { name: String::new(), portrait_path: None, default_position: PortraitPosition::Center }
    }

    /// 创建带默认值的角色创建向导
    ///
    /// 使用指定名称和默认位置创建向导。
    pub fn with_defaults(name: String, default_position: PortraitPosition) -> Self {
        Self { name, portrait_path: None, default_position }
    }

    /// 验证向导参数
    ///
    /// 检查角色名称是否非空。
    /// 返回验证错误信息列表，空列表表示验证通过。
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push("角色名称不能为空".to_string());
        }
        errors
    }

    /// 根据向导参数创建角色定义
    ///
    /// 生成唯一 ID 并构建 CharacterDef 实例。
    /// 如果验证失败（名称为空），返回 None。
    pub fn create_character(&self) -> Option<CharacterDef> {
        if !self.validate().is_empty() {
            return None;
        }
        let new_id = format!("character_{}", uuid::Uuid::new_v4());
        Some(CharacterDef {
            id: new_id,
            name: self.name.clone(),
            default_portrait_path: self.portrait_path.clone(),
            expression_map: HashMap::new(),
            default_position: self.default_position.clone(),
            color: None,
        })
    }
}

impl Default for CharacterCreateWizard {
    fn default() -> Self {
        Self::new()
    }
}

/// 角色管理器面板
///
/// 提供角色的创建、删除、表情映射编辑、批量立绘导入和骨骼绑定功能。
/// 面板内部维护选中角色、预览表情状态和骨骼层级，便于 UI 交互。
pub struct CharacterManagerPanel {
    /// 面板是否可见
    visible: bool,
    /// 选中的角色 ID
    selected_character_id: Option<String>,
    /// 预览表情标签
    preview_expression: Option<String>,
    /// 批量导入路径
    import_path: Option<String>,
    /// 角色列表缓存
    characters_cache: Vec<CharacterInfo>,
    /// 角色创建向导
    create_wizard: CharacterCreateWizard,
    /// 骨骼层级映射（角色 ID -> 骨骼层级）
    skeleton_hierarchies: HashMap<String, SkeletonHierarchy>,
    /// 选中的骨骼节点 ID
    selected_skeleton_node: Option<String>,
    /// 动画预览状态
    animation_preview: AnimationPreviewState,
    /// 时间轴控制
    timeline_control: TimelineControl,
    /// 已加载的动画文件路径
    loaded_animation_path: Option<String>,
}

impl CharacterManagerPanel {
    /// 创建新的角色管理器面板
    pub fn new() -> Self {
        Self {
            visible: true,
            selected_character_id: None,
            preview_expression: None,
            import_path: None,
            characters_cache: Vec::new(),
            create_wizard: CharacterCreateWizard::new(),
            skeleton_hierarchies: HashMap::new(),
            selected_skeleton_node: None,
            animation_preview: AnimationPreviewState::new(),
            timeline_control: TimelineControl::new(600.0),
            loaded_animation_path: None,
        }
    }

    /// 刷新角色列表缓存
    ///
    /// 从 World 中查询所有 CharacterDef 组件，更新 characters_cache。
    pub fn refresh_character_list(&mut self, context: &mut EditorContext) -> GResult<()> {
        self.characters_cache.clear();
        let world = context.world();
        for &entity in world.ecs_world.entities().iter() {
            if let Some(char_def) = world.ecs_world.get_component::<CharacterDef>(entity) {
                self.characters_cache.push(CharacterInfo {
                    id: char_def.id.clone(),
                    name: char_def.name.clone(),
                    expression_count: char_def.expression_map.len(),
                });
            }
        }
        Ok(())
    }

    /// 创建新角色
    ///
    /// 在世界中生成新实体并添加 CharacterDef 组件，
    /// 默认位置为 PortraitPosition::Center，无表情映射。
    pub fn create_character(&mut self, context: &mut EditorContext) -> GResult<()> {
        let new_id = format!("character_{}", uuid::Uuid::new_v4());
        let character_def = CharacterDef {
            id: new_id.clone(),
            name: String::new(),
            default_portrait_path: None,
            expression_map: HashMap::new(),
            default_position: PortraitPosition::Center,
            color: None,
        };
        let entity = context.world_mut().ecs_world.spawn().id();
        context.world_mut().ecs_world.add_component(entity, character_def)?;
        self.selected_character_id = Some(new_id);
        Ok(())
    }

    /// 通过向导创建角色
    ///
    /// 使用 CharacterCreateWizard 中的参数创建角色。
    /// 验证通过后在世界中生成实体并添加 CharacterDef 组件，
    /// 同时为该角色创建空的骨骼层级。
    /// 返回创建的角色 ID，验证失败返回 None。
    pub fn create_character_from_wizard(&mut self, context: &mut EditorContext) -> Option<String> {
        let character_def = self.create_wizard.create_character()?;
        let character_id = character_def.id.clone();
        let entity = context.world_mut().ecs_world.spawn().id();
        if context.world_mut().ecs_world.add_component(entity, character_def).is_err() {
            return None;
        }
        self.skeleton_hierarchies.insert(character_id.clone(), SkeletonHierarchy::new());
        self.selected_character_id = Some(character_id.clone());
        Some(character_id)
    }

    /// 删除指定角色
    ///
    /// 根据 character_id 查找对应的实体并从世界中移除。
    /// 如果当前选中的角色被删除，将清除选中状态。
    pub fn delete_character(&mut self, character_id: &str, context: &mut EditorContext) -> GResult<()> {
        let world = context.world_mut();
        let target_entity = world
            .ecs_world
            .entities()
            .iter()
            .find(|&&entity| {
                world.ecs_world.get_component::<CharacterDef>(entity).map(|c| c.id == character_id).unwrap_or(false)
            })
            .copied();

        if let Some(entity) = target_entity {
            world.ecs_world.despawn(entity)?;
        }

        if self.selected_character_id.as_deref() == Some(character_id) {
            self.selected_character_id = None;
        }

        self.skeleton_hierarchies.remove(character_id);
        Ok(())
    }

    /// 添加表情映射
    ///
    /// 为指定角色添加一个表情标签到立绘资源路径的映射。
    /// 如果标签已存在则覆盖。
    pub fn add_expression(
        &mut self,
        character_id: &str,
        tag: String,
        asset_path: String,
        context: &mut EditorContext,
    ) -> GResult<()> {
        let world = context.world_mut();
        for &entity in world.ecs_world.entities().iter() {
            if let Some(char_def) = world.ecs_world.get_component_mut::<CharacterDef>(entity) {
                if char_def.id == character_id {
                    char_def.expression_map.insert(tag, asset_path);
                    break;
                }
            }
        }
        Ok(())
    }

    /// 移除表情映射
    ///
    /// 从指定角色的表情映射中移除给定标签的条目。
    pub fn remove_expression(&mut self, character_id: &str, tag: &str, context: &mut EditorContext) -> GResult<()> {
        let world = context.world_mut();
        for &entity in world.ecs_world.entities().iter() {
            if let Some(char_def) = world.ecs_world.get_component_mut::<CharacterDef>(entity) {
                if char_def.id == character_id {
                    char_def.expression_map.remove(tag);
                    break;
                }
            }
        }
        Ok(())
    }

    /// 设置角色默认位置
    ///
    /// 更新指定角色的 default_position 字段。
    pub fn set_default_position(
        &mut self,
        character_id: &str,
        position: PortraitPosition,
        context: &mut EditorContext,
    ) -> GResult<()> {
        let world = context.world_mut();
        for &entity in world.ecs_world.entities().iter() {
            if let Some(char_def) = world.ecs_world.get_component_mut::<CharacterDef>(entity) {
                if char_def.id == character_id {
                    char_def.default_position = position;
                    break;
                }
            }
        }
        Ok(())
    }

    /// 批量导入立绘
    ///
    /// 扫描指定目录中符合 characterid_expression.png 命名格式的文件，
    /// 自动创建角色定义和表情映射。
    /// 返回创建的角色 ID 列表。
    pub fn batch_import(&mut self, directory: &str, context: &mut EditorContext) -> GResult<Vec<String>> {
        let dir = std::path::Path::new(directory);
        if !dir.exists() || !dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut character_map: HashMap<String, HashMap<String, String>> = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

                if extension != "png" && extension != "jpg" && extension != "jpeg" {
                    continue;
                }

                let parts: Vec<&str> = file_stem.splitn(2, '_').collect();
                if parts.len() != 2 {
                    continue;
                }

                let char_id = parts[0].to_string();
                let expression_tag = parts[1].to_string();
                let asset_path = path.to_str().unwrap_or("").to_string();

                character_map.entry(char_id).or_default().insert(expression_tag, asset_path);
            }
        }

        let mut created_ids = Vec::new();
        let world = context.world_mut();

        for (char_id, expressions) in character_map {
            let character_def = CharacterDef {
                id: char_id.clone(),
                name: char_id.clone(),
                default_portrait_path: expressions.values().next().cloned(),
                expression_map: expressions,
                default_position: PortraitPosition::Center,
                color: None,
            };
            let entity = world.ecs_world.spawn().id();
            world.ecs_world.add_component(entity, character_def)?;
            created_ids.push(char_id);
        }

        Ok(created_ids)
    }

    /// 获取指定角色的骨骼层级可变引用
    ///
    /// 如果角色没有骨骼层级，则创建一个空的层级结构。
    pub fn get_or_create_skeleton_hierarchy(&mut self, character_id: &str) -> &mut SkeletonHierarchy {
        self.skeleton_hierarchies.entry(character_id.to_string()).or_insert_with(SkeletonHierarchy::new)
    }

    /// 向指定角色的骨骼层级添加节点
    ///
    /// 在指定父节点下添加新节点。如果父节点为 None，则作为根节点添加。
    /// 返回新节点的 ID。
    pub fn add_skeleton_node(&mut self, character_id: &str, name: String, parent_id: Option<String>) -> Option<String> {
        let hierarchy = self.get_or_create_skeleton_hierarchy(character_id);
        let node_id = hierarchy.add_node(name, parent_id);
        Some(node_id)
    }

    /// 从指定角色的骨骼层级移除节点
    ///
    /// 移除目标节点及其所有子节点，同时移除关联的骨骼绑定。
    pub fn remove_skeleton_node(&mut self, character_id: &str, node_id: &str) {
        if let Some(hierarchy) = self.skeleton_hierarchies.get_mut(character_id) {
            hierarchy.remove_node(node_id);
        }
        if self.selected_skeleton_node.as_deref() == Some(node_id) {
            self.selected_skeleton_node = None;
        }
    }

    /// 重命名指定角色的骨骼节点
    pub fn rename_skeleton_node(&mut self, character_id: &str, node_id: &str, new_name: String) {
        if let Some(hierarchy) = self.skeleton_hierarchies.get_mut(character_id) {
            hierarchy.rename_node(node_id, new_name);
        }
    }

    /// 为指定角色的骨骼节点添加绑定
    pub fn add_skeleton_binding(&mut self, character_id: &str, binding: SkeletonBinding) {
        let hierarchy = self.get_or_create_skeleton_hierarchy(character_id);
        hierarchy.add_binding(binding);
    }

    /// 移除指定角色骨骼节点的绑定
    pub fn remove_skeleton_binding(&mut self, character_id: &str, node_id: &str) {
        if let Some(hierarchy) = self.skeleton_hierarchies.get_mut(character_id) {
            hierarchy.remove_binding(node_id);
        }
    }

    /// 获取指定角色选中骨骼节点的绑定信息
    pub fn get_selected_node_binding(&self, character_id: &str) -> Option<&SkeletonBinding> {
        let hierarchy = self.skeleton_hierarchies.get(character_id)?;
        let node_id = self.selected_skeleton_node.as_deref()?;
        hierarchy.bindings.iter().find(|b| b.node_id == node_id)
    }

    /// 选中角色并发布选中事件
    ///
    /// 设置选中角色 ID，并通过事件总线发布 EntitySelected 事件，
    /// 以便 Inspector 等其他面板响应角色选择。
    pub fn select_character(&mut self, character_id: String, context: &mut EditorContext) {
        self.selected_character_id = Some(character_id.clone());
        self.selected_skeleton_node = None;
        self.preview_expression = None;

        let world = context.world();
        for &entity in world.ecs_world.entities().iter() {
            if let Some(char_def) = world.ecs_world.get_component::<CharacterDef>(entity) {
                if char_def.id == character_id {
                    context.events_mut().publish(EditorEvent::EntitySelected { entity: entity as u64 });
                    break;
                }
            }
        }
    }

    /// 获取选中角色的属性数据
    ///
    /// 返回选中角色的 (id, name, position) 元组，用于 Inspector 面板显示。
    pub fn get_selected_character_properties(&self, context: &EditorContext) -> Option<(String, String, PortraitPosition)> {
        let selected_id = self.selected_character_id.as_deref()?;
        let world = context.world();
        for &entity in world.ecs_world.entities().iter() {
            if let Some(char_def) = world.ecs_world.get_component::<CharacterDef>(entity) {
                if char_def.id == selected_id {
                    return Some((char_def.id.clone(), char_def.name.clone(), char_def.default_position.clone()));
                }
            }
        }
        None
    }

    /// 加载动画文件并开始预览
    ///
    /// 从指定路径加载 .animation 文件，解析轨道和关键帧数据。
    pub fn load_animation(&mut self, path: &str) -> GResult<()> {
        let state = AnimationLoader::load_from_file(path)?;
        self.animation_preview = state;
        self.loaded_animation_path = Some(path.to_string());
        Ok(())
    }

    /// 播放动画预览
    pub fn animation_play(&mut self) {
        self.animation_preview.play();
    }

    /// 暂停动画预览
    pub fn animation_pause(&mut self) {
        self.animation_preview.pause();
    }

    /// 停止动画预览
    pub fn animation_stop(&mut self) {
        self.animation_preview.stop();
    }

    /// 设置动画播放速度
    ///
    /// 速度范围限制在 0.1x ~ 3.0x 之间。
    pub fn animation_set_speed(&mut self, speed: f32) {
        self.animation_preview.set_speed(speed);
    }

    /// 切换动画循环播放
    pub fn animation_toggle_loop(&mut self) {
        self.animation_preview.toggle_loop();
    }

    /// 跳转到动画指定时间位置
    pub fn animation_seek(&mut self, position_secs: f32) {
        self.animation_preview.seek(position_secs);
    }

    /// 推进动画一帧
    ///
    /// 根据经过的时间更新动画播放位置。
    pub fn animation_tick(&mut self, delta_secs: f32) {
        self.animation_preview.tick(delta_secs);
    }

    /// 构建骨骼绑定面板 UI 子树
    ///
    /// 为选中角色构建骨骼层级 TreeView 和绑定信息显示。
    fn build_skeleton_binding_ui(&self, ui_tree: &mut UiTree, parent_id: UiNodeId, character_id: &str) {
        let skeleton_section_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Auto,
                padding: 4.0,
                gap: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let skeleton_section_id = ui_tree.create_node("skeleton_section", skeleton_section_style, UiNodeData::Container);
        ui_tree.add_child(parent_id, skeleton_section_id);

        let skeleton_title_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let skeleton_title_id =
            ui_tree.create_node("skeleton_title", skeleton_title_style, UiNodeData::Text { content: "骨骼绑定".to_string() });
        ui_tree.add_child(skeleton_section_id, skeleton_title_id);

        if let Some(hierarchy) = self.skeleton_hierarchies.get(character_id) {
            Self::build_skeleton_tree_nodes(ui_tree, skeleton_section_id, hierarchy, hierarchy.root_id.as_deref(), 0);
        }

        if let Some(ref node_id) = self.selected_skeleton_node {
            if let Some(hierarchy) = self.skeleton_hierarchies.get(character_id) {
                if let Some(binding) = hierarchy.bindings.iter().find(|b| b.node_id == *node_id) {
                    let binding_style = Style {
                        layout: LayoutStyle {
                            width: SizeValue::Percent(1.0),
                            height: SizeValue::Px(20.0),
                            padding: 2.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let binding_text = format!(
                        "绑定: {} -> {} (偏移: [{:.1}, {:.1}])",
                        binding.node_id, binding.region_name, binding.offset[0], binding.offset[1]
                    );
                    let binding_id =
                        ui_tree.create_node("skeleton_binding_info", binding_style, UiNodeData::Text { content: binding_text });
                    ui_tree.add_child(skeleton_section_id, binding_id);
                }
            }
        }
    }

    /// 递归构建骨骼树节点 UI
    fn build_skeleton_tree_nodes(
        ui_tree: &mut UiTree,
        parent_id: UiNodeId,
        hierarchy: &SkeletonHierarchy,
        node_id: Option<&str>,
        depth: usize,
    ) {
        let Some(nid) = node_id
        else {
            return;
        };
        let Some(node) = hierarchy.get_node(nid)
        else {
            return;
        };

        let indent = "  ".repeat(depth);
        let prefix = if node.children.is_empty() { "·" } else { "▸" };
        let display = format!("{}{} {}", indent, prefix, node.name);

        let node_style = Style {
            layout: LayoutStyle {
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(20.0),
                padding: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let ui_node_id =
            ui_tree.create_node(&format!("sk_node_{}", node.id), node_style, UiNodeData::Text { content: display });
        ui_tree.add_child(parent_id, ui_node_id);

        for child_id in &node.children {
            Self::build_skeleton_tree_nodes(ui_tree, parent_id, hierarchy, Some(child_id.as_str()), depth + 1);
        }
    }
}

impl EditorPanel for CharacterManagerPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Character Manager"
    }

    /// 获取面板可见性
    fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置面板可见性
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 构建角色管理器面板 UI 节点树
    ///
    /// 使用 SplitView 布局，左侧为角色列表面板（搜索框、创建/删除按钮、角色树），
    /// 右侧为角色详情面板（立绘预览、表情列表、骨骼绑定区域）。
    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> Option<UiNodeId> {
        let root_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Row,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Percent(1.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let root_id = ui_tree.create_node("char_root", root_style, UiNodeData::Container);

        let left_panel_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(0.3),
                height: SizeValue::Percent(1.0),
                padding: 4.0,
                gap: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let left_panel_id = ui_tree.create_node("left_panel", left_panel_style, UiNodeData::Container);
        ui_tree.add_child(root_id, left_panel_id);

        let search_style = Style {
            layout: LayoutStyle {
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(24.0),
                padding: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let search_id = ui_tree.create_node("char_search", search_style, UiNodeData::Custom { kind: "Input".to_string() });
        ui_tree.add_child(left_panel_id, search_id);

        let button_row_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Row,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(28.0),
                gap: 4.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let button_row_id = ui_tree.create_node("button_row", button_row_style, UiNodeData::Container);
        ui_tree.add_child(left_panel_id, button_row_id);

        let create_btn_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(0.5), height: SizeValue::Px(24.0), ..Default::default() },
            ..Default::default()
        };
        let create_btn_id =
            ui_tree.create_node("create_btn", create_btn_style, UiNodeData::Text { content: "创建角色".to_string() });
        ui_tree.add_child(button_row_id, create_btn_id);

        let delete_btn_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(0.5), height: SizeValue::Px(24.0), ..Default::default() },
            ..Default::default()
        };
        let delete_btn_id =
            ui_tree.create_node("delete_btn", delete_btn_style, UiNodeData::Text { content: "删除角色".to_string() });
        ui_tree.add_child(button_row_id, delete_btn_id);

        let list_title_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let list_title_id =
            ui_tree.create_node("char_list_title", list_title_style, UiNodeData::Text { content: "角色列表".to_string() });
        ui_tree.add_child(left_panel_id, list_title_id);

        let scroll_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Percent(1.0),
                ..Default::default()
            },
            overflow: gg_ui::Overflow::Clip,
            ..Default::default()
        };
        let scroll_id = ui_tree.create_node("char_scroll", scroll_style, UiNodeData::Container);
        ui_tree.add_child(left_panel_id, scroll_id);

        for char_info in &self.characters_cache {
            let item_style = Style {
                layout: LayoutStyle {
                    width: SizeValue::Percent(1.0),
                    height: SizeValue::Px(24.0),
                    padding: 2.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let is_selected = self.selected_character_id.as_deref() == Some(&char_info.id);
            let display = if is_selected {
                format!("▸ {} ({})", char_info.name, char_info.id)
            }
            else {
                format!("  {} ({})", char_info.name, char_info.id)
            };
            let item_id =
                ui_tree.create_node(&format!("char_item_{}", char_info.id), item_style, UiNodeData::Text { content: display });
            ui_tree.add_child(scroll_id, item_id);
        }

        let right_panel_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(0.7),
                height: SizeValue::Percent(1.0),
                padding: 4.0,
                gap: 4.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let right_panel_id = ui_tree.create_node("right_panel", right_panel_style, UiNodeData::Container);
        ui_tree.add_child(root_id, right_panel_id);

        let portrait_section_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Auto,
                padding: 4.0,
                gap: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let portrait_section_id = ui_tree.create_node("portrait_section", portrait_section_style, UiNodeData::Container);
        ui_tree.add_child(right_panel_id, portrait_section_id);

        let portrait_title_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let portrait_title_id =
            ui_tree.create_node("portrait_title", portrait_title_style, UiNodeData::Text { content: "立绘预览".to_string() });
        ui_tree.add_child(portrait_section_id, portrait_title_id);

        let portrait_preview_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(200.0), ..Default::default() },
            ..Default::default()
        };
        let portrait_preview_id =
            ui_tree.create_node("portrait_preview", portrait_preview_style, UiNodeData::Custom { kind: "Image".to_string() });
        ui_tree.add_child(portrait_section_id, portrait_preview_id);

        let expr_section_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Auto,
                padding: 4.0,
                gap: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let expr_section_id = ui_tree.create_node("expr_section", expr_section_style, UiNodeData::Container);
        ui_tree.add_child(right_panel_id, expr_section_id);

        let expr_title_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let expr_title_id =
            ui_tree.create_node("expr_title", expr_title_style, UiNodeData::Text { content: "表情列表".to_string() });
        ui_tree.add_child(expr_section_id, expr_title_id);

        let expr_scroll_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(120.0),
                ..Default::default()
            },
            overflow: gg_ui::Overflow::Clip,
            ..Default::default()
        };
        let expr_scroll_id = ui_tree.create_node("expr_scroll", expr_scroll_style, UiNodeData::Container);
        ui_tree.add_child(expr_section_id, expr_scroll_id);

        if let Some(ref selected_id) = self.selected_character_id {
            let selected_style = Style {
                layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(20.0), ..Default::default() },
                ..Default::default()
            };
            let selected_id_node = ui_tree.create_node(
                "selected_char_id",
                selected_style,
                UiNodeData::Text { content: format!("选中: {}", selected_id) },
            );
            ui_tree.add_child(expr_scroll_id, selected_id_node);
        }

        if let Some(ref character_id) = self.selected_character_id {
            self.build_skeleton_binding_ui(ui_tree, right_panel_id, character_id);
        }

        let anim_section_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Auto,
                padding: 4.0,
                gap: 2.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let anim_section_id = ui_tree.create_node("anim_section", anim_section_style, UiNodeData::Container);
        ui_tree.add_child(right_panel_id, anim_section_id);

        let anim_title_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let anim_name = self.animation_preview.animation_name.as_deref().unwrap_or("未加载动画");
        let anim_title_id = ui_tree.create_node(
            "anim_title",
            anim_title_style,
            UiNodeData::Text { content: format!("动画预览: {}", anim_name) },
        );
        ui_tree.add_child(anim_section_id, anim_title_id);

        let anim_toolbar_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Row,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(28.0),
                gap: 4.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let anim_toolbar_id = ui_tree.create_node("anim_toolbar", anim_toolbar_style, UiNodeData::Container);
        ui_tree.add_child(anim_section_id, anim_toolbar_id);

        let toolbar_items = self.animation_preview.build_toolbar_ui();
        for item in &toolbar_items {
            let label = match item.kind {
                crate::animation::AnimationToolbarKind::Play => "▶ 播放",
                crate::animation::AnimationToolbarKind::Pause => "⏸ 暂停",
                crate::animation::AnimationToolbarKind::Stop => "⏹ 停止",
                crate::animation::AnimationToolbarKind::LoopToggle => {
                    if item.active {
                        "🔁 循环:开"
                    }
                    else {
                        "🔁 循环:关"
                    }
                }
            };
            let btn_style = Style {
                layout: LayoutStyle { width: SizeValue::Auto, height: SizeValue::Px(24.0), padding: 4.0, ..Default::default() },
                ..Default::default()
            };
            let btn_id = ui_tree.create_node(
                &format!("anim_btn_{:?}", item.kind),
                btn_style,
                UiNodeData::Custom { kind: format!("AnimBtn:{:?}", item.kind) },
            );
            ui_tree.add_child(anim_toolbar_id, btn_id);
        }

        let speed_label_style = Style {
            layout: LayoutStyle { width: SizeValue::Auto, height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let speed_label_id = ui_tree.create_node(
            "speed_label",
            speed_label_style,
            UiNodeData::Text { content: format!("速度: {:.1}x", self.animation_preview.speed) },
        );
        ui_tree.add_child(anim_toolbar_id, speed_label_id);

        let speed_slider_style = Style {
            layout: LayoutStyle { width: SizeValue::Px(100.0), height: SizeValue::Px(20.0), ..Default::default() },
            ..Default::default()
        };
        let speed_slider_id =
            ui_tree.create_node("speed_slider", speed_slider_style, UiNodeData::Custom { kind: "Slider:Speed".to_string() });
        ui_tree.add_child(anim_toolbar_id, speed_slider_id);

        let timeline_style = Style {
            layout: LayoutStyle {
                direction: FlexDirection::Column,
                width: SizeValue::Percent(1.0),
                height: SizeValue::Px(40.0),
                ..Default::default()
            },
            ..Default::default()
        };
        let timeline_id =
            ui_tree.create_node("anim_timeline", timeline_style, UiNodeData::Custom { kind: "Timeline".to_string() });
        ui_tree.add_child(anim_section_id, timeline_id);

        let keyframe_times = self.animation_preview.all_keyframe_times();
        if !keyframe_times.is_empty() {
            let kf_style = Style {
                layout: LayoutStyle {
                    direction: FlexDirection::Row,
                    width: SizeValue::Percent(1.0),
                    height: SizeValue::Px(16.0),
                    ..Default::default()
                },
                ..Default::default()
            };
            let kf_row_id = ui_tree.create_node("kf_row", kf_style, UiNodeData::Container);
            ui_tree.add_child(timeline_id, kf_row_id);

            for (i, &time) in keyframe_times.iter().enumerate() {
                let kf_marker_style = Style {
                    layout: LayoutStyle { width: SizeValue::Px(6.0), height: SizeValue::Px(12.0), ..Default::default() },
                    ..Default::default()
                };
                let kf_marker_id = ui_tree.create_node(
                    &format!("kf_{}", i),
                    kf_marker_style,
                    UiNodeData::Custom { kind: "KeyframeMarker".to_string() },
                );
                ui_tree.add_child(kf_row_id, kf_marker_id);
            }
        }

        let pos_text_style = Style {
            layout: LayoutStyle { width: SizeValue::Percent(1.0), height: SizeValue::Px(16.0), ..Default::default() },
            ..Default::default()
        };
        let pos_text =
            format!("位置: {:.2}s / {:.2}s", self.animation_preview.timeline.position, self.animation_preview.duration);
        let pos_text_id = ui_tree.create_node("anim_pos_text", pos_text_style, UiNodeData::Text { content: pos_text });
        ui_tree.add_child(timeline_id, pos_text_id);

        Some(root_id)
    }

    /// 获取面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint { position: PanelPosition::Right, preferred_size: Some((300.0, 400.0)), min_size: Some((250.0, 300.0)) }
    }

    /// 处理编辑器事件
    fn on_event(&mut self, event: &EditorEvent, context: &mut EditorContext) {
        match event {
            EditorEvent::FileChanged { .. } => {
                let _ = self.refresh_character_list(context);
            }
            EditorEvent::MouseDown { .. } => {}
            _ => {}
        }
    }
}
