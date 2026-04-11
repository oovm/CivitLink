//! 存档 UI 组件模块
//! 提供存档槽位、存档面板和自动存档指示器等 UI 组件

use gg_core::GResult;
use gg_render::TextureId;
use gg_ui::{FlexDirection, FontStyle, LayoutStyle, SizeValue, Style, UiNodeData, UiNodeId, UiTree, Widget};

/// 存档槽位 UI 组件
///
/// 显示单个存档槽位的信息，包括缩略图、时间戳、章节和游戏变量摘要。
pub struct SaveSlotWidget {
    /// 存档槽位号
    pub slot: u32,
    /// 存档时间戳
    pub timestamp: Option<f64>,
    /// 当前节点 ID（章节信息）
    pub current_node_id: Option<String>,
    /// 游戏变量摘要
    pub variable_summary: Option<String>,
    /// 是否有截图
    pub has_screenshot: bool,
    /// 截图纹理 ID
    pub screenshot_texture_id: Option<TextureId>,
    /// 是否为空槽位
    pub is_empty: bool,
    /// 样式
    pub style: Style,
    /// 点击回调
    pub on_click: Option<Box<dyn FnMut(u32) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 缩略图节点 ID
    thumbnail_node_id: Option<UiNodeId>,
    /// 信息区域节点 ID
    info_node_id: Option<UiNodeId>,
}

impl SaveSlotWidget {
    /// 创建存档槽位组件
    ///
    /// # 参数
    ///
    /// - `slot` - 存档槽位号
    pub fn new(slot: u32) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.15, 0.15, 0.15, 1.0))
            .with_border_color(gg_render::Color::new(0.3, 0.3, 0.3, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(8.0).with_gap(8.0));

        Self {
            slot,
            timestamp: None,
            current_node_id: None,
            variable_summary: None,
            has_screenshot: false,
            screenshot_texture_id: None,
            is_empty: false,
            style,
            on_click: None,
            node_id: None,
            thumbnail_node_id: None,
            info_node_id: None,
        }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 设置存档数据
    ///
    /// # 参数
    ///
    /// - `timestamp` - 存档时间戳
    /// - `node_id` - 当前节点 ID
    /// - `summary` - 游戏变量摘要
    pub fn with_save_data(mut self, timestamp: f64, node_id: String, summary: String) -> Self {
        self.timestamp = Some(timestamp);
        self.current_node_id = Some(node_id);
        self.variable_summary = Some(summary);
        self.is_empty = false;
        self
    }

    /// 标记为空槽位
    pub fn mark_empty(mut self) -> Self {
        self.is_empty = true;
        self.timestamp = None;
        self.current_node_id = None;
        self.variable_summary = None;
        self.has_screenshot = false;
        self.screenshot_texture_id = None;
        self
    }

    /// 设置截图纹理
    ///
    /// # 参数
    ///
    /// - `texture_id` - 截图纹理 ID
    pub fn set_screenshot(&mut self, texture_id: TextureId) {
        self.screenshot_texture_id = Some(texture_id);
        self.has_screenshot = true;
    }

    /// 格式化时间戳为可读字符串
    pub fn format_timestamp(&self) -> String {
        match self.timestamp {
            Some(ts) => {
                let secs = ts as u64;
                let datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
                let datetime_str = format!("{:?}", datetime);
                datetime_str
            }
            None => String::from("---"),
        }
    }
}

impl Widget for SaveSlotWidget {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node(format!("SaveSlot({})", self.slot), self.style.clone(), UiNodeData::Container);

        let thumbnail_style = Style::new()
            .with_background_color(gg_render::Color::new(0.1, 0.1, 0.1, 1.0))
            .with_corner_radius(2.0)
            .with_layout(LayoutStyle::new().with_width(SizeValue::Px(120.0)).with_height(SizeValue::Px(80.0)));

        let thumbnail_data = if let Some(texture_id) = self.screenshot_texture_id {
            UiNodeData::Image { texture_id: Some(texture_id), size: Some((120.0, 80.0)) }
        }
        else {
            UiNodeData::Container
        };

        let thumbnail_id = tree.create_node(format!("SaveSlot_Thumbnail({})", self.slot), thumbnail_style, thumbnail_data);

        let info_style =
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(4.0).with_padding(4.0));

        let info_id = tree.create_node(format!("SaveSlot_Info({})", self.slot), info_style, UiNodeData::Container);

        let slot_label_style =
            Style::new().with_font(FontStyle::new().with_size(14.0).with_color(gg_render::Color::new(0.8, 0.8, 0.8, 1.0)));

        let slot_label_id = tree.create_node(
            format!("SaveSlot_Label({})", self.slot),
            slot_label_style,
            UiNodeData::Text { content: format!("Slot {}", self.slot) },
        );

        tree.add_child(info_id, slot_label_id);

        if self.is_empty {
            let empty_style =
                Style::new().with_font(FontStyle::new().with_size(12.0).with_color(gg_render::Color::new(0.5, 0.5, 0.5, 1.0)));

            let empty_id = tree.create_node(
                format!("SaveSlot_Empty({})", self.slot),
                empty_style,
                UiNodeData::Text { content: String::from("Empty Slot") },
            );

            tree.add_child(info_id, empty_id);
        }
        else {
            let timestamp_style =
                Style::new().with_font(FontStyle::new().with_size(12.0).with_color(gg_render::Color::new(0.7, 0.7, 0.7, 1.0)));

            let timestamp_id = tree.create_node(
                format!("SaveSlot_Timestamp({})", self.slot),
                timestamp_style,
                UiNodeData::Text { content: self.format_timestamp() },
            );

            tree.add_child(info_id, timestamp_id);

            if let Some(ref node_id_str) = self.current_node_id {
                let chapter_style = Style::new()
                    .with_font(FontStyle::new().with_size(12.0).with_color(gg_render::Color::new(0.7, 0.7, 0.7, 1.0)));

                let chapter_id = tree.create_node(
                    format!("SaveSlot_Chapter({})", self.slot),
                    chapter_style,
                    UiNodeData::Text { content: format!("Chapter: {}", node_id_str) },
                );

                tree.add_child(info_id, chapter_id);
            }

            if let Some(ref summary) = self.variable_summary {
                let summary_style = Style::new()
                    .with_font(FontStyle::new().with_size(11.0).with_color(gg_render::Color::new(0.6, 0.6, 0.6, 1.0)));

                let summary_id = tree.create_node(
                    format!("SaveSlot_Summary({})", self.slot),
                    summary_style,
                    UiNodeData::Text { content: summary.clone() },
                );

                tree.add_child(info_id, summary_id);
            }
        }

        tree.add_child(root_id, thumbnail_id);
        tree.add_child(root_id, info_id);

        self.node_id = Some(root_id);
        self.thumbnail_node_id = Some(thumbnail_id);
        self.info_node_id = Some(info_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(thumbnail_id) = self.thumbnail_node_id {
            if let Some(node) = tree.get_mut(thumbnail_id) {
                if let Some(texture_id) = self.screenshot_texture_id {
                    node.data = UiNodeData::Image { texture_id: Some(texture_id), size: Some((120.0, 80.0)) };
                }
            }
        }

        if let Some(info_id) = self.info_node_id {
            if let Some(info_node) = tree.get_mut(info_id) {
                let children = info_node.children.clone();
                for child_id in children {
                    if let Some(child) = tree.get_mut(child_id) {
                        if let UiNodeData::Text { ref mut content } = child.data {
                            if content.starts_with("Slot ") {
                                *content = format!("Slot {}", self.slot);
                            }
                            else if content == "Empty Slot" && !self.is_empty {
                                *content = self.format_timestamp();
                            }
                        }
                    }
                }
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }

    fn render_template(&self) -> oak_voc::TemplateNode {
        oak_voc::TemplateNode::text(String::new())
    }

    fn script_setup(&mut self) {}

    fn get_id(&self) -> &str {
        ""
    }

    fn handle_event(&mut self, _event: &gg_ui::GuiEvent, _ctx: &mut gg_ui::EventContext) {}
}

/// 存档面板操作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavePanelMode {
    /// 保存模式
    Save,
    /// 读取模式
    Load,
}

/// 存档面板
///
/// 提供存档/读档/删除操作的面板，显示所有存档槽位。
pub struct SavePanel {
    /// 存档槽位数量
    pub slot_count: u32,
    /// 存档槽位组件列表
    pub slots: Vec<SaveSlotWidget>,
    /// 当前操作模式
    pub mode: SavePanelMode,
    /// 样式
    pub style: Style,
    /// 保存回调
    pub on_save: Option<Box<dyn FnMut(u32) + Send + Sync>>,
    /// 加载回调
    pub on_load: Option<Box<dyn FnMut(u32) + Send + Sync>>,
    /// 删除回调
    pub on_delete: Option<Box<dyn FnMut(u32) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 内容区域节点 ID
    content_node_id: Option<UiNodeId>,
    /// 保存按钮节点 ID
    save_button_node_id: Option<UiNodeId>,
    /// 加载按钮节点 ID
    load_button_node_id: Option<UiNodeId>,
    /// 删除按钮节点 ID
    delete_button_node_id: Option<UiNodeId>,
    /// 当前选中的存档槽位
    pub selected_slot: Option<u32>,
}

impl SavePanel {
    /// 创建存档面板
    ///
    /// # 参数
    ///
    /// - `slot_count` - 存档槽位数量
    pub fn new(slot_count: u32) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.1, 0.1, 0.1, 0.95))
            .with_border_color(gg_render::Color::new(0.3, 0.3, 0.3, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(8.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_padding(16.0).with_gap(8.0));

        let slots = (0..slot_count).map(|i| SaveSlotWidget::new(i).mark_empty()).collect();

        Self {
            slot_count,
            slots,
            mode: SavePanelMode::Save,
            style,
            on_save: None,
            on_load: None,
            on_delete: None,
            node_id: None,
            content_node_id: None,
            save_button_node_id: None,
            load_button_node_id: None,
            delete_button_node_id: None,
            selected_slot: None,
        }
    }

    /// 设置操作模式
    pub fn with_mode(mut self, mode: SavePanelMode) -> Self {
        self.mode = mode;
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 动态设置操作模式
    pub fn set_mode(&mut self, mode: SavePanelMode) {
        self.mode = mode;
    }

    /// 更新指定槽位的存档数据
    ///
    /// # 参数
    ///
    /// - `slot` - 存档槽位号
    /// - `timestamp` - 存档时间戳
    /// - `node_id` - 当前节点 ID
    /// - `summary` - 游戏变量摘要
    pub fn update_slot(&mut self, slot: u32, timestamp: f64, node_id: String, summary: String) {
        if let Some(slot_widget) = self.slots.iter_mut().find(|s| s.slot == slot) {
            slot_widget.timestamp = Some(timestamp);
            slot_widget.current_node_id = Some(node_id);
            slot_widget.variable_summary = Some(summary);
            slot_widget.is_empty = false;
        }
    }

    /// 标记指定槽位为空
    ///
    /// # 参数
    ///
    /// - `slot` - 存档槽位号
    pub fn mark_slot_empty(&mut self, slot: u32) {
        if let Some(slot_widget) = self.slots.iter_mut().find(|s| s.slot == slot) {
            slot_widget.is_empty = true;
            slot_widget.timestamp = None;
            slot_widget.current_node_id = None;
            slot_widget.variable_summary = None;
            slot_widget.has_screenshot = false;
            slot_widget.screenshot_texture_id = None;
        }
    }

    /// 选中指定存档槽位
    ///
    /// # 参数
    ///
    /// - `slot` - 存档槽位号
    pub fn select_slot(&mut self, slot: u32) {
        self.selected_slot = Some(slot);
    }
}

impl Widget for SavePanel {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("SavePanel", self.style.clone(), UiNodeData::Container);

        let mode_text = match self.mode {
            SavePanelMode::Save => "Save Game",
            SavePanelMode::Load => "Load Game",
        };

        let title_style = Style::new()
            .with_font(FontStyle::new().with_size(18.0).with_color(gg_render::Color::WHITE))
            .with_layout(LayoutStyle::new().with_padding(8.0));

        let title_id = tree.create_node("SavePanel_Title", title_style, UiNodeData::Text { content: mode_text.to_string() });

        tree.add_child(root_id, title_id);

        let button_bar_style =
            Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_gap(8.0).with_padding(4.0));

        let button_bar_id = tree.create_node("SavePanel_ButtonBar", button_bar_style, UiNodeData::Container);

        let button_bg_style = Style::new()
            .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 1.0))
            .with_border_color(gg_render::Color::new(0.4, 0.4, 0.4, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_padding(8.0));

        let button_text_style = Style::new().with_font(FontStyle::new().with_size(14.0).with_color(gg_render::Color::WHITE));

        match self.mode {
            SavePanelMode::Save => {
                let save_btn_id = tree.create_node("SavePanel_SaveButton", button_bg_style.clone(), UiNodeData::Container);
                let save_label_id = tree.create_node(
                    "SavePanel_SaveButton_Label",
                    button_text_style.clone(),
                    UiNodeData::Text { content: String::from("Save") },
                );
                tree.add_child(save_btn_id, save_label_id);
                tree.add_child(button_bar_id, save_btn_id);
                self.save_button_node_id = Some(save_btn_id);
            }
            SavePanelMode::Load => {
                let load_btn_id = tree.create_node("SavePanel_LoadButton", button_bg_style.clone(), UiNodeData::Container);
                let load_label_id = tree.create_node(
                    "SavePanel_LoadButton_Label",
                    button_text_style.clone(),
                    UiNodeData::Text { content: String::from("Load") },
                );
                tree.add_child(load_btn_id, load_label_id);
                tree.add_child(button_bar_id, load_btn_id);
                self.load_button_node_id = Some(load_btn_id);
            }
        }

        let delete_btn_id = tree.create_node("SavePanel_DeleteButton", button_bg_style, UiNodeData::Container);
        let delete_label_id = tree.create_node(
            "SavePanel_DeleteButton_Label",
            button_text_style,
            UiNodeData::Text { content: String::from("Delete") },
        );
        tree.add_child(delete_btn_id, delete_label_id);
        tree.add_child(button_bar_id, delete_btn_id);
        self.delete_button_node_id = Some(delete_btn_id);

        tree.add_child(root_id, button_bar_id);

        let content_style = Style::new().with_layout(LayoutStyle::new().with_direction(FlexDirection::Column).with_gap(4.0));

        let content_id = tree.create_node("SavePanel_Content", content_style, UiNodeData::Container);

        for slot_widget in &mut self.slots {
            let slot_id = slot_widget.build(tree)?;
            tree.add_child(content_id, slot_id);
        }

        tree.add_child(root_id, content_id);

        self.node_id = Some(root_id);
        self.content_node_id = Some(content_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            if let Some(node) = tree.get_mut(id) {
                node.style = self.style.clone();
            }
        }

        for slot_widget in &self.slots {
            slot_widget.update(tree);
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }

    fn render_template(&self) -> oak_voc::TemplateNode {
        oak_voc::TemplateNode::text(String::new())
    }

    fn script_setup(&mut self) {}

    fn get_id(&self) -> &str {
        ""
    }

    fn handle_event(&mut self, _event: &gg_ui::GuiEvent, _ctx: &mut gg_ui::EventContext) {}
}

/// 自动存档指示器
///
/// 在自动存档触发时短暂显示后消失的 UI 组件。
pub struct AutoSaveIndicator {
    /// 是否正在显示
    pub is_showing: bool,
    /// 显示持续时间（秒）
    pub display_duration: f32,
    /// 已显示时间（秒）
    pub elapsed: f32,
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl AutoSaveIndicator {
    /// 创建自动存档指示器
    pub fn new() -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.0, 0.0, 0.0, 0.6))
            .with_corner_radius(4.0)
            .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(8.0).with_gap(6.0))
            .with_opacity(0.0);

        Self { is_showing: false, display_duration: 2.0, elapsed: 0.0, style, node_id: None }
    }

    /// 设置显示持续时间
    ///
    /// # 参数
    ///
    /// - `duration` - 持续时间（秒）
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.display_duration = duration;
        self
    }

    /// 触发自动存档指示器显示
    pub fn trigger(&mut self) {
        self.is_showing = true;
        self.elapsed = 0.0;
    }

    /// 更新计时器
    ///
    /// # 参数
    ///
    /// - `delta_secs` - 距上一帧的时间间隔（秒）
    pub fn update_timer(&mut self, delta_secs: f32) {
        if self.is_showing {
            self.elapsed += delta_secs;
            if self.elapsed >= self.display_duration {
                self.is_showing = false;
                self.elapsed = 0.0;
            }
        }
    }

    /// 获取当前是否可见
    pub fn is_visible(&self) -> bool {
        self.is_showing
    }
}

impl Default for AutoSaveIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for AutoSaveIndicator {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node("AutoSaveIndicator", self.style.clone(), UiNodeData::Container);

        let icon_style = Style::new().with_font(FontStyle::new().with_size(14.0).with_color(gg_render::Color::WHITE));

        let icon_id = tree.create_node("AutoSaveIndicator_Icon", icon_style, UiNodeData::Text { content: String::from("💾") });

        let text_style = Style::new().with_font(FontStyle::new().with_size(14.0).with_color(gg_render::Color::WHITE));

        let text_id = tree.create_node(
            "AutoSaveIndicator_Text",
            text_style,
            UiNodeData::Text { content: String::from("Auto Saving...") },
        );

        tree.add_child(root_id, icon_id);
        tree.add_child(root_id, text_id);

        self.node_id = Some(root_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(id) = self.node_id {
            if let Some(node) = tree.get_mut(id) {
                if self.is_showing {
                    let fade_start = self.display_duration * 0.7;
                    let opacity = if self.elapsed < fade_start {
                        1.0
                    }
                    else {
                        let fade_progress = (self.elapsed - fade_start) / (self.display_duration - fade_start);
                        (1.0 - fade_progress).max(0.0)
                    };
                    node.style.opacity = opacity;
                    node.visible = true;
                }
                else {
                    node.visible = false;
                    node.style.opacity = 0.0;
                }
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }

    fn render_template(&self) -> oak_voc::TemplateNode {
        oak_voc::TemplateNode::text(String::new())
    }

    fn script_setup(&mut self) {}

    fn get_id(&self) -> &str {
        ""
    }

    fn handle_event(&mut self, _event: &gg_ui::GuiEvent, _ctx: &mut gg_ui::EventContext) {}
}
