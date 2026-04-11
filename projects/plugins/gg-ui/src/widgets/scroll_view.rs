use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexDirection, LayoutStyle, Overflow, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 滚动视图控件
///
/// 提供可滚动的内容区域，通过裁剪溢出内容和偏移内容位置实现滚动效果，
/// 支持滚动条显示和鼠标滚轮/触摸滚动。
pub struct ScrollView {
    /// 可视区域尺寸 `[width, height]`
    pub view_size: [f32; 2],
    /// 内容尺寸 `[width, height]`
    pub content_size: [f32; 2],
    /// 滚动偏移 `[x, y]`
    pub scroll_offset: [f32; 2],
    /// 滚动条宽度
    pub scroll_bar_width: f32,
    /// 是否显示垂直滚动条
    pub show_vertical_scroll_bar: bool,
    /// 是否显示水平滚动条
    pub show_horizontal_scroll_bar: bool,
    /// 是否正在拖动滚动条
    pub is_dragging_scroll_bar: bool,
    /// 根节点 ID（裁剪容器）
    node_id: Option<UiNodeId>,
    /// 内容容器节点 ID
    content_node_id: Option<UiNodeId>,
    /// 垂直滚动条轨道节点 ID
    v_scroll_track_node_id: Option<UiNodeId>,
    /// 垂直滚动条滑块节点 ID
    v_scroll_thumb_node_id: Option<UiNodeId>,
    /// 水平滚动条轨道节点 ID
    h_scroll_track_node_id: Option<UiNodeId>,
    /// 水平滚动条滑块节点 ID
    h_scroll_thumb_node_id: Option<UiNodeId>,
}

impl ScrollView {
    /// 创建滚动视图控件
    ///
    /// # 参数
    ///
    /// - `view_size` - 可视区域尺寸 `[width, height]`
    /// - `content_size` - 内容尺寸 `[width, height]`
    pub fn new(view_size: [f32; 2], content_size: [f32; 2]) -> Self {
        let show_vertical_scroll_bar = content_size[1] > view_size[1];
        let show_horizontal_scroll_bar = content_size[0] > view_size[0];

        Self {
            view_size,
            content_size,
            scroll_offset: [0.0, 0.0],
            scroll_bar_width: 12.0,
            show_vertical_scroll_bar,
            show_horizontal_scroll_bar,
            is_dragging_scroll_bar: false,
            node_id: None,
            content_node_id: None,
            v_scroll_track_node_id: None,
            v_scroll_thumb_node_id: None,
            h_scroll_track_node_id: None,
            h_scroll_thumb_node_id: None,
        }
    }

    /// 设置滚动偏移，自动限制在内容边界内
    ///
    /// # 参数
    ///
    /// - `offset` - 目标滚动偏移 `[x, y]`
    pub fn set_scroll_offset(&mut self, offset: [f32; 2]) {
        let max_x = (self.content_size[0] - self.view_size[0]).max(0.0);
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[0] = offset[0].clamp(0.0, max_x);
        self.scroll_offset[1] = offset[1].clamp(0.0, max_y);
    }

    /// 获取当前滚动偏移
    pub fn scroll_offset(&self) -> [f32; 2] {
        self.scroll_offset
    }

    /// 处理鼠标滚轮滚动
    ///
    /// 根据滚动增量调整垂直方向的滚动偏移，自动限制在内容边界内。
    ///
    /// # 参数
    ///
    /// - `delta_y` - 垂直方向滚动增量
    pub fn handle_mouse_scroll(&mut self, delta_y: f32) {
        let new_y = self.scroll_offset[1] + delta_y;
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[1] = new_y.clamp(0.0, max_y);
    }

    /// 处理触摸滚动
    ///
    /// 根据触摸滑动增量同时调整水平和垂直方向的滚动偏移。
    ///
    /// # 参数
    ///
    /// - `delta_x` - 水平方向滚动增量
    /// - `delta_y` - 垂直方向滚动增量
    pub fn handle_touch_scroll(&mut self, delta_x: f32, delta_y: f32) {
        let new_x = self.scroll_offset[0] + delta_x;
        let new_y = self.scroll_offset[1] + delta_y;
        let max_x = (self.content_size[0] - self.view_size[0]).max(0.0);
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[0] = new_x.clamp(0.0, max_x);
        self.scroll_offset[1] = new_y.clamp(0.0, max_y);
    }

    /// 滚动到顶部
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset[1] = 0.0;
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self) {
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[1] = max_y;
    }

    /// 是否滚动到顶部
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset[1] <= 0.0
    }

    /// 是否滚动到底部
    pub fn is_at_bottom(&self) -> bool {
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        self.scroll_offset[1] >= max_y
    }

    /// 获取垂直滚动比例
    ///
    /// 返回 0.0（顶部）到 1.0（底部）之间的值，表示当前滚动位置。
    pub fn vertical_scroll_ratio(&self) -> f32 {
        let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
        if max_y <= 0.0 {
            0.0
        } else {
            self.scroll_offset[1] / max_y
        }
    }
}

impl Widget for ScrollView {
    fn build(&mut self, ui_tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new()
            .with_overflow(Overflow::Clip)
            .with_layout(
                LayoutStyle::new()
                    .with_width(SizeValue::Px(self.view_size[0]))
                    .with_height(SizeValue::Px(self.view_size[1])),
            );

        let root_id = ui_tree.create_node("ScrollView", root_style, UiNodeData::Container);

        let content_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_width(SizeValue::Px(self.content_size[0]))
                .with_height(SizeValue::Px(self.content_size[1]))
                .with_margin_left(-self.scroll_offset[0])
                .with_margin_top(-self.scroll_offset[1]),
        );

        let content_id = ui_tree.create_node("ScrollView_Content", content_style, UiNodeData::Container);
        ui_tree.add_child(root_id, content_id);

        if self.show_vertical_scroll_bar {
            let track_style = Style::new()
                .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 0.6))
                .with_corner_radius(self.scroll_bar_width / 2.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Column)
                        .with_width(SizeValue::Px(self.scroll_bar_width))
                        .with_height(SizeValue::Px(self.view_size[1]))
                        .with_margin_left(self.view_size[0] - self.scroll_bar_width),
                );

            let track_id = ui_tree.create_node(
                "ScrollView_VTrack",
                track_style,
                UiNodeData::Container,
            );

            let ratio = self.vertical_scroll_ratio();
            let thumb_height = if self.content_size[1] > 0.0 {
                (self.view_size[1] / self.content_size[1] * self.view_size[1]).max(20.0).min(self.view_size[1])
            } else {
                self.view_size[1]
            };

            let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
            let thumb_top = if max_y > 0.0 {
                ratio * (self.view_size[1] - thumb_height)
            } else {
                0.0
            };

            let thumb_style = Style::new()
                .with_background_color(gg_render::Color::new(0.5, 0.5, 0.5, 0.8))
                .with_corner_radius(self.scroll_bar_width / 2.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_width(SizeValue::Px(self.scroll_bar_width))
                        .with_height(SizeValue::Px(thumb_height))
                        .with_margin_top(thumb_top),
                );

            let thumb_id = ui_tree.create_node(
                "ScrollView_VThumb",
                thumb_style,
                UiNodeData::Container,
            );

            ui_tree.add_child(track_id, thumb_id);
            ui_tree.add_child(root_id, track_id);

            self.v_scroll_track_node_id = Some(track_id);
            self.v_scroll_thumb_node_id = Some(thumb_id);
        }

        if self.show_horizontal_scroll_bar {
            let track_style = Style::new()
                .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 0.6))
                .with_corner_radius(self.scroll_bar_width / 2.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_width(SizeValue::Px(self.view_size[0]))
                        .with_height(SizeValue::Px(self.scroll_bar_width))
                        .with_margin_top(self.view_size[1] - self.scroll_bar_width),
                );

            let track_id = ui_tree.create_node(
                "ScrollView_HTrack",
                track_style,
                UiNodeData::Container,
            );

            let thumb_width = if self.content_size[0] > 0.0 {
                (self.view_size[0] / self.content_size[0] * self.view_size[0]).max(20.0).min(self.view_size[0])
            } else {
                self.view_size[0]
            };

            let max_x = (self.content_size[0] - self.view_size[0]).max(0.0);
            let ratio_x = if max_x > 0.0 {
                self.scroll_offset[0] / max_x
            } else {
                0.0
            };
            let thumb_left = ratio_x * (self.view_size[0] - thumb_width);

            let thumb_style = Style::new()
                .with_background_color(gg_render::Color::new(0.5, 0.5, 0.5, 0.8))
                .with_corner_radius(self.scroll_bar_width / 2.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_width(SizeValue::Px(thumb_width))
                        .with_height(SizeValue::Px(self.scroll_bar_width))
                        .with_margin_left(thumb_left),
                );

            let thumb_id = ui_tree.create_node(
                "ScrollView_HThumb",
                thumb_style,
                UiNodeData::Container,
            );

            ui_tree.add_child(track_id, thumb_id);
            ui_tree.add_child(root_id, track_id);

            self.h_scroll_track_node_id = Some(track_id);
            self.h_scroll_thumb_node_id = Some(thumb_id);
        }

        self.node_id = Some(root_id);
        self.content_node_id = Some(content_id);
        Ok(root_id)
    }

    fn update(&self, ui_tree: &mut UiTree) {
        if let Some(content_id) = self.content_node_id {
            if let Some(node) = ui_tree.get_mut(content_id) {
                node.style.layout.margin_left = -self.scroll_offset[0];
                node.style.layout.margin_top = -self.scroll_offset[1];
            }
        }

        if let Some(thumb_id) = self.v_scroll_thumb_node_id {
            let thumb_height = if self.content_size[1] > 0.0 {
                (self.view_size[1] / self.content_size[1] * self.view_size[1]).max(20.0).min(self.view_size[1])
            } else {
                self.view_size[1]
            };

            let max_y = (self.content_size[1] - self.view_size[1]).max(0.0);
            let thumb_top = if max_y > 0.0 {
                self.vertical_scroll_ratio() * (self.view_size[1] - thumb_height)
            } else {
                0.0
            };

            if let Some(node) = ui_tree.get_mut(thumb_id) {
                node.style.layout.height = SizeValue::Px(thumb_height);
                node.style.layout.margin_top = thumb_top;
            }
        }

        if let Some(thumb_id) = self.h_scroll_thumb_node_id {
            let thumb_width = if self.content_size[0] > 0.0 {
                (self.view_size[0] / self.content_size[0] * self.view_size[0]).max(20.0).min(self.view_size[0])
            } else {
                self.view_size[0]
            };

            let max_x = (self.content_size[0] - self.view_size[0]).max(0.0);
            let ratio_x = if max_x > 0.0 {
                self.scroll_offset[0] / max_x
            } else {
                0.0
            };
            let thumb_left = ratio_x * (self.view_size[0] - thumb_width);

            if let Some(node) = ui_tree.get_mut(thumb_id) {
                node.style.layout.width = SizeValue::Px(thumb_width);
                node.style.layout.margin_left = thumb_left;
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
