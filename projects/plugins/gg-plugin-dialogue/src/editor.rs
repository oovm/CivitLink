//! 对话编辑器面板实现
//! 提供对话节点的列表浏览、选中查看、属性编辑和对话预览功能

use gg_core::GResult;
use gg_ecs::World;
use gg_editor_shell::context::EditorContext;
use gg_editor_shell::panel::{EditorPanel, PanelLayoutHint, PanelPosition};
use gg_ui::node::UiNodeData;
use gg_ui::style::{FlexDirection, FontStyle, LayoutStyle, Style};
use gg_ui::UiTree;

use crate::schema::{Choice, DialogueCommand, DialogueNode, DialogueScript};

/// 对话编辑状态
///
/// 跟踪当前正在编辑的节点属性，包括说话者、文本、命令和选项。
/// 编辑操作先修改此状态，确认后再写回对话节点。
pub struct DialogueEditState {
    /// 正在编辑的说话者 ID
    editing_speaker_id: Option<String>,
    /// 正在编辑的文本内容
    editing_text: String,
    /// 正在编辑的命令列表
    editing_commands: Vec<DialogueCommand>,
    /// 正在编辑的选项列表
    editing_choices: Vec<Choice>,
}

impl Default for DialogueEditState {
    fn default() -> Self {
        Self {
            editing_speaker_id: None,
            editing_text: String::new(),
            editing_commands: Vec::new(),
            editing_choices: Vec::new(),
        }
    }
}

/// 对话预览状态
///
/// 跟踪对话预览的运行时状态，支持打字机效果逐字显示文本、
/// 选项选择和节点推进。
pub struct DialoguePreviewState {
    /// 当前预览的节点 ID
    current_node_id: String,
    /// 当前显示的文本（逐字显示用）
    displayed_text: String,
    /// 完整文本
    full_text: String,
    /// 说话者名称
    speaker_name: Option<String>,
    /// 当前显示的选项
    current_choices: Vec<Choice>,
    /// 是否等待用户点击推进
    waiting_for_advance: bool,
    /// 打字机当前字符索引
    typewriter_index: usize,
}

/// 对话编辑器面板
///
/// 提供对话节点的列表浏览、属性编辑和对话预览功能，
/// 支持从 World 中刷新节点列表、加载对话脚本、编辑节点属性和预览对话流程。
pub struct DialogueEditorPanel {
    /// 面板是否可见
    visible: bool,
    /// 当前编辑的对话脚本
    current_script: Option<DialogueScript>,
    /// 当前选中的节点 ID
    selected_node_id: Option<String>,
    /// 对话节点 ID 列表
    node_ids: Vec<String>,
    /// 编辑状态
    edit_state: DialogueEditState,
    /// 预览状态
    preview_state: Option<DialoguePreviewState>,
}

impl DialogueEditorPanel {
    /// 创建新的对话编辑器面板
    pub fn new() -> Self {
        Self {
            visible: true,
            current_script: None,
            selected_node_id: None,
            node_ids: Vec::new(),
            edit_state: DialogueEditState::default(),
            preview_state: None,
        }
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

    /// 加载对话脚本用于编辑
    ///
    /// 加载脚本后自动提取所有节点 ID 到内部列表，
    /// 并重置选中状态和编辑状态。
    pub fn load_script(&mut self, script: DialogueScript) {
        self.node_ids = script.nodes.iter().map(|n| n.id.clone()).collect();
        self.selected_node_id = None;
        self.edit_state = DialogueEditState::default();
        self.preview_state = None;
        self.current_script = Some(script);
    }

    /// 选中指定节点并填充编辑状态
    ///
    /// 从当前脚本中查找指定 ID 的节点，将其属性复制到编辑状态中。
    /// 如果未找到对应节点，则清除选中状态。
    pub fn select_node(&mut self, node_id: &str) {
        if let Some(node) = self.selected_node() {
            if node.id == node_id {
                return;
            }
        }

        let script = match &self.current_script {
            Some(s) => s,
            None => {
                self.selected_node_id = None;
                return;
            }
        };

        let found = script.nodes.iter().find(|n| n.id == node_id);
        match found {
            Some(node) => {
                self.selected_node_id = Some(node_id.to_string());
                self.edit_state.editing_speaker_id = node.speaker_id.clone();
                self.edit_state.editing_text = node.text.clone();
                self.edit_state.editing_commands = node.commands.clone();
                self.edit_state.editing_choices = node.choices.clone();
            }
            None => {
                self.selected_node_id = None;
            }
        }
    }

    /// 向编辑状态的命令列表末尾添加一条命令
    ///
    /// 添加后需调用 `apply_edit` 将变更写回节点。
    pub fn add_command(&mut self, command: DialogueCommand) {
        self.edit_state.editing_commands.push(command);
    }

    /// 从编辑状态的命令列表中移除指定索引的命令
    ///
    /// 如果索引越界则不做任何操作。
    /// 移除后需调用 `apply_edit` 将变更写回节点。
    pub fn remove_command(&mut self, index: usize) {
        if index < self.edit_state.editing_commands.len() {
            self.edit_state.editing_commands.remove(index);
        }
    }

    /// 向编辑状态的选项列表末尾添加一个选项
    ///
    /// 添加后需调用 `apply_edit` 将变更写回节点。
    pub fn add_choice(&mut self, choice: Choice) {
        self.edit_state.editing_choices.push(choice);
    }

    /// 从编辑状态的选项列表中移除指定索引的选项
    ///
    /// 如果索引越界则不做任何操作。
    /// 移除后需调用 `apply_edit` 将变更写回节点。
    pub fn remove_choice(&mut self, index: usize) {
        if index < self.edit_state.editing_choices.len() {
            self.edit_state.editing_choices.remove(index);
        }
    }

    /// 将当前编辑状态应用回选中的节点
    ///
    /// 将编辑状态中的说话者、文本、命令和选项写回到脚本中对应的节点。
    /// 如果没有选中节点或没有加载脚本，则不做任何操作。
    pub fn apply_edit(&mut self) {
        let selected_id = match &self.selected_node_id {
            Some(id) => id.clone(),
            None => return,
        };

        let speaker_id = self.edit_state.editing_speaker_id.clone();
        let text = self.edit_state.editing_text.clone();
        let commands = self.edit_state.editing_commands.clone();
        let choices = self.edit_state.editing_choices.clone();

        if let Some(node) = self.selected_node_mut() {
            node.speaker_id = speaker_id;
            node.text = text;
            node.commands = commands;
            node.choices = choices;
        }

        let _ = selected_id;
    }

    /// 从第一个节点开始对话预览
    ///
    /// 如果当前脚本没有节点，则不会启动预览。
    /// 启动后打字机效果从第一个字符开始逐字显示。
    pub fn start_preview(&mut self) {
        let script = match &self.current_script {
            Some(s) => s,
            None => return,
        };

        let first_node = match script.nodes.first() {
            Some(n) => n,
            None => return,
        };

        let speaker_name = first_node
            .speaker_id
            .as_ref()
            .and_then(|sid| script.characters.iter().find(|c| &c.id == sid))
            .map(|c| c.name.clone());

        self.preview_state = Some(DialoguePreviewState {
            current_node_id: first_node.id.clone(),
            displayed_text: String::new(),
            full_text: first_node.text.clone(),
            speaker_name,
            current_choices: first_node.choices.clone(),
            waiting_for_advance: false,
            typewriter_index: 0,
        });
    }

    /// 推进对话预览
    ///
    /// 如果打字机效果尚未完成，则显示下一个字符；
    /// 如果文本已全部显示且有选项，则等待用户选择；
    /// 如果文本已全部显示且无选项，则推进到下一个节点。
    pub fn advance_preview(&mut self) {
        let preview = match &mut self.preview_state {
            Some(p) => p,
            None => return,
        };

        if preview.typewriter_index < preview.full_text.len() {
            let next_end = preview.full_text.len().min(preview.typewriter_index + 1);
            preview.displayed_text = preview.full_text[..next_end].to_string();
            preview.typewriter_index = next_end;

            if preview.typewriter_index >= preview.full_text.len() {
                preview.waiting_for_advance = preview.current_choices.is_empty();
            }
            return;
        }

        if !preview.current_choices.is_empty() {
            return;
        }

        let script = match &self.current_script {
            Some(s) => s,
            None => return,
        };

        let current_node = script.nodes.iter().find(|n| n.id == preview.current_node_id);
        let next_node_id = match current_node.and_then(|n| n.next_node_id.as_ref()) {
            Some(id) => id.clone(),
            None => {
                self.preview_state = None;
                return;
            }
        };

        let next_node = match script.nodes.iter().find(|n| n.id == next_node_id) {
            Some(n) => n,
            None => {
                self.preview_state = None;
                return;
            }
        };

        let speaker_name = next_node
            .speaker_id
            .as_ref()
            .and_then(|sid| script.characters.iter().find(|c| &c.id == sid))
            .map(|c| c.name.clone());

        preview.current_node_id = next_node.id.clone();
        preview.displayed_text = String::new();
        preview.full_text = next_node.text.clone();
        preview.speaker_name = speaker_name;
        preview.current_choices = next_node.choices.clone();
        preview.waiting_for_advance = false;
        preview.typewriter_index = 0;
    }

    /// 在预览中选择指定索引的选项
    ///
    /// 跳转到选项对应的下一个节点，并重置打字机效果。
    /// 如果索引越界或没有激活的预览，则不做任何操作。
    pub fn select_preview_choice(&mut self, choice_index: usize) {
        let preview = match &mut self.preview_state {
            Some(p) => p,
            None => return,
        };

        let choice = match preview.current_choices.get(choice_index) {
            Some(c) => c.clone(),
            None => return,
        };

        let script = match &self.current_script {
            Some(s) => s,
            None => return,
        };

        let next_node = match script.nodes.iter().find(|n| n.id == choice.next_node_id) {
            Some(n) => n,
            None => return,
        };

        let speaker_name = next_node
            .speaker_id
            .as_ref()
            .and_then(|sid| script.characters.iter().find(|c| &c.id == sid))
            .map(|c| c.name.clone());

        preview.current_node_id = next_node.id.clone();
        preview.displayed_text = String::new();
        preview.full_text = next_node.text.clone();
        preview.speaker_name = speaker_name;
        preview.current_choices = next_node.choices.clone();
        preview.waiting_for_advance = false;
        preview.typewriter_index = 0;
    }

    /// 停止对话预览
    pub fn stop_preview(&mut self) {
        self.preview_state = None;
    }

    /// 检查是否正在预览对话
    pub fn is_previewing(&self) -> bool {
        self.preview_state.is_some()
    }

    /// 获取当前选中的对话节点的不可变引用
    ///
    /// 从当前脚本中查找与 `selected_node_id` 匹配的节点。
    pub fn selected_node(&self) -> Option<&DialogueNode> {
        let script = self.current_script.as_ref()?;
        let id = self.selected_node_id.as_ref()?;
        script.nodes.iter().find(|n| &n.id == id)
    }

    /// 获取当前选中的对话节点的可变引用
    ///
    /// 从当前脚本中查找与 `selected_node_id` 匹配的节点。
    pub fn selected_node_mut(&mut self) -> Option<&mut DialogueNode> {
        let script = self.current_script.as_mut()?;
        let id = self.selected_node_id.clone()?;
        script.nodes.iter_mut().find(|n| n.id == id)
    }
}

impl Default for DialogueEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// 将对话命令格式化为可读字符串
fn format_command(cmd: &DialogueCommand) -> String {
    match cmd {
        DialogueCommand::PlayBgm { asset_path, volume, fade_in_secs } => {
            format!("PlayBgm: {} (vol={}, fade={})", asset_path, volume, fade_in_secs)
        }
        DialogueCommand::StopBgm { fade_out_secs } => {
            format!("StopBgm (fade={})", fade_out_secs)
        }
        DialogueCommand::PlaySe { asset_path, volume } => {
            format!("PlaySe: {} (vol={})", asset_path, volume)
        }
        DialogueCommand::ShowPortrait { character_id, expression, .. } => {
            format!("ShowPortrait: {} [{}]", character_id, expression)
        }
        DialogueCommand::HidePortrait { character_id, .. } => {
            format!("HidePortrait: {}", character_id)
        }
        DialogueCommand::ChangeBackground { asset_path, .. } => {
            format!("ChangeBackground: {}", asset_path)
        }
        DialogueCommand::SetVariable { name, .. } => {
            format!("SetVariable: {}", name)
        }
        DialogueCommand::Wait { duration_secs } => {
            format!("Wait: {}s", duration_secs)
        }
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
    /// 创建包含标题、节点列表、属性编辑区域、预览按钮和预览区域的面板布局。
    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> GResult<()> {
        let root_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0).with_gap(4.0));

        let root_id = ui_tree.create_node("dialogue_editor_root", root_style, UiNodeData::Container);
        ui_tree.set_root(root_id);

        let title_style = Style::new().with_font(FontStyle::new().with_size(18.0));
        let title_id = ui_tree.create_node(
            "dialogue_title",
            title_style,
            UiNodeData::Text { content: "Dialogue Editor".to_string() },
        );
        ui_tree.add_child(root_id, title_id);

        let list_style = Style::new()
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0));
        let list_id = ui_tree.create_node("dialogue_node_list", list_style, UiNodeData::Container);
        ui_tree.add_child(root_id, list_id);

        let list_header_style = Style::new().with_font(FontStyle::new().with_size(14.0));
        let list_header_id = ui_tree.create_node(
            "node_list_header",
            list_header_style,
            UiNodeData::Text { content: "Nodes".to_string() },
        );
        ui_tree.add_child(list_id, list_header_id);

        if let Some(script) = &self.current_script {
            for node in &script.nodes {
                let text_summary = if node.text.len() > 30 {
                    format!("{}...", &node.text[..30])
                } else {
                    node.text.clone()
                };
                let speaker_label = node
                    .speaker_id
                    .as_ref()
                    .and_then(|sid| script.characters.iter().find(|c| &c.id == sid))
                    .map(|c| c.name.clone())
                    .or_else(|| node.speaker_id.clone())
                    .unwrap_or_else(|| "Narrator".to_string());

                let item_style = Style::new()
                    .with_layout(LayoutStyle::new().with_padding(4.0))
                    .with_font(FontStyle::new().with_size(13.0));
                let label = format!("{} [{}]: {}", node.id, speaker_label, text_summary);
                let item_id = ui_tree.create_node(
                    format!("node_item_{}", node.id),
                    item_style,
                    UiNodeData::Text { content: label },
                );
                ui_tree.add_child(list_id, item_id);
            }
        } else {
            let empty_style = Style::new().with_font(FontStyle::new().with_size(13.0));
            let empty_id = ui_tree.create_node(
                "node_list_empty",
                empty_style,
                UiNodeData::Text { content: "No script loaded".to_string() },
            );
            ui_tree.add_child(list_id, empty_id);
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

            let speaker_label = self
                .edit_state
                .editing_speaker_id
                .as_deref()
                .unwrap_or("(none)");
            let speaker_id = ui_tree.create_node(
                "prop_speaker",
                prop_style.clone(),
                UiNodeData::Text { content: format!("Speaker: {}", speaker_label) },
            );
            ui_tree.add_child(section_id, speaker_id);

            let text_preview = if self.edit_state.editing_text.len() > 80 {
                format!("Text: {}...", &self.edit_state.editing_text[..80])
            } else if self.edit_state.editing_text.is_empty() {
                "Text: (empty)".to_string()
            } else {
                format!("Text: {}", self.edit_state.editing_text)
            };
            let text_id = ui_tree.create_node(
                "prop_text",
                prop_style.clone(),
                UiNodeData::Text { content: text_preview },
            );
            ui_tree.add_child(section_id, text_id);

            let cmd_header_style = Style::new().with_font(FontStyle::new().with_size(14.0));
            let cmd_header_id = ui_tree.create_node(
                "prop_commands_header",
                cmd_header_style,
                UiNodeData::Text { content: "Commands".to_string() },
            );
            ui_tree.add_child(section_id, cmd_header_id);

            let cmd_list_style = Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(1.0));
            let cmd_list_id = ui_tree.create_node("prop_commands_list", cmd_list_style, UiNodeData::Container);
            ui_tree.add_child(section_id, cmd_list_id);

            for (i, cmd) in self.edit_state.editing_commands.iter().enumerate() {
                let cmd_row_style = Style::new()
                    .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(4.0));
                let cmd_row_id = ui_tree.create_node(
                    format!("prop_command_row_{}", i),
                    cmd_row_style,
                    UiNodeData::Container,
                );
                ui_tree.add_child(cmd_list_id, cmd_row_id);

                let cmd_item_style = Style::new().with_font(FontStyle::new().with_size(12.0));
                let cmd_item_id = ui_tree.create_node(
                    format!("prop_command_{}", i),
                    cmd_item_style,
                    UiNodeData::Text { content: format_command(cmd) },
                );
                ui_tree.add_child(cmd_row_id, cmd_item_id);

                let cmd_del_style = Style::new().with_font(FontStyle::new().with_size(12.0));
                let cmd_del_id = ui_tree.create_node(
                    format!("prop_command_del_{}", i),
                    cmd_del_style,
                    UiNodeData::Custom { kind: "button_delete_command".to_string() },
                );
                ui_tree.add_child(cmd_row_id, cmd_del_id);
            }

            let add_cmd_style = Style::new().with_font(FontStyle::new().with_size(12.0));
            let add_cmd_id = ui_tree.create_node(
                "btn_add_command",
                add_cmd_style,
                UiNodeData::Custom { kind: "button_add_command".to_string() },
            );
            ui_tree.add_child(cmd_list_id, add_cmd_id);

            let choice_header_style = Style::new().with_font(FontStyle::new().with_size(14.0));
            let choice_header_id = ui_tree.create_node(
                "prop_choices_header",
                choice_header_style,
                UiNodeData::Text { content: "Choices".to_string() },
            );
            ui_tree.add_child(section_id, choice_header_id);

            let choice_list_style = Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(1.0));
            let choice_list_id =
                ui_tree.create_node("prop_choices_list", choice_list_style, UiNodeData::Container);
            ui_tree.add_child(section_id, choice_list_id);

            for (i, choice) in self.edit_state.editing_choices.iter().enumerate() {
                let choice_row_style = Style::new()
                    .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(4.0));
                let choice_row_id = ui_tree.create_node(
                    format!("prop_choice_row_{}", i),
                    choice_row_style,
                    UiNodeData::Container,
                );
                ui_tree.add_child(choice_list_id, choice_row_id);

                let choice_item_style = Style::new().with_font(FontStyle::new().with_size(12.0));
                let condition_label = choice
                    .condition
                    .as_deref()
                    .map(|c| format!(" [if: {}]", c))
                    .unwrap_or_default();
                let choice_item_id = ui_tree.create_node(
                    format!("prop_choice_{}", i),
                    choice_item_style,
                    UiNodeData::Text { content: format!("{} -> {}{}", choice.text, choice.next_node_id, condition_label) },
                );
                ui_tree.add_child(choice_row_id, choice_item_id);

                let choice_del_style = Style::new().with_font(FontStyle::new().with_size(12.0));
                let choice_del_id = ui_tree.create_node(
                    format!("prop_choice_del_{}", i),
                    choice_del_style,
                    UiNodeData::Custom { kind: "button_delete_choice".to_string() },
                );
                ui_tree.add_child(choice_row_id, choice_del_id);
            }

            let add_choice_style = Style::new().with_font(FontStyle::new().with_size(12.0));
            let add_choice_id = ui_tree.create_node(
                "btn_add_choice",
                add_choice_style,
                UiNodeData::Custom { kind: "button_add_choice".to_string() },
            );
            ui_tree.add_child(choice_list_id, add_choice_id);

            let apply_btn_style = Style::new().with_font(FontStyle::new().with_size(13.0));
            let apply_btn_id = ui_tree.create_node(
                "btn_apply_edit",
                apply_btn_style,
                UiNodeData::Custom { kind: "button_apply".to_string() },
            );
            ui_tree.add_child(section_id, apply_btn_id);
        }

        let preview_btn_style = Style::new().with_font(FontStyle::new().with_size(13.0));
        let preview_btn_kind = if self.is_previewing() {
            "button_stop_preview"
        } else {
            "button_start_preview"
        };
        let preview_btn_id = ui_tree.create_node(
            "btn_preview",
            preview_btn_style,
            UiNodeData::Custom { kind: preview_btn_kind.to_string() },
        );
        ui_tree.add_child(root_id, preview_btn_id);

        if let Some(preview) = &self.preview_state {
            let preview_style = Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(4.0).with_padding(8.0));
            let preview_id = ui_tree.create_node("dialogue_preview", preview_style, UiNodeData::Container);
            ui_tree.add_child(root_id, preview_id);

            let preview_header_style = Style::new().with_font(FontStyle::new().with_size(16.0));
            let preview_header_id = ui_tree.create_node(
                "preview_header",
                preview_header_style,
                UiNodeData::Text { content: "Preview".to_string() },
            );
            ui_tree.add_child(preview_id, preview_header_id);

            let speaker_style = Style::new().with_font(FontStyle::new().with_size(15.0));
            let speaker_text = preview
                .speaker_name
                .as_deref()
                .unwrap_or("Narrator");
            let speaker_id = ui_tree.create_node(
                "preview_speaker",
                speaker_style,
                UiNodeData::Text { content: speaker_text.to_string() },
            );
            ui_tree.add_child(preview_id, speaker_id);

            let text_style = Style::new().with_font(FontStyle::new().with_size(14.0));
            let text_id = ui_tree.create_node(
                "preview_text",
                text_style,
                UiNodeData::Text { content: preview.displayed_text.clone() },
            );
            ui_tree.add_child(preview_id, text_id);

            if preview.typewriter_index >= preview.full_text.len()
                && !preview.current_choices.is_empty()
            {
                let choices_style = Style::new()
                    .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0));
                let choices_id = ui_tree.create_node("preview_choices", choices_style, UiNodeData::Container);
                ui_tree.add_child(preview_id, choices_id);

                for (i, choice) in preview.current_choices.iter().enumerate() {
                    let choice_btn_style = Style::new().with_font(FontStyle::new().with_size(13.0));
                    let choice_btn_id = ui_tree.create_node(
                        format!("preview_choice_{}", i),
                        choice_btn_style,
                        UiNodeData::Custom { kind: "button_choice".to_string() },
                    );
                    ui_tree.add_child(choices_id, choice_btn_id);

                    let choice_label_style = Style::new().with_font(FontStyle::new().with_size(13.0));
                    let choice_label_id = ui_tree.create_node(
                        format!("preview_choice_label_{}", i),
                        choice_label_style,
                        UiNodeData::Text { content: choice.text.clone() },
                    );
                    ui_tree.add_child(choice_btn_id, choice_label_id);
                }
            }

            let advance_btn_style = Style::new().with_font(FontStyle::new().with_size(13.0));
            let advance_btn_id = ui_tree.create_node(
                "btn_advance_preview",
                advance_btn_style,
                UiNodeData::Custom { kind: "button_advance".to_string() },
            );
            ui_tree.add_child(preview_id, advance_btn_id);
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
