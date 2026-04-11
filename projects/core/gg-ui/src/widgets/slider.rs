use crate::{
    node::{UiNodeData, UiNodeId, UiTree},
    style::{FlexAlign, FlexDirection, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 滑块控件
///
/// 提供可拖动的滑块 UI 元素，支持设置最小值、最大值和当前值。
pub struct Slider {
    /// 最小值
    min: f32,
    /// 最大值
    max: f32,
    /// 当前值
    value: f32,
    /// 值变化回调
    on_change: Option<Box<dyn FnMut(f32) + Send + Sync>>,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
    /// 轨道节点 ID
    track_node_id: Option<UiNodeId>,
    /// 滑块节点 ID
    thumb_node_id: Option<UiNodeId>,
    /// 是否正在拖动
    is_dragging: bool,
}

impl Slider {
    /// 创建滑块控件
    ///
    /// # 参数
    ///
    /// - `min` - 最小值
    /// - `max` - 最大值
    /// - `value` - 初始值
    pub fn new(min: f32, max: f32, value: f32) -> Self {
        let clamped = value.clamp(min, max);
        Self {
            min,
            max,
            value: clamped,
            on_change: None,
            node_id: None,
            track_node_id: None,
            thumb_node_id: None,
            is_dragging: false,
        }
    }

    /// 设置当前值，自动限制在 [min, max] 范围内
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    /// 获取当前值
    pub fn value(&self) -> f32 {
        self.value
    }

    /// 设置值变化回调
    pub fn set_on_change(&mut self, callback: Box<dyn FnMut(f32) + Send + Sync>) {
        self.on_change = Some(callback);
    }

    /// 处理拖动事件
    ///
    /// 根据鼠标位置计算新的值并触发回调。
    ///
    /// # 参数
    ///
    /// - `mouse_x` - 鼠标 X 坐标
    /// - `track_x` - 轨道 X 坐标
    /// - `track_width` - 轨道宽度
    pub fn handle_drag(&mut self, mouse_x: f32, track_x: f32, track_width: f32) {
        if track_width <= 0.0 {
            return;
        }
        let ratio = ((mouse_x - track_x) / track_width).clamp(0.0, 1.0);
        self.value = self.min + ratio * (self.max - self.min);
        if let Some(ref mut cb) = self.on_change {
            cb(self.value);
        }
    }
}

impl Widget for Slider {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_id = tree.create_node(
            "Slider",
            Style::new().with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Row)
                    .with_align_items(FlexAlign::Center)
                    .with_height(SizeValue::Px(24.0)),
            ),
            UiNodeData::Container,
        );

        let track_id = tree.create_node(
            "Slider_Track",
            Style::new()
                .with_background_color(gg_render::Color::new(0.3, 0.3, 0.3, 1.0))
                .with_corner_radius(4.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_direction(FlexDirection::Row)
                        .with_width(SizeValue::Percent(1.0))
                        .with_height(SizeValue::Px(8.0)),
                ),
            UiNodeData::Container,
        );

        let ratio = if self.max > self.min {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.0
        };

        let thumb_id = tree.create_node(
            "Slider_Thumb",
            Style::new()
                .with_background_color(gg_render::Color::new(0.5, 0.7, 1.0, 1.0))
                .with_corner_radius(8.0)
                .with_layout(
                    LayoutStyle::new()
                        .with_width(SizeValue::Px(16.0))
                        .with_height(SizeValue::Px(16.0))
                        .with_margin(ratio * 100.0),
                ),
            UiNodeData::Container,
        );

        tree.add_child(root_id, track_id);
        tree.add_child(track_id, thumb_id);

        self.node_id = Some(root_id);
        self.track_node_id = Some(track_id);
        self.thumb_node_id = Some(thumb_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(thumb_id) = self.thumb_node_id {
            if let Some(track_id) = self.track_node_id {
                let track_width = tree
                    .get(track_id)
                    .and_then(|n| n.layout_result)
                    .map(|r| r.width)
                    .unwrap_or(200.0);

                let ratio = if self.max > self.min {
                    (self.value - self.min) / (self.max - self.min)
                } else {
                    0.0
                };

                let margin_left = ratio * track_width;

                if let Some(node) = tree.get_mut(thumb_id) {
                    node.style.layout.margin = margin_left;
                }
            }
        }
    }

    fn node_id(&self) -> Option<UiNodeId> {
        self.node_id
    }
}
