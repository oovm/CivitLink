use std::collections::HashMap;

use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 下拉选择框控件
///
/// 提供可展开的选项列表，用户可以从中选择一个选项。
pub struct ComboBox {
    /// 选项列表
    pub options: Vec<String>,
    /// 当前选中索引
    pub selected_index: Option<usize>,
    /// 样式
    pub style: Style,
    /// 选中变更回调
    pub on_change: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 下拉列表是否打开
    dropdown_open: bool,
    /// 下拉列表容器节点 ID
    dropdown_node_id: Option<UiNodeId>,
    /// 选项节点 ID 映射
    option_node_ids: Vec<UiNodeId>,
    /// 选中文本节点 ID
    selected_text_node_id: Option<UiNodeId>,
    /// 箭头指示器节点 ID
    arrow_node_id: Option<UiNodeId>,
    /// 选项节点 ID 到索引的映射
    option_index_map: HashMap<UiNodeId, usize>,
}

impl ComboBox {
    /// 创建下拉选择框控件
    ///
    /// # 参数
    ///
    /// - `options` - 选项文本列表
    pub fn new(options: Vec<String>) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 1.0))
            .with_border_color(gg_render::Color::new(0.5, 0.5, 0.5, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_padding(8.0),
            )
            .with_font(FontStyle::new());

        Self {
            options,
            selected_index: None,
            style,
            on_change: None,
            node_id: None,
            dropdown_open: false,
            dropdown_node_id: None,
            option_node_ids: Vec::new(),
            selected_text_node_id: None,
            arrow_node_id: None,
            option_index_map: HashMap::new(),
        }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 设置初始选中索引
    pub fn with_selected(mut self, index: usize) -> Self {
        if index < self.options.len() {
            self.selected_index = Some(index);
        }
        self
    }

    /// 设置选中索引
    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = Some(index);
        }
    }

    /// 获取选中索引
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// 获取选中选项文本
    pub fn selected_text(&self) -> Option<&str> {
        self.selected_index.and_then(|i| self.options.get(i).map(|s| s.as_str()))
    }

    /// 下拉列表是否打开
    pub fn is_dropdown_open(&self) -> bool {
        self.dropdown_open
    }

    /// 切换下拉列表的展开/收起状态
    pub fn toggle_dropdown(&mut self) {
        self.dropdown_open = !self.dropdown_open;
    }

    /// 关闭下拉列表
    pub fn close_dropdown(&mut self) {
        self.dropdown_open = false;
    }
}

impl Widget for ComboBox {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node(
            "ComboBox",
            self.style.clone(),
            UiNodeData::Container,
        );

        let display_row_style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_justify_content(FlexAlign::SpaceBetween),
            );

        let display_row_id = tree.create_node(
            "ComboBox_DisplayRow",
            display_row_style,
            UiNodeData::Container,
        );

        let selected_text = self
            .selected_index
            .and_then(|i| self.options.get(i))
            .cloned()
            .unwrap_or_default();

        let selected_text_style = Style::new().with_font(
            self.style.font.clone().unwrap_or_default(),
        );

        let selected_text_id = tree.create_node(
            "ComboBox_SelectedText",
            selected_text_style,
            UiNodeData::Text {
                content: selected_text,
            },
        );

        let arrow_style = Style::new().with_font(
            FontStyle::new().with_size(12.0),
        );

        let arrow_id = tree.create_node(
            "ComboBox_Arrow",
            arrow_style,
            UiNodeData::Text {
                content: "▼".to_string(),
            },
        );

        tree.add_child(display_row_id, selected_text_id);
        tree.add_child(display_row_id, arrow_id);
        tree.add_child(root_id, display_row_id);

        let mut option_node_ids = Vec::new();
        let mut option_index_map = HashMap::new();

        if self.dropdown_open {
            let dropdown_style = Style::new()
                .with_background_color(gg_render::Color::new(0.25, 0.25, 0.25, 1.0))
                .with_border_color(gg_render::Color::new(0.4, 0.4, 0.4, 1.0))
                .with_border_width(1.0)
                .with_corner_radius(4.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Column)
                        .with_padding(4.0)
                        .with_gap(2.0),
                );

            let dropdown_id = tree.create_node(
                "ComboBox_Dropdown",
                dropdown_style,
                UiNodeData::Container,
            );

            for (i, option) in self.options.iter().enumerate() {
                let is_selected = self.selected_index == Some(i);
                let bg = if is_selected {
                    gg_render::Color::new(0.26, 0.52, 0.96, 1.0)
                } else {
                    gg_render::Color::new(0.0, 0.0, 0.0, 0.0)
                };

                let option_style = Style::new()
                    .with_background_color(bg)
                    .with_corner_radius(2.0)
                    .with_layout(
                        LayoutStyle::new()
                            .with_direction(FlexDirection::Row)
                            .with_padding(6.0),
                    )
                    .with_font(self.style.font.clone().unwrap_or_default());

                let option_id = tree.create_node(
                    format!("ComboBox_Option({})", option),
                    option_style,
                    UiNodeData::Text {
                        content: option.clone(),
                    },
                );

                tree.add_child(dropdown_id, option_id);
                option_node_ids.push(option_id);
                option_index_map.insert(option_id, i);
            }

            tree.add_child(root_id, dropdown_id);
            self.dropdown_node_id = Some(dropdown_id);
        }

        self.node_id = Some(root_id);
        self.selected_text_node_id = Some(selected_text_id);
        self.arrow_node_id = Some(arrow_id);
        self.option_node_ids = option_node_ids;
        self.option_index_map = option_index_map;

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(selected_text_id) = self.selected_text_node_id {
            let selected_text = self
                .selected_index
                .and_then(|i| self.options.get(i))
                .cloned()
                .unwrap_or_default();

            if let Some(node) = tree.get_mut(selected_text_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = selected_text;
                }
            }
        }

        if let Some(arrow_id) = self.arrow_node_id {
            let arrow_text = if self.dropdown_open { "▲" } else { "▼" };
            if let Some(node) = tree.get_mut(arrow_id) {
                if let UiNodeData::Text { ref mut content } = node.data {
                    *content = arrow_text.to_string();
                }
            }
        }

        if let Some(dropdown_id) = self.dropdown_node_id {
            if let Some(dropdown_node) = tree.get_mut(dropdown_id) {
                dropdown_node.visible = self.dropdown_open;
            }
        }

        for &option_id in &self.option_node_ids {
            if let Some(&index) = self.option_index_map.get(&option_id) {
                let is_selected = self.selected_index == Some(index);
                let bg = if is_selected {
                    gg_render::Color::new(0.26, 0.52, 0.96, 1.0)
                } else {
                    gg_render::Color::new(0.0, 0.0, 0.0, 0.0)
                };
                if let Some(node) = tree.get_mut(option_id) {
                    node.style.background_color = Some(bg);
                }
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
