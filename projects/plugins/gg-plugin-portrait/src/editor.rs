//! 立绘编辑器面板实现
//! 提供立绘条目的列表浏览、选中查看和属性编辑功能

use gg_core::GResult;
use gg_ecs::World;
use gg_editor_shell::context::EditorContext;
use gg_editor_shell::panel::{EditorPanel, PanelLayoutHint, PanelPosition};
use gg_galgame_schema::components::{PortraitPosition, PortraitState};
use gg_ui::node::UiNodeData;
use gg_ui::style::{FlexDirection, FontStyle, LayoutStyle, Style};
use gg_ui::UiTree;

/// 将立绘位置格式化为可读字符串
fn format_position(pos: &PortraitPosition) -> String {
    match pos {
        PortraitPosition::Left => "Left".to_string(),
        PortraitPosition::Center => "Center".to_string(),
        PortraitPosition::Right => "Right".to_string(),
        PortraitPosition::Custom { x, y } => format!("Custom({}, {})", x, y),
    }
}

/// 立绘编辑器面板
///
/// 提供立绘条目的列表浏览和属性编辑功能，
/// 支持从 World 中刷新立绘列表、选中条目查看详情、添加和移除立绘。
pub struct PortraitEditorPanel {
    /// 面板是否可见
    visible: bool,
    /// 当前选中的实体 ID
    selected_entity: Option<u64>,
    /// 立绘条目列表：(实体 ID, 角色 ID, 位置描述)
    portrait_entries: Vec<(u64, String, String)>,
}

impl PortraitEditorPanel {
    /// 创建新的立绘编辑器面板
    pub fn new() -> Self {
        Self { visible: true, selected_entity: None, portrait_entries: Vec::new() }
    }

    /// 从 World 中刷新立绘条目列表
    ///
    /// 查询所有拥有 `PortraitState` 组件的实体，提取实体 ID、角色 ID 和位置信息存储到内部列表。
    pub fn refresh_portraits(&mut self, world: &World) {
        self.portrait_entries.clear();
        for (entity, state) in world.query::<PortraitState>() {
            let entity_id = ((entity.generation() as u64) << 32) | (entity.index() as u64);
            let position_str = format_position(&state.position);
            self.portrait_entries.push((entity_id, state.character_id.clone(), position_str));
        }
    }

    /// 获取当前选中的实体 ID
    pub fn selected_entity(&self) -> Option<u64> {
        self.selected_entity
    }

    /// 设置当前选中的实体 ID
    pub fn set_selected_entity(&mut self, entity: Option<u64>) {
        self.selected_entity = entity;
    }
}

impl Default for PortraitEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for PortraitEditorPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Portrait Editor"
    }

    /// 获取面板可见性
    fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置面板可见性
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 构建立绘编辑器面板 UI 节点树
    ///
    /// 创建包含标题、立绘列表、属性编辑区域和操作按钮的面板布局。
    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        let root_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0).with_gap(4.0));

        let root_id = ui_tree.create_node("portrait_editor_root", root_style, UiNodeData::Container);
        ui_tree.set_root(root_id);

        let title_style = Style::new().with_font(FontStyle::new().with_size(18.0));
        let title_id = ui_tree.create_node(
            "portrait_title",
            title_style,
            UiNodeData::Text { content: "Portraits".to_string() },
        );
        ui_tree.add_child(root_id, title_id);

        let list_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0));
        let list_id = ui_tree.create_node("portrait_entry_list", list_style, UiNodeData::Container);
        ui_tree.add_child(root_id, list_id);

        for (entity_id, character_id, position) in &self.portrait_entries {
            let item_style = Style::new()
                .with_layout(LayoutStyle::new().with_padding(4.0))
                .with_font(FontStyle::new().with_size(14.0));
            let label = format!("{} @ {}", character_id, position);
            let item_id = ui_tree.create_node(
                format!("portrait_item_{}", entity_id),
                item_style,
                UiNodeData::Text { content: label },
            );
            ui_tree.add_child(list_id, item_id);
        }

        let button_row_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(4.0));
        let button_row_id = ui_tree.create_node("portrait_buttons", button_row_style, UiNodeData::Container);
        ui_tree.add_child(root_id, button_row_id);

        let add_btn_style = Style::new().with_font(FontStyle::new().with_size(13.0));
        let add_btn_id = ui_tree.create_node(
            "btn_add_portrait",
            add_btn_style,
            UiNodeData::Custom { kind: "button_add".to_string() },
        );
        ui_tree.add_child(button_row_id, add_btn_id);

        let remove_btn_style = Style::new().with_font(FontStyle::new().with_size(13.0));
        let remove_btn_id = ui_tree.create_node(
            "btn_remove_portrait",
            remove_btn_style,
            UiNodeData::Custom { kind: "button_remove".to_string() },
        );
        ui_tree.add_child(button_row_id, remove_btn_id);

        if let Some(selected_id) = self.selected_entity {
            let section_style = Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(4.0));
            let section_id = ui_tree.create_node("portrait_properties", section_style, UiNodeData::Container);
            ui_tree.add_child(root_id, section_id);

            let header_style = Style::new().with_font(FontStyle::new().with_size(16.0));
            let header_id = ui_tree.create_node(
                "properties_header",
                header_style,
                UiNodeData::Text { content: format!("Selected: Entity {}", selected_id) },
            );
            ui_tree.add_child(section_id, header_id);

            let prop_style = Style::new().with_font(FontStyle::new().with_size(13.0));

            let position_label_id = ui_tree.create_node(
                "prop_position_label",
                prop_style.clone(),
                UiNodeData::Text { content: "Position: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, position_label_id);

            let expression_label_id = ui_tree.create_node(
                "prop_expression_label",
                prop_style.clone(),
                UiNodeData::Text { content: "Expression: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, expression_label_id);

            let scale_label_id = ui_tree.create_node(
                "prop_scale_label",
                prop_style.clone(),
                UiNodeData::Text { content: "Scale: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, scale_label_id);

            let opacity_label_id = ui_tree.create_node(
                "prop_opacity_label",
                prop_style,
                UiNodeData::Text { content: "Opacity: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, opacity_label_id);
        }

        Ok(())
    }

    /// 获取面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Right,
            preferred_size: Some((350.0, 600.0)),
            min_size: None,
        }
    }
}
