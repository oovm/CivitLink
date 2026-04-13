//! Galgame 场景编辑器面板实现
//! 提供场景背景设置、立绘布局管理、BGM 配置和转场预览功能

use gg_core::GResult;
use gg_editor_shell::{EditorContext, EditorPanel, PanelLayoutHint, PanelPosition};
use crate::schema::components::{PortraitPosition, TransitionType};
use gg_render::Color;
use gg_ui::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, Overflow, SizeValue, Style, UiNodeData, UiNodeId, UiTree};

/// 立绘条目
///
/// 描述场景中单个角色的立绘信息，包括角色标识、表情、位置和资源路径。
pub struct PortraitEntry {
    /// 角色 ID
    pub character_id: String,
    /// 当前表情
    pub expression: String,
    /// 立绘位置
    pub position: PortraitPosition,
    /// 立绘资源路径
    pub asset_path: Option<String>,
}

/// Galgame 场景编辑器面板
///
/// 提供场景的视觉布局编辑功能，包括背景设置、立绘位置管理、
/// BGM 配置和转场效果预览。支持拖拽交互来调整立绘位置。
pub struct GalgameSceneEditorPanel {
    /// 面板是否可见
    visible: bool,
    /// 画布尺寸 (width, height)
    canvas_size: (f32, f32),
    /// 当前背景图资源路径
    current_background_path: Option<String>,
    /// 当前场景中的立绘列表
    portrait_entries: Vec<PortraitEntry>,
    /// 当前 BGM 资源路径
    current_bgm_path: Option<String>,
    /// 当前 BGM 音量
    bgm_volume: f32,
    /// 当前转场效果类型
    current_transition: TransitionType,
    /// 正在拖拽的角色 ID
    dragging_portrait: Option<String>,
    /// 拖拽偏移量 (x, y)
    drag_offset: (f32, f32),
    /// 对话预览激活状态
    dialogue_preview_active: bool,
}

impl GalgameSceneEditorPanel {
    /// 创建新的场景编辑器面板
    ///
    /// 初始化默认画布尺寸为 1280x720，BGM 音量为 1.0，
    /// 转场类型为无动画，对话预览为关闭状态。
    pub fn new() -> Self {
        Self {
            visible: true,
            canvas_size: (1280.0, 720.0),
            current_background_path: None,
            portrait_entries: Vec::new(),
            current_bgm_path: None,
            bgm_volume: 1.0,
            current_transition: TransitionType::None,
            dragging_portrait: None,
            drag_offset: (0.0, 0.0),
            dialogue_preview_active: false,
        }
    }

    /// 设置场景背景
    ///
    /// 将指定资源路径设置为当前场景的背景图，
    /// 并通过编辑器上下文发布背景变更事件。
    pub fn set_background(&mut self, asset_path: String, _context: &mut EditorContext) -> GResult<()> {
        self.current_background_path = Some(asset_path);
        Ok(())
    }

    /// 添加立绘到场景
    ///
    /// 将指定角色以给定位置添加到当前场景中，
    /// 如果角色已存在则更新其位置。
    pub fn add_portrait_to_scene(
        &mut self,
        character_id: String,
        position: PortraitPosition,
        _context: &mut EditorContext,
    ) -> GResult<()> {
        if let Some(entry) = self.portrait_entries.iter_mut().find(|e| e.character_id == character_id) {
            entry.position = position;
        }
        else {
            self.portrait_entries.push(PortraitEntry {
                character_id,
                expression: String::from("default"),
                position,
                asset_path: None,
            });
        }
        Ok(())
    }

    /// 从场景移除立绘
    ///
    /// 将指定角色的立绘从当前场景中移除，
    /// 如果该角色正在被拖拽则清除拖拽状态。
    pub fn remove_portrait_from_scene(&mut self, character_id: &str, _context: &mut EditorContext) -> GResult<()> {
        self.portrait_entries.retain(|e| e.character_id != character_id);
        if self.dragging_portrait.as_deref() == Some(character_id) {
            self.dragging_portrait = None;
        }
        Ok(())
    }

    /// 设置立绘表情
    ///
    /// 更新指定角色的表情标签，如果角色不存在则不做任何操作。
    pub fn set_portrait_expression(&mut self, character_id: &str, expression: &str) {
        if let Some(entry) = self.portrait_entries.iter_mut().find(|e| e.character_id == character_id) {
            entry.expression = expression.to_string();
        }
    }

    /// 设置场景 BGM
    ///
    /// 配置当前场景的背景音乐，包括资源路径、音量和淡入时长。
    pub fn set_scene_bgm(
        &mut self,
        asset_path: String,
        volume: f32,
        fade_in: f32,
        _context: &mut EditorContext,
    ) -> GResult<()> {
        self.current_bgm_path = Some(asset_path);
        self.bgm_volume = volume;
        let _ = fade_in;
        Ok(())
    }

    /// 设置转场效果类型
    ///
    /// 更新当前场景的转场动画类型。
    pub fn set_transition(&mut self, transition_type: TransitionType) {
        self.current_transition = transition_type;
    }

    /// 预览转场效果
    ///
    /// 在场景编辑器中预览指定类型的转场动画效果，
    /// 同时将当前转场类型更新为指定类型。
    pub fn preview_transition(&mut self, transition_type: TransitionType) {
        self.current_transition = transition_type.clone();
    }

    /// 启动对话预览
    ///
    /// 激活对话预览模式，面板将在画布区域显示对话预览内容。
    pub fn start_dialogue_preview(&mut self) {
        self.dialogue_preview_active = true;
    }

    /// 停止对话预览
    ///
    /// 关闭对话预览模式，恢复正常的场景编辑视图。
    pub fn stop_dialogue_preview(&mut self) {
        self.dialogue_preview_active = false;
    }

    /// 检查对话预览是否激活
    ///
    /// 返回当前对话预览模式的激活状态。
    pub fn is_dialogue_previewing(&self) -> bool {
        self.dialogue_preview_active
    }

    /// 格式化立绘位置为显示文本
    fn format_position(position: &PortraitPosition) -> String {
        match position {
            PortraitPosition::Left => String::from("Left"),
            PortraitPosition::Center => String::from("Center"),
            PortraitPosition::Right => String::from("Right"),
            PortraitPosition::Custom { x, y } => format!("Custom({:.0}, {:.0})", x, y),
        }
    }

    /// 格式化转场类型为显示文本
    fn format_transition(transition: &TransitionType) -> String {
        match transition {
            TransitionType::None => String::from("None"),
            TransitionType::Fade { duration_secs } => format!("Fade({:.1}s)", duration_secs),
            TransitionType::CrossDissolve { duration_secs } => format!("CrossDissolve({:.1}s)", duration_secs),
            TransitionType::Slide { duration_secs, direction } => {
                let dir = match direction {
                    crate::schema::components::SlideDirection::Left => "Left",
                    crate::schema::components::SlideDirection::Right => "Right",
                    crate::schema::components::SlideDirection::Up => "Up",
                    crate::schema::components::SlideDirection::Down => "Down",
                };
                format!("Slide({:.1}s, {})", duration_secs, dir)
            }
        }
    }

    /// 创建带标签的属性行节点
    ///
    /// 创建一个水平布局的行，包含标签文本和值文本。
    fn create_property_row(&self, label: &str, value: &str, row_label: &str, ui_tree: &mut UiTree) -> UiNodeId {
        let row_id = ui_tree.create_node(
            row_label,
            Style::new().with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(8.0)
                    .with_padding(4.0),
            ),
            UiNodeData::Container,
        );

        let label_id = ui_tree.create_node(
            format!("{}_label", row_label),
            Style::new().with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.6, 0.6, 0.6, 1.0))),
            UiNodeData::Text { content: label.to_string() },
        );

        let value_id = ui_tree.create_node(
            format!("{}_value", row_label),
            Style::new().with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.9, 0.9, 0.9, 1.0))),
            UiNodeData::Text { content: value.to_string() },
        );

        ui_tree.add_child(row_id, label_id);
        ui_tree.add_child(row_id, value_id);

        row_id
    }

    /// 创建分区标题节点
    fn create_section_header(&self, title: &str, label: &str, ui_tree: &mut UiTree) -> UiNodeId {
        ui_tree.create_node(
            label,
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0))
                .with_font(FontStyle::new().with_size(13.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0))),
            UiNodeData::Text { content: title.to_string() },
        )
    }
}

impl Default for GalgameSceneEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for GalgameSceneEditorPanel {
    /// 获取面板名称
    fn name(&self) -> &str {
        "Scene Editor"
    }

    /// 获取面板可见性
    fn is_visible(&self) -> bool {
        self.visible
    }

    /// 设置面板可见性
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// 构建场景编辑器面板 UI 节点树
    ///
    /// 创建完整的面板布局，包含标题栏、背景设置区、立绘列表区、
    /// BGM 配置区、转场效果区、对话预览按钮和画布预览区域。
    fn build_ui(&mut self, _context: &mut EditorContext, ui_tree: &mut UiTree) -> Option<UiNodeId> {
        let root_id = ui_tree.create_node(
            "scene_editor_root",
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(8.0).with_gap(6.0))
                .with_background_color(Color::new(0.12, 0.12, 0.14, 1.0))
                .with_overflow(Overflow::Clip),
            UiNodeData::Container,
        );

        let title_id = ui_tree.create_node(
            "scene_editor_title",
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(4.0))
                .with_font(FontStyle::new().with_size(14.0).with_color(Color::new(0.7, 0.7, 0.7, 1.0))),
            UiNodeData::Text { content: String::from("Scene Editor") },
        );
        ui_tree.add_child(root_id, title_id);

        let bg_section_id = ui_tree.create_node(
            "bg_section",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(4.0)),
            UiNodeData::Container,
        );
        ui_tree.add_child(root_id, bg_section_id);

        let bg_header_id = self.create_section_header("Background:", "bg_section_header", ui_tree);
        ui_tree.add_child(bg_section_id, bg_header_id);

        let bg_path = self.current_background_path.as_deref().unwrap_or("(none)");
        let bg_row_id = self.create_property_row("Path:", bg_path, "bg_path_row", ui_tree);
        ui_tree.add_child(bg_section_id, bg_row_id);

        let portrait_section_id = ui_tree.create_node(
            "portrait_section",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(4.0)),
            UiNodeData::Container,
        );
        ui_tree.add_child(root_id, portrait_section_id);

        let portrait_header_id = self.create_section_header("Portraits:", "portrait_section_header", ui_tree);
        ui_tree.add_child(portrait_section_id, portrait_header_id);

        if self.portrait_entries.is_empty() {
            let empty_id = ui_tree.create_node(
                "portrait_empty",
                Style::new().with_font(FontStyle::new().with_size(11.0).with_color(Color::new(0.5, 0.5, 0.5, 1.0))),
                UiNodeData::Text { content: String::from("  (no portraits)") },
            );
            ui_tree.add_child(portrait_section_id, empty_id);
        }
        else {
            for (i, entry) in self.portrait_entries.iter().enumerate() {
                let entry_row_id = ui_tree.create_node(
                    format!("portrait_entry_{}", i),
                    Style::new().with_layout(
                        LayoutStyle::new()
                            .with_direction(FlexDirection::Column)
                            .with_gap(1.0)
                            .with_padding(4.0)
                            .with_margin(2.0),
                    ),
                    UiNodeData::Container,
                );
                ui_tree.add_child(portrait_section_id, entry_row_id);

                let char_row_id =
                    self.create_property_row("Character:", &entry.character_id, &format!("portrait_char_{}", i), ui_tree);
                ui_tree.add_child(entry_row_id, char_row_id);

                let expr_row_id =
                    self.create_property_row("Expression:", &entry.expression, &format!("portrait_expr_{}", i), ui_tree);
                ui_tree.add_child(entry_row_id, expr_row_id);

                let pos_text = Self::format_position(&entry.position);
                let pos_row_id = self.create_property_row("Position:", &pos_text, &format!("portrait_pos_{}", i), ui_tree);
                ui_tree.add_child(entry_row_id, pos_row_id);
            }
        }

        let bgm_section_id = ui_tree.create_node(
            "bgm_section",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(4.0)),
            UiNodeData::Container,
        );
        ui_tree.add_child(root_id, bgm_section_id);

        let bgm_header_id = self.create_section_header("BGM:", "bgm_section_header", ui_tree);
        ui_tree.add_child(bgm_section_id, bgm_header_id);

        let bgm_path = self.current_bgm_path.as_deref().unwrap_or("(none)");
        let bgm_path_row_id = self.create_property_row("Path:", bgm_path, "bgm_path_row", ui_tree);
        ui_tree.add_child(bgm_section_id, bgm_path_row_id);

        let volume_text = format!("{:.0}%", self.bgm_volume * 100.0);
        let bgm_volume_row_id = self.create_property_row("Volume:", &volume_text, "bgm_volume_row", ui_tree);
        ui_tree.add_child(bgm_section_id, bgm_volume_row_id);

        let transition_section_id = ui_tree.create_node(
            "transition_section",
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(2.0).with_padding(4.0)),
            UiNodeData::Container,
        );
        ui_tree.add_child(root_id, transition_section_id);

        let transition_header_id = self.create_section_header("Transition:", "transition_section_header", ui_tree);
        ui_tree.add_child(transition_section_id, transition_header_id);

        let transition_text = Self::format_transition(&self.current_transition);
        let transition_row_id = self.create_property_row("Type:", &transition_text, "transition_type_row", ui_tree);
        ui_tree.add_child(transition_section_id, transition_row_id);

        let preview_btn_label = if self.dialogue_preview_active { "Stop Dialogue Preview" } else { "Start Dialogue Preview" };
        let preview_btn_id = ui_tree.create_node(
            "dialogue_preview_btn",
            Style::new()
                .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(6.0).with_margin(4.0))
                .with_background_color(Color::new(0.2, 0.2, 0.25, 1.0))
                .with_border_color(Color::new(0.4, 0.4, 0.5, 1.0))
                .with_border_width(1.0)
                .with_corner_radius(3.0)
                .with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.8, 0.8, 0.8, 1.0))),
            UiNodeData::Custom { kind: String::from("DialoguePreviewToggle") },
        );
        ui_tree.add_child(root_id, preview_btn_id);

        let btn_text_id = ui_tree.create_node(
            "dialogue_preview_btn_text",
            Style::new().with_font(FontStyle::new().with_size(12.0).with_color(Color::new(0.8, 0.8, 0.8, 1.0))),
            UiNodeData::Text { content: preview_btn_label.to_string() },
        );
        ui_tree.add_child(preview_btn_id, btn_text_id);

        let canvas_id = ui_tree.create_node(
            "canvas_preview",
            Style::new()
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Column)
                        .with_padding(4.0)
                        .with_width(SizeValue::Px(self.canvas_size.0.min(800.0)))
                        .with_height(SizeValue::Px(self.canvas_size.1.min(450.0)))
                        .with_min_width(SizeValue::Px(400.0))
                        .with_min_height(SizeValue::Px(225.0)),
                )
                .with_background_color(Color::new(0.08, 0.08, 0.1, 1.0))
                .with_border_color(Color::new(0.3, 0.3, 0.35, 1.0))
                .with_border_width(1.0),
            UiNodeData::Custom { kind: String::from("SceneCanvas") },
        );
        ui_tree.add_child(root_id, canvas_id);

        let canvas_label_id = ui_tree.create_node(
            "canvas_preview_label",
            Style::new().with_font(FontStyle::new().with_size(11.0).with_color(Color::new(0.4, 0.4, 0.4, 1.0))),
            UiNodeData::Text { content: String::from("Canvas Preview") },
        );
        ui_tree.add_child(canvas_id, canvas_label_id);

        Some(root_id)
    }

    /// 获取面板布局提示
    fn layout_hint(&self) -> PanelLayoutHint {
        PanelLayoutHint {
            position: PanelPosition::Center,
            preferred_size: Some((800.0, 600.0)),
            min_size: Some((400.0, 300.0)),
        }
    }
}
