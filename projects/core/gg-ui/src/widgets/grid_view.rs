use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexDirection, LayoutStyle, Overflow, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 网格视图控件
///
/// 提供可滚动的网格布局，支持虚拟滚动（仅渲染可见单元格）、
/// 单元格选择高亮和滚动到指定项等功能。
pub struct GridView {
    /// 网格列数
    pub columns: usize,
    /// 项目总数
    pub item_count: usize,
    /// 单元格尺寸 `[width, height]`
    pub cell_size: [f32; 2],
    /// 单元格间距
    pub gap: f32,
    /// 可视区域尺寸 `[width, height]`
    pub view_size: [f32; 2],
    /// 当前垂直滚动偏移
    pub scroll_offset_y: f32,
    /// 当前选中的项目索引
    pub selected_index: Option<usize>,
    /// 选中回调
    pub on_select: Option<Box<dyn FnMut(usize) + Send + Sync>>,
    /// 根节点 ID（裁剪容器）
    node_id: Option<UiNodeId>,
    /// 内容容器节点 ID
    content_node_id: Option<UiNodeId>,
    /// 当前渲染的单元格节点 ID 列表
    cell_node_ids: Vec<UiNodeId>,
}

impl GridView {
    /// 创建网格视图控件
    ///
    /// # 参数
    ///
    /// - `columns` - 网格列数
    /// - `item_count` - 项目总数
    /// - `cell_size` - 单元格尺寸 `[width, height]`
    /// - `gap` - 单元格间距
    /// - `view_size` - 可视区域尺寸 `[width, height]`
    pub fn new(columns: usize, item_count: usize, cell_size: [f32; 2], gap: f32, view_size: [f32; 2]) -> Self {
        Self {
            columns,
            item_count,
            cell_size,
            gap,
            view_size,
            scroll_offset_y: 0.0,
            selected_index: None,
            on_select: None,
            node_id: None,
            content_node_id: None,
            cell_node_ids: Vec::new(),
        }
    }

    /// 计算总行数
    pub fn row_count(&self) -> usize {
        if self.columns == 0 {
            return 0;
        }
        (self.item_count + self.columns - 1) / self.columns
    }

    /// 计算第一个可见行的索引
    pub fn first_visible_row(&self) -> usize {
        let row_height = self.cell_size[1] + self.gap;
        if row_height <= 0.0 {
            return 0;
        }
        let row = (self.scroll_offset_y / row_height).floor() as usize;
        row.max(0)
    }

    /// 计算最后一个可见行的索引（含）
    pub fn last_visible_row(&self) -> usize {
        let row_height = self.cell_size[1] + self.gap;
        if row_height <= 0.0 {
            return 0;
        }
        let visible_bottom = self.scroll_offset_y + self.view_size[1];
        let row = (visible_bottom / row_height).ceil() as usize;
        let total_rows = self.row_count();
        if row == 0 {
            0
        } else {
            (row - 1).min(total_rows.saturating_sub(1))
        }
    }

    /// 获取指定索引单元格的位置 `(x, y)`
    ///
    /// 根据单元格的列和行索引计算其在内容区域中的绝对坐标。
    pub fn cell_position(&self, index: usize) -> (f32, f32) {
        if self.columns == 0 {
            return (0.0, 0.0);
        }
        let col = index % self.columns;
        let row = index / self.columns;
        let x = col as f32 * (self.cell_size[0] + self.gap);
        let y = row as f32 * (self.cell_size[1] + self.gap);
        (x, y)
    }

    /// 判断指定索引的单元格是否在可视区域内
    pub fn is_cell_visible(&self, index: usize) -> bool {
        if index >= self.item_count {
            return false;
        }
        let first_row = self.first_visible_row();
        let last_row = self.last_visible_row();
        let row = index / self.columns.max(1);
        row >= first_row && row <= last_row
    }

    /// 处理滚动事件
    ///
    /// 根据垂直方向滚动增量调整滚动偏移，自动限制在内容边界内。
    ///
    /// # 参数
    ///
    /// - `delta_y` - 垂直方向滚动增量
    pub fn handle_scroll(&mut self, delta_y: f32) {
        let new_y = self.scroll_offset_y + delta_y;
        self.scroll_offset_y = new_y.clamp(0.0, self.max_scroll_offset());
    }

    /// 滚动到使指定索引项可见的位置
    ///
    /// # 参数
    ///
    /// - `index` - 目标项目的索引
    pub fn scroll_to_index(&mut self, index: usize) {
        if index >= self.item_count || self.columns == 0 {
            return;
        }
        let row = index / self.columns;
        let row_height = self.cell_size[1] + self.gap;
        let row_top = row as f32 * row_height;
        let row_bottom = row_top + self.cell_size[1];

        if row_top < self.scroll_offset_y {
            self.scroll_offset_y = row_top;
        } else if row_bottom > self.scroll_offset_y + self.view_size[1] {
            self.scroll_offset_y = (row_bottom - self.view_size[1]).max(0.0);
        }
    }

    /// 获取最大滚动偏移量
    pub fn max_scroll_offset(&self) -> f32 {
        let total_rows = self.row_count();
        if total_rows == 0 {
            return 0.0;
        }
        let content_height = total_rows as f32 * (self.cell_size[1] + self.gap);
        (content_height - self.view_size[1]).max(0.0)
    }

    /// 重建可见单元格节点
    ///
    /// 移除旧单元格节点，创建当前可见行中的单元格节点并添加到内容容器中。
    fn rebuild_visible_cells(&mut self, ui_tree: &mut UiTree) {
        if let Some(content_id) = self.content_node_id {
            for &cell_id in &self.cell_node_ids {
                ui_tree.remove_node(cell_id);
            }
            self.cell_node_ids.clear();

            let first_row = self.first_visible_row();
            let last_row = self.last_visible_row();

            for row in first_row..=last_row {
                for col in 0..self.columns {
                    let index = row * self.columns + col;
                    if index >= self.item_count {
                        break;
                    }

                    let (x, y) = self.cell_position(index);
                    let is_selected = self.selected_index == Some(index);

                    let bg_color = if is_selected {
                        gg_render::Color::new(0.2, 0.5, 0.9, 0.6)
                    } else {
                        gg_render::Color::new(0.15, 0.15, 0.15, 0.4)
                    };

                    let cell_style = Style::new()
                        .with_background_color(bg_color)
                        .with_corner_radius(4.0)
                        .with_layout(
                            LayoutStyle::new()
                                .with_width(SizeValue::Px(self.cell_size[0]))
                                .with_height(SizeValue::Px(self.cell_size[1]))
                                .with_margin_left(x)
                                .with_margin_top(y),
                        );

                    let cell_id = ui_tree.create_node("GridView_Cell", cell_style, UiNodeData::Container);
                    ui_tree.add_child(content_id, cell_id);
                    self.cell_node_ids.push(cell_id);
                }
            }
        }
    }
}

impl Widget for GridView {
    fn build(&mut self, ui_tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new().with_overflow(Overflow::Clip).with_layout(
            LayoutStyle::new().with_width(SizeValue::Px(self.view_size[0])).with_height(SizeValue::Px(self.view_size[1])),
        );

        let root_id = ui_tree.create_node("GridView", root_style, UiNodeData::Container);

        let total_rows = self.row_count();
        let content_height = if total_rows > 0 {
            total_rows as f32 * (self.cell_size[1] + self.gap)
        } else {
            0.0
        };
        let content_width = self.columns as f32 * (self.cell_size[0] + self.gap);

        let content_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_direction(FlexDirection::Row)
                .with_wrap(true)
                .with_gap(self.gap)
                .with_width(SizeValue::Px(content_width))
                .with_height(SizeValue::Px(content_height))
                .with_margin_top(-self.scroll_offset_y),
        );

        let content_id = ui_tree.create_node("GridView_Content", content_style, UiNodeData::Container);
        ui_tree.add_child(root_id, content_id);

        self.node_id = Some(root_id);
        self.content_node_id = Some(content_id);
        self.cell_node_ids.clear();

        self.rebuild_visible_cells(ui_tree);

        Ok(root_id)
    }

    fn update(&self, ui_tree: &mut UiTree) {
        if let Some(content_id) = self.content_node_id {
            if let Some(node) = ui_tree.get_mut(content_id) {
                node.style.layout.margin_top = -self.scroll_offset_y;
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
            GuiEvent::MouseClick { y, button, .. } => {
                if *button == MouseButton::Left {
                    let click_y = y + self.scroll_offset_y;
                    let row_height = self.cell_size[1] + self.gap;
                    if row_height <= 0.0 {
                        return;
                    }
                    let row = (click_y / row_height).floor() as usize;
                    if row < self.row_count() {
                        let col = 0;
                        let index = row * self.columns + col;
                        if index < self.item_count {
                            self.selected_index = Some(index);
                            if let Some(callback) = &mut self.on_select {
                                callback(index);
                            }
                        }
                    }
                }
            }
            GuiEvent::Custom { name, data } => {
                if name == "scroll" {
                    if let Some(&delta_y) = data.downcast_ref::<f32>() {
                        self.handle_scroll(delta_y);
                    }
                }
            }
            _ => {}
        }
    }
}
