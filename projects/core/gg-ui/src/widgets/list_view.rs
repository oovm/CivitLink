use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexDirection, LayoutStyle, Overflow, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 列表视图控件
///
/// 提供虚拟滚动的列表布局，仅渲染可视区域内的列表项，
/// 支持垂直和水平方向、项目选择和滚动定位。
pub struct ListView {
    /// 列表方向（垂直或水平）
    pub direction: FlexDirection,
    /// 总项目数
    pub item_count: usize,
    /// 每个项目的尺寸 `[width, height]`
    pub item_size: [f32; 2],
    /// 可视区域尺寸 `[width, height]`
    pub view_size: [f32; 2],
    /// 当前沿主轴的滚动偏移
    pub scroll_offset: f32,
    /// 当前选中项索引
    pub selected_index: Option<usize>,
    /// 项目选中回调
    pub on_select: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 根节点 ID（裁剪容器）
    node_id: Option<UiNodeId>,
    /// 内容容器节点 ID
    content_node_id: Option<UiNodeId>,
    /// 当前渲染的项目节点 ID 列表
    item_node_ids: Vec<UiNodeId>,
}

impl ListView {
    /// 创建列表视图控件
    ///
    /// # 参数
    ///
    /// - `direction` - 列表方向（垂直或水平）
    /// - `item_count` - 总项目数
    /// - `item_size` - 每个项目的尺寸 `[width, height]`
    /// - `view_size` - 可视区域尺寸 `[width, height]`
    pub fn new(
        direction: FlexDirection,
        item_count: usize,
        item_size: [f32; 2],
        view_size: [f32; 2],
    ) -> Self {
        Self {
            direction,
            item_count,
            item_size,
            view_size,
            scroll_offset: 0.0,
            selected_index: None,
            on_select: None,
            node_id: None,
            content_node_id: None,
            item_node_ids: Vec::new(),
        }
    }

    /// 获取第一个可见项目的索引
    pub fn first_visible_index(&self) -> usize {
        let main_size = self.main_item_size();
        if main_size <= 0.0 {
            return 0;
        }
        let index = (self.scroll_offset / main_size) as usize;
        index.min(self.item_count.saturating_sub(1))
    }

    /// 获取最后一个可见项目的索引
    pub fn last_visible_index(&self) -> usize {
        let main_size = self.main_item_size();
        if main_size <= 0.0 {
            return 0;
        }
        let view_main = self.main_view_size();
        let index = ((self.scroll_offset + view_main) / main_size) as usize;
        index.min(self.item_count.saturating_sub(1))
    }

    /// 获取最大滚动偏移量
    pub fn max_scroll_offset(&self) -> f32 {
        let total = self.total_content_size();
        let view = self.main_view_size();
        (total - view).max(0.0)
    }

    /// 判断指定索引的项目是否在可视区域内
    pub fn is_item_visible(&self, index: usize) -> bool {
        if index >= self.item_count {
            return false;
        }
        index >= self.first_visible_index() && index <= self.last_visible_index()
    }

    /// 处理滚动，根据增量调整滚动偏移
    ///
    /// # 参数
    ///
    /// - `delta` - 滚动增量（正值向下/向右滚动，负值向上/向左滚动）
    pub fn handle_scroll(&mut self, delta: f32) {
        let new_offset = self.scroll_offset + delta;
        self.scroll_offset = new_offset.clamp(0.0, self.max_scroll_offset());
    }

    /// 滚动到指定索引，使该项目可见
    ///
    /// # 参数
    ///
    /// - `index` - 目标项目索引
    pub fn scroll_to_index(&mut self, index: usize) {
        if index >= self.item_count {
            return;
        }
        let main_size = self.main_item_size();
        let view_main = self.main_view_size();
        let item_start = index as f32 * main_size;
        let item_end = item_start + main_size;

        if item_start < self.scroll_offset {
            self.scroll_offset = item_start;
        } else if item_end > self.scroll_offset + view_main {
            self.scroll_offset = item_end - view_main;
        }

        self.scroll_offset = self.scroll_offset.clamp(0.0, self.max_scroll_offset());
    }

    /// 获取主轴方向的项目尺寸
    fn main_item_size(&self) -> f32 {
        match self.direction {
            FlexDirection::Column => self.item_size[1],
            FlexDirection::Row => self.item_size[0],
        }
    }

    /// 获取主轴方向的可视区域尺寸
    fn main_view_size(&self) -> f32 {
        match self.direction {
            FlexDirection::Column => self.view_size[1],
            FlexDirection::Row => self.view_size[0],
        }
    }

    /// 获取内容总尺寸（沿主轴）
    fn total_content_size(&self) -> f32 {
        self.item_count as f32 * self.main_item_size()
    }

    /// 根据全局坐标计算点击的项目索引
    fn index_at_position(&self, x: f32, y: f32) -> Option<usize> {
        let (main_pos, cross_pos) = match self.direction {
            FlexDirection::Column => (y + self.scroll_offset, x),
            FlexDirection::Row => (x + self.scroll_offset, y),
        };

        let main_size = self.main_item_size();
        let cross_size = match self.direction {
            FlexDirection::Column => self.item_size[0],
            FlexDirection::Row => self.item_size[1],
        };

        if cross_pos < 0.0 || cross_pos > cross_size {
            return None;
        }

        let index = (main_pos / main_size) as usize;
        if index < self.item_count {
            Some(index)
        } else {
            None
        }
    }

    /// 创建单个列表项的样式
    fn item_style(&self, index: usize) -> Style {
        let is_selected = self.selected_index == Some(index);
        let bg_color = if is_selected {
            gg_render::Color::new(0.2, 0.4, 0.8, 0.6)
        } else {
            gg_render::Color::new(0.15, 0.15, 0.15, 0.4)
        };

        let main_size = self.main_item_size();
        let (width, height) = match self.direction {
            FlexDirection::Column => (SizeValue::Px(self.item_size[0]), SizeValue::Px(main_size)),
            FlexDirection::Row => (SizeValue::Px(main_size), SizeValue::Px(self.item_size[1])),
        };

        Style::new()
            .with_background_color(bg_color)
            .with_layout(
                LayoutStyle::new()
                    .with_width(width)
                    .with_height(height),
            )
    }

    /// 重建可视区域内的列表项节点
    fn rebuild_visible_items(&mut self, ui_tree: &mut UiTree) {
        if let Some(content_id) = self.content_node_id {
            for &item_id in &self.item_node_ids {
                ui_tree.remove_node(item_id);
            }
            self.item_node_ids.clear();

            let first = self.first_visible_index();
            let last = self.last_visible_index();
            let main_size = self.main_item_size();

            for index in first..=last {
                let offset = index as f32 * main_size - self.scroll_offset;
                let style = self.item_style(index);

                let item_id = ui_tree.create_node(
                    format!("ListView_Item_{}", index),
                    style,
                    UiNodeData::Container,
                );

                match self.direction {
                    FlexDirection::Column => {
                        if let Some(node) = ui_tree.get_mut(item_id) {
                            node.style.layout.margin_top = offset;
                        }
                    }
                    FlexDirection::Row => {
                        if let Some(node) = ui_tree.get_mut(item_id) {
                            node.style.layout.margin_left = offset;
                        }
                    }
                }

                ui_tree.add_child(content_id, item_id);
                self.item_node_ids.push(item_id);
            }
        }
    }
}

impl Widget for ListView {
    fn build(&mut self, ui_tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new()
            .with_overflow(Overflow::Clip)
            .with_layout(
                LayoutStyle::new()
                    .with_width(SizeValue::Px(self.view_size[0]))
                    .with_height(SizeValue::Px(self.view_size[1])),
            );

        let root_id = ui_tree.create_node("ListView", root_style, UiNodeData::Container);

        let content_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_width(SizeValue::Px(self.view_size[0]))
                .with_height(SizeValue::Px(self.view_size[1])),
        );

        let content_id = ui_tree.create_node("ListView_Content", content_style, UiNodeData::Container);
        ui_tree.add_child(root_id, content_id);

        self.node_id = Some(root_id);
        self.content_node_id = Some(content_id);

        self.rebuild_visible_items(ui_tree);

        Ok(root_id)
    }

    fn update(&self, ui_tree: &mut UiTree) {
        if let Some(content_id) = self.content_node_id {
            if let Some(node) = ui_tree.get_mut(content_id) {
                match self.direction {
                    FlexDirection::Column => {
                        node.style.layout.margin_top = -self.scroll_offset;
                    }
                    FlexDirection::Row => {
                        node.style.layout.margin_left = -self.scroll_offset;
                    }
                }
            }
        }

        for &item_id in &self.item_node_ids {
            if let Some(node) = ui_tree.get_mut(item_id) {
                if let Some(label) = node.label.strip_prefix("ListView_Item_") {
                    if let Ok(index) = label.parse::<usize>() {
                        let is_selected = self.selected_index == Some(index);
                        node.style.background_color = if is_selected {
                            Some(gg_render::Color::new(0.2, 0.4, 0.8, 0.6))
                        } else {
                            Some(gg_render::Color::new(0.15, 0.15, 0.15, 0.4))
                        };
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

    fn handle_event(&mut self, event: &GuiEvent, _ctx: &mut EventContext) {
        match event {
            GuiEvent::MouseClick { x, y, button } => {
                if *button == MouseButton::Left {
                    if let Some(index) = self.index_at_position(*x, *y) {
                        self.selected_index = Some(index);
                        if let Some(ref mut callback) = self.on_select {
                            callback(index);
                        }
                    }
                }
            }
            GuiEvent::Custom { name, data } => {
                if name == "scroll" {
                    if let Some(&delta) = data.downcast_ref::<f32>() {
                        self.handle_scroll(delta);
                    }
                }
            }
            _ => {}
        }
    }
}
