use crate::{
    gui_event::{EventContext, GuiEvent, MouseButton},
    node::{UiNodeData, UiNodeId, UiTree},
    reactive::Signal,
    style::{FlexDirection, FontStyle, LayoutStyle, SizeValue, Style},
    widget::Widget,
};
use gg_core::GResult;

/// 菜单项
pub enum MenuItem {
    /// 普通菜单项
    Item {
        /// 菜单项标签
        label: String,
        /// 点击回调
        on_click: Option<Box<dyn FnMut() + Send + Sync>>,
    },
    /// 分隔符
    Separator,
    /// 子菜单
    SubMenu {
        /// 子菜单标签
        label: String,
        /// 子菜单项列表
        items: Vec<MenuItem>,
    },
}

/// 右键菜单组件
///
/// 提供右键弹出的上下文菜单 UI 元素，支持普通菜单项、分隔符和子菜单。
pub struct ContextMenu {
    /// 菜单项列表
    pub items: Vec<MenuItem>,
    /// 打开状态
    pub is_open: Signal<bool>,
    /// 显示位置
    pub position: (f32, f32),
    /// 样式
    pub style: Style,
    /// 根节点 ID
    node_id: Option<UiNodeId>,
}

impl ContextMenu {
    /// 创建右键菜单
    ///
    /// # 参数
    ///
    /// - `items` - 菜单项列表
    pub fn new(items: Vec<MenuItem>) -> Self {
        let style = Style::new()
            .with_background_color(gg_render::Color::new(0.2, 0.2, 0.2, 1.0))
            .with_border_color(gg_render::Color::new(0.4, 0.4, 0.4, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_layout(
                LayoutStyle::new()
                    .with_direction(FlexDirection::Column)
                    .with_padding(4.0)
                    .with_gap(2.0)
                    .with_min_width(SizeValue::Px(150.0)),
            )
            .with_font(FontStyle::new());

        Self { items, is_open: Signal::new(false), position: (0.0, 0.0), style, node_id: None }
    }

    /// 设置样式
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// 在指定位置打开菜单
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.position = (x, y);
        self.is_open.set(true);
    }

    /// 关闭菜单
    pub fn close(&mut self) {
        self.is_open.set(false);
    }
}

impl Widget for ContextMenu {
    fn build(&mut self, tree: &mut UiTree) -> GResult<UiNodeId> {
        let root_style = Style::new().with_layout(
            LayoutStyle::new()
                .with_direction(FlexDirection::Column)
                .with_margin_left(self.position.0)
                .with_margin_top(self.position.1),
        );

        let root_id = tree.create_node("ContextMenu", root_style, UiNodeData::Container);

        let container_style = self.style.clone();

        let container_id = tree.create_node("ContextMenu_Container", container_style, UiNodeData::Container);

        for item in &self.items {
            match item {
                MenuItem::Item { label, .. } => {
                    let item_style = Style::new()
                        .with_corner_radius(2.0)
                        .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(6.0))
                        .with_font(self.style.font.clone().unwrap_or_default());

                    let item_id = tree.create_node(
                        format!("ContextMenu_Item({})", label),
                        item_style,
                        UiNodeData::Text { content: label.clone() },
                    );

                    tree.add_child(container_id, item_id);
                }
                MenuItem::Separator => {
                    let sep_style = Style::new().with_background_color(gg_render::Color::new(0.4, 0.4, 0.4, 1.0)).with_layout(
                        LayoutStyle::new().with_direction(FlexDirection::Row).with_height(SizeValue::Px(1.0)).with_padding(4.0),
                    );

                    let sep_id = tree.create_node("ContextMenu_Separator", sep_style, UiNodeData::Container);

                    tree.add_child(container_id, sep_id);
                }
                MenuItem::SubMenu { label, .. } => {
                    let sub_style = Style::new()
                        .with_corner_radius(2.0)
                        .with_layout(LayoutStyle::new().with_direction(FlexDirection::Row).with_padding(6.0))
                        .with_font(self.style.font.clone().unwrap_or_default());

                    let sub_id = tree.create_node(
                        format!("ContextMenu_SubMenu({})", label),
                        sub_style,
                        UiNodeData::Text { content: format!("{} ▶", label) },
                    );

                    tree.add_child(container_id, sub_id);
                }
            }
        }

        tree.add_child(root_id, container_id);

        if let Some(node) = tree.get_mut(root_id) {
            node.visible = *self.is_open.get();
        }

        self.node_id = Some(root_id);

        Ok(root_id)
    }

    fn update(&self, tree: &mut UiTree) {
        if let Some(node_id) = self.node_id {
            if let Some(node) = tree.get_mut(node_id) {
                node.visible = *self.is_open.get();
                node.style.layout.margin_left = self.position.0;
                node.style.layout.margin_top = self.position.1;
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
            GuiEvent::MouseClick { button: MouseButton::Right, x, y } => {
                self.position = (*x, *y);
                *self.is_open.write().unwrap() = true;
            }
            GuiEvent::MouseClick { button: MouseButton::Left, .. } => {
                if *self.is_open.read().unwrap() {
                    *self.is_open.write().unwrap() = false;
                }
            }
            _ => {}
        }
    }
}
