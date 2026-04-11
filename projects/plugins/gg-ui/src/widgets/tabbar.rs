use std::collections::HashMap;

use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexAlign, FlexDirection, FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 标签页项
///
/// 表示标签栏中的单个标签，包含标识、标签文本和是否可关闭。
#[derive(Debug, Clone)]
pub struct TabItem {
    /// 标签页唯一标识
    pub id: String,
    /// 标签页显示文本
    pub label: String,
    /// 是否显示关闭按钮
    pub closable: bool,
}

impl TabItem {
    /// 创建标签页项
    ///
    /// # 参数
    ///
    /// - `id` - 标签页唯一标识
    /// - `label` - 标签页显示文本
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            closable: false,
        }
    }

    /// 设置是否可关闭
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// 标签栏控件
///
/// 提供水平排列的标签页切换 UI 元素，支持标签选择和关闭操作。
pub struct TabBar {
    /// 标签页列表
    pub tabs: Vec<TabItem>,
    /// 当前激活标签页 ID
    pub active_tab_id: Option<String>,
    /// 样式
    pub style: Style,
    /// 标签页切换回调
    pub on_tab_changed: Option<Box<dyn FnMut(&str) + Send + Sync>>,
    /// 标签页关闭回调
    pub on_tab_closed: Option<Box<dyn FnMut(&str) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 标签节点 ID 映射
    tab_node_ids: HashMap<String, UiNodeId>,
}

impl TabBar {
    /// 创建标签栏控件
    ///
    /// # 参数
    ///
    /// - `tabs` - 标签页列表
    pub fn new(tabs: Vec<TabItem>) -> Self {
        let style = Style::new()
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_gap(2.0),
            )
            .with_font(FontStyle::new());

        let active_tab_id = tabs.first().map(|t| t.id.clone());

        Self {
            tabs,
            active_tab_id,
            style,
            on_tab_changed: None,
            on_tab_closed: None,
            node_id: None,
            tab_node_ids: HashMap::new(),
        }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 添加标签页
    pub fn add_tab(&mut self, tab: TabItem) {
        self.tabs.push(tab);
    }

    /// 移除标签页
    pub fn remove_tab(&mut self, id: &str) {
        self.tabs.retain(|t| t.id != id);
        self.tab_node_ids.remove(id);
        if self.active_tab_id.as_deref() == Some(id) {
            self.active_tab_id = self.tabs.first().map(|t| t.id.clone());
        }
    }

    /// 设置激活标签页
    pub fn set_active_tab(&mut self, id: &str) {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_tab_id = Some(id.to_string());
        }
    }

    /// 获取当前激活标签页
    pub fn active_tab(&self) -> Option<&TabItem> {
        self.active_tab_id
            .as_deref()
            .and_then(|id| self.tabs.iter().find(|t| t.id == id))
    }
}

impl Widget for TabBar {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node(
            "TabBar",
            self.style.clone(),
            UiNodeData::Container,
        );

        let mut tab_node_ids = HashMap::new();

        for tab in &self.tabs {
            let is_active = self.active_tab_id.as_deref() == Some(tab.id.as_str());

            let tab_bg = if is_active {
                gg_render::Color::new(0.3, 0.3, 0.3, 1.0)
            } else {
                gg_render::Color::new(0.2, 0.2, 0.2, 1.0)
            };

            let tab_border = if is_active {
                gg_render::Color::new(0.26, 0.52, 0.96, 1.0)
            } else {
                gg_render::Color::new(0.4, 0.4, 0.4, 1.0)
            };

            let tab_style = Style::new()
                .with_background_color(tab_bg)
                .with_border_color(tab_border)
                .with_border_width(if is_active { 2.0 } else { 1.0 })
                .with_corner_radius(4.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_align_items(FlexAlign::Center)
                        .with_gap(6.0)
                        .with_padding(8.0),
                )
                .with_font(self.style.font.clone().unwrap_or_default());

            let tab_id = tree.create_node(
                format!("TabBar_Tab({})", tab.id),
                tab_style,
                UiNodeData::Container,
            );

            let label_style = Style::new().with_font(
                FontStyle::new(),
            );

            let label_id = tree.create_node(
                format!("TabBar_TabLabel({})", tab.id),
                label_style,
                UiNodeData::Text {
                    content: tab.label.clone(),
                },
            );

            tree.add_child(tab_id, label_id);

            if tab.closable {
                let close_style = Style::new().with_font(
                    FontStyle::new().with_size(12.0),
                );

                let close_id = tree.create_node(
                    format!("TabBar_TabClose({})", tab.id),
                    close_style,
                    UiNodeData::Text {
                        content: "×".to_string(),
                    },
                );

                tree.add_child(tab_id, close_id);
            }

            tree.add_child(root_id, tab_id);
            tab_node_ids.insert(tab.id.clone(), tab_id);
        }

        self.node_id = Some(root_id);
        self.tab_node_ids = tab_node_ids;

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        for tab in &self.tabs {
            let is_active = self.active_tab_id.as_deref() == Some(tab.id.as_str());

            let tab_bg = if is_active {
                gg_render::Color::new(0.3, 0.3, 0.3, 1.0)
            } else {
                gg_render::Color::new(0.2, 0.2, 0.2, 1.0)
            };

            let tab_border = if is_active {
                gg_render::Color::new(0.26, 0.52, 0.96, 1.0)
            } else {
                gg_render::Color::new(0.4, 0.4, 0.4, 1.0)
            };

            if let Some(&tab_node_id) = self.tab_node_ids.get(&tab.id) {
                if let Some(node) = tree.get_mut(tab_node_id) {
                    node.style.background_color = Some(tab_bg);
                    node.style.border_color = Some(tab_border);
                    node.style.border_width = if is_active { 2.0 } else { 1.0 };
                }
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
