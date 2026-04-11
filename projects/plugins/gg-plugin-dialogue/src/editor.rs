//! 对话编辑器面板实现
//! 提供对话节点的列表浏览、选中查看和属性编辑功能

use gg_core::GResult;
use gg_ecs::World;
use gg_editor_shell::context::EditorContext;
use gg_editor_shell::panel::{EditorPanel, PanelLayoutHint, PanelPosition};
use gg_ui::node::UiNodeData;
use gg_ui::style::{FlexDirection, FontStyle, LayoutStyle, Style};
use gg_ui::UiTree;

use crate::schema::DialogueNode;

/// 对话编辑器面板
///
/// 提供对话节点的列表浏览和属性编辑功能，
/// 支持从 World 中刷新节点列表、选中节点查看详情。
pub struct DialogueEditorPanel {
    /// 面板是否可见
    visible: bool,
    /// 当前选中的节点 ID
    selected_node_id: Option<String>,
    /// 对话节点 ID 列表
    node_ids: Vec<String>,
}

impl DialogueEditorPanel {
    /// 创建新的对话编辑器面板
    pub fn new() -> Self {
        Self { visible: true, selected_node_id: None, node_ids: Vec::new() }
    }

    /// 从 World 中刷新对话节点 ID 列表
    ///
    /// 查询所有拥有 `DialogueNode` 组件的实体，提取其 `id` 字段存储到内部列表。
    pub fn refresh_nodes(&mut self, world: &World) {
        self.node_ids.clear();
        for (_entity, node) in world.query::<DialogueNode>() {
            self.node_ids.push(node.id.clone());
        }
    }

    /// 获取当前选中的节点 ID
    pub fn selected_node_id(&self) -> Option<&str> {
        self.selected_node_id.as_deref()
    }

    /// 设置当前选中的节点 ID
    pub fn set_selected_node_id(&mut self, id: Option<String>) {
        self.selected_node_id = id;
    }
}

impl Default for DialogueEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for DialogueEditorPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Dialogue Editor"
    }

    /// 获取面板可见性
    fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置面板可见性
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 构建对话编辑器面板 UI 节点树
    ///
    /// 创建包含标题、节点列表和属性编辑区域的面板布局。
    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        let root_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0).with_gap(4.0));

        let root_id = ui_tree.create_node("dialogue_editor_root", root_style, UiNodeData::Container);
        ui_tree.set_root(root_id);

        let title_style = Style::new().with_font(FontStyle::new().with_size(18.0));
        let title_id = ui_tree.create_node(
            "dialogue_title",
            title_style,
            UiNodeData::Text { content: "Dialogue Nodes".to_string() },
        );
        ui_tree.add_child(root_id, title_id);

        let list_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0));
        let list_id = ui_tree.create_node("dialogue_node_list", list_style, UiNodeData::Container);
        ui_tree.add_child(root_id, list_id);

        for node_id in &self.node_ids {
            let item_style = Style::new()
                .with_layout(LayoutStyle::new().with_padding(4.0))
                .with_font(FontStyle::new().with_size(14.0));
            let item_id = ui_tree.create_node(
                format!("node_item_{}", node_id),
                item_style,
                UiNodeData::Text { content: node_id.clone() },
            );
            ui_tree.add_child(list_id, item_id);
        }

        if let Some(selected_id) = &self.selected_node_id {
            let section_style = Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(4.0));
            let section_id = ui_tree.create_node("dialogue_properties", section_style, UiNodeData::Container);
            ui_tree.add_child(root_id, section_id);

            let header_style = Style::new().with_font(FontStyle::new().with_size(16.0));
            let header_id = ui_tree.create_node(
                "properties_header",
                header_style,
                UiNodeData::Text { content: format!("Selected: {}", selected_id) },
            );
            ui_tree.add_child(section_id, header_id);

            let prop_style = Style::new().with_font(FontStyle::new().with_size(13.0));

            let text_label_id = ui_tree.create_node(
                "prop_text_label",
                prop_style.clone(),
                UiNodeData::Text { content: "Text: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, text_label_id);

            let speaker_label_id = ui_tree.create_node(
                "prop_speaker_label",
                prop_style.clone(),
                UiNodeData::Text { content: "Speaker: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, speaker_label_id);

            let commands_label_id = ui_tree.create_node(
                "prop_commands_label",
                prop_style.clone(),
                UiNodeData::Text { content: "Commands: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, commands_label_id);

            let choices_label_id = ui_tree.create_node(
                "prop_choices_label",
                prop_style,
                UiNodeData::Text { content: "Choices: (edit in inspector)".to_string() },
            );
            ui_tree.add_child(section_id, choices_label_id);
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
