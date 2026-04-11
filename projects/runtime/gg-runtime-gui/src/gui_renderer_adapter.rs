use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use gg_render::{Color, RenderContext, Renderer};
use gg_ui::{FontStyle, LayoutEngine, LayoutStyle, Overflow, SizeValue, Style, UiNodeData, UiNodeId, UiRenderer, UiTree};

use crate::{EventContext, EventPhase, GuiEvent, GuiRenderer, TemplateNode, VxComponent};

/// GUI 渲染器适配器
///
/// 将 GUI 运行时桥接到平台渲染后端，
/// 负责将 VxComponent 的模板转换为 UiTree，
/// 再通过 UiRenderer 生成 DrawCommand 提交给 Renderer。
///
/// 同时提供事件桥接功能：平台输入事件通过 `push_event` 进入待处理队列，
/// `process_events` 对事件进行命中测试后按照 DOM 标准的
/// 捕获-目标-冒泡三阶段模型分发到组件树。
pub struct GuiRendererAdapter {
    /// 平台渲染器引用
    renderer: Rc<RefCell<dyn Renderer>>,
    /// UI 节点树
    ui_tree: UiTree,
    /// 当前视口宽度
    viewport_width: u32,
    /// 当前视口高度
    viewport_height: u32,
    /// 待处理的事件队列
    pending_events: Vec<GuiEvent>,
    /// 已处理的事件队列
    processed_events: Vec<GuiEvent>,
    /// 当前聚焦的节点 ID
    focused_node: Option<UiNodeId>,
    /// UiNodeId 到子组件的映射（根组件通过 process_events 参数传入）
    component_map: HashMap<UiNodeId, Arc<RwLock<dyn VxComponent>>>,
}

unsafe impl Send for GuiRendererAdapter {}

unsafe impl Sync for GuiRendererAdapter {}

impl GuiRendererAdapter {
    /// 创建新的 GUI 渲染器适配器
    ///
    /// 默认视口尺寸为 800×600。
    pub fn new(renderer: Rc<RefCell<dyn Renderer>>) -> Self {
        Self {
            renderer,
            ui_tree: UiTree::new(),
            viewport_width: 800,
            viewport_height: 600,
            pending_events: Vec::new(),
            processed_events: Vec::new(),
            focused_node: None,
            component_map: HashMap::new(),
        }
    }

    /// 获取当前视口宽度
    pub fn viewport_width(&self) -> u32 {
        self.viewport_width
    }

    /// 获取当前视口高度
    pub fn viewport_height(&self) -> u32 {
        self.viewport_height
    }

    /// 获取 UI 树的不可变引用
    pub fn ui_tree(&self) -> &UiTree {
        &self.ui_tree
    }

    /// 将平台输入事件推入待处理队列
    ///
    /// 事件将在下一次 `process_events` 调用时被处理。
    pub fn push_event(&mut self, event: GuiEvent) {
        self.pending_events.push(event);
    }

    /// 取回所有已处理的事件
    ///
    /// 调用后内部队列会被清空，调用者负责将事件分发到组件。
    pub fn drain_events(&mut self) -> Vec<GuiEvent> {
        std::mem::take(&mut self.processed_events)
    }

    /// 获取当前聚焦的节点 ID
    pub fn focused_node(&self) -> Option<UiNodeId> {
        self.focused_node
    }

    /// 设置聚焦的节点 ID
    ///
    /// 键盘事件（KeyPress / TextInput）将路由到聚焦节点。
    pub fn set_focused_node(&mut self, node_id: Option<UiNodeId>) {
        self.focused_node = node_id;
    }

    /// 对指定坐标执行命中测试，返回最深层可见节点
    ///
    /// 从根节点开始递归遍历，找到包含指定坐标的最前端可见节点。
    /// 如果没有节点命中或树为空，返回 `None`。
    pub fn find_node_at(&self, x: f32, y: f32) -> Option<UiNodeId> {
        let root_id = self.ui_tree.root()?;
        let mut hit: Option<UiNodeId> = None;
        self.hit_test_node(root_id, x, y, &mut hit);
        hit
    }

    /// 递归命中测试辅助方法
    fn hit_test_node(&self, node_id: UiNodeId, x: f32, y: f32, hit: &mut Option<UiNodeId>) {
        let node = match self.ui_tree.get(node_id) {
            Some(n) => n,
            None => return,
        };

        if !node.visible {
            return;
        }

        if let Some(ref layout) = node.layout_result {
            if x >= layout.x
                && x <= layout.x + layout.width
                && y >= layout.y
                && y <= layout.y + layout.height
            {
                *hit = Some(node_id);
            }
        }

        let children = node.children.clone();
        for &child_id in &children {
            self.hit_test_node(child_id, x, y, hit);
        }
    }

    /// 构建从根节点到指定节点的路径
    ///
    /// 返回从根到目标的 UiNodeId 向量（含两端）。
    /// 如果指定节点不存在或树为空，返回空向量。
    pub fn find_node_path(&self, target_id: UiNodeId) -> Vec<UiNodeId> {
        let mut path = vec![target_id];
        let mut current = target_id;
        while let Some(node) = self.ui_tree.get(current) {
            match node.parent {
                Some(parent_id) => {
                    path.push(parent_id);
                    current = parent_id;
                }
                None => break,
            }
        }
        path.reverse();
        path
    }
}

impl GuiRenderer for GuiRendererAdapter {
    fn render(&mut self, component: Arc<dyn VxComponent>) {
        let template = component.render_template();
        let mut tree = UiTree::new();
        let mut component_map = HashMap::new();

        let comp_children = component.children();
        if let Some(root_id) = convert_node_mapped(&template, &mut tree, &comp_children, &mut component_map) {
            tree.set_root(root_id);
        }

        self.ui_tree = tree;
        self.component_map = component_map;

        LayoutEngine::compute(&mut self.ui_tree, self.viewport_width as f32, self.viewport_height as f32);

        let mut render_context = RenderContext::new(self.viewport_width, self.viewport_height);
        UiRenderer::render(&self.ui_tree, &mut render_context);

        if let Ok(mut renderer) = self.renderer.try_borrow_mut() {
            let _ = renderer.begin_frame();
            let _ = renderer.draw(&render_context);
            let _ = renderer.end_frame();
            let _ = renderer.present();
        }
    }

    fn process_events(&mut self, root: Option<&Arc<RwLock<dyn VxComponent>>>) {
        let events = std::mem::take(&mut self.pending_events);
        for event in events {
            let target_id = match &event {
                GuiEvent::MouseClick { x, y, .. } => self.find_node_at(*x, *y),
                GuiEvent::MouseMove { x, y } => self.find_node_at(*x, *y),
                GuiEvent::KeyPress { .. } | GuiEvent::TextInput { .. } => self.focused_node,
                GuiEvent::Custom { .. } => None,
            };

            if let (Some(target_id), Some(root_comp)) = (target_id, root) {
                let path = self.find_node_path(target_id);
                self.dispatch_event(&event, root_comp, &path);
            }

            self.processed_events.push(event);
        }
    }

    fn update(&mut self) {}

    fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
        if let Ok(mut renderer) = self.renderer.try_borrow_mut() {
            renderer.resize(width, height);
        }
    }
}

impl GuiRendererAdapter {
    /// 按照 DOM 标准三阶段模型分发事件
    ///
    /// 依次执行捕获阶段（根→目标父）、目标阶段、冒泡阶段（目标父→根），
    /// 任何阶段调用 `stop_propagation` 后立即停止后续分发。
    fn dispatch_event(
        &self,
        event: &GuiEvent,
        root_comp: &Arc<RwLock<dyn VxComponent>>,
        path: &[UiNodeId],
    ) {
        if path.is_empty() {
            return;
        }

        let root_id = match self.ui_tree.root() {
            Some(id) => id,
            None => return,
        };

        for &node_id in &path[..path.len() - 1] {
            let mut ctx = EventContext::new(EventPhase::Capturing);
            self.dispatch_to_node(event, node_id, root_id, root_comp, &mut ctx);
            if ctx.stop_propagation {
                return;
            }
        }

        {
            let target_id = path[path.len() - 1];
            let mut ctx = EventContext::new(EventPhase::AtTarget);
            self.dispatch_to_node(event, target_id, root_id, root_comp, &mut ctx);
            if ctx.stop_propagation {
                return;
            }
        }

        for &node_id in path[..path.len() - 1].iter().rev() {
            let mut ctx = EventContext::new(EventPhase::Bubbling);
            self.dispatch_to_node(event, node_id, root_id, root_comp, &mut ctx);
            if ctx.stop_propagation {
                return;
            }
        }
    }

    /// 将事件分发到指定 UiNodeId 对应的组件
    fn dispatch_to_node(
        &self,
        event: &GuiEvent,
        node_id: UiNodeId,
        root_id: UiNodeId,
        root_comp: &Arc<RwLock<dyn VxComponent>>,
        ctx: &mut EventContext,
    ) {
        if node_id == root_id {
            if let Ok(mut comp) = root_comp.write() {
                comp.handle_event(event, ctx);
            }
        } else if let Some(comp) = self.component_map.get(&node_id) {
            if let Ok(mut c) = comp.write() {
                c.handle_event(event, ctx);
            }
        }
    }
}

/// 将 TemplateNode 转换为 UiTree
///
/// 递归遍历模板节点树，将每个节点映射为 UiNode 并构建 UiTree 结构。
pub fn template_node_to_ui_tree(node: &TemplateNode) -> UiTree {
    let mut tree = UiTree::new();
    if let Some(root_id) = convert_node(node, &mut tree) {
        tree.set_root(root_id);
    }
    tree
}

fn convert_node(node: &TemplateNode, tree: &mut UiTree) -> Option<gg_ui::UiNodeId> {
    match node {
        TemplateNode::Text(text) => {
            if text.is_empty() {
                return None;
            }
            let style = Style::new().with_font(FontStyle::new());
            let data = UiNodeData::Text { content: text.clone() };
            let id = tree.create_node("text", style, data);
            Some(id)
        }
        TemplateNode::Element { tag, attributes, children } => {
            let (style, node_id_attr, _node_class) = extract_attributes(attributes, tag);
            let data = match tag.as_str() {
                "Text" => {
                    let content = extract_text_content(children);
                    UiNodeData::Text { content }
                }
                "Image" => UiNodeData::Image { texture_id: None, size: None },
                _ => UiNodeData::Container,
            };
            let label = node_id_attr.unwrap_or_else(|| tag.clone());
            let id = tree.create_node(label, style, data);
            for child in children {
                if let Some(child_id) = convert_node(child, tree) {
                    tree.add_child(id, child_id);
                }
            }
            Some(id)
        }
    }
}

/// 将 TemplateNode 转换为 UiTree，同时构建 UiNodeId 到组件的映射
///
/// 与 `convert_node` 类似，但额外跟踪每个 Element 节点对应的子组件，
/// 将映射关系记录到 `component_map` 中。
fn convert_node_mapped(
    node: &TemplateNode,
    tree: &mut UiTree,
    component_children: &[Arc<RwLock<dyn VxComponent>>],
    component_map: &mut HashMap<UiNodeId, Arc<RwLock<dyn VxComponent>>>,
) -> Option<gg_ui::UiNodeId> {
    match node {
        TemplateNode::Text(text) => {
            if text.is_empty() {
                return None;
            }
            let style = Style::new().with_font(FontStyle::new());
            let data = UiNodeData::Text { content: text.clone() };
            let id = tree.create_node("text", style, data);
            Some(id)
        }
        TemplateNode::Element { tag, attributes, children } => {
            let (style, node_id_attr, _node_class) = extract_attributes(attributes, tag);
            let data = match tag.as_str() {
                "Text" => {
                    let content = extract_text_content(children);
                    UiNodeData::Text { content }
                }
                "Image" => UiNodeData::Image { texture_id: None, size: None },
                _ => UiNodeData::Container,
            };
            let label = node_id_attr.unwrap_or_else(|| tag.clone());
            let id = tree.create_node(label, style, data);

            let mut comp_child_idx = 0;
            for child in children {
                let child_id = match child {
                    TemplateNode::Element { .. } => {
                        if comp_child_idx < component_children.len() {
                            let child_comp = &component_children[comp_child_idx];
                            let grand_children: Vec<Arc<RwLock<dyn VxComponent>>> =
                                match child_comp.read() {
                                    Ok(guard) => guard.children(),
                                    Err(_) => Vec::new(),
                                };
                            let cid =
                                convert_node_mapped(child, tree, &grand_children, component_map);
                            if let Some(cid) = cid {
                                component_map.insert(cid, child_comp.clone());
                            }
                            comp_child_idx += 1;
                            cid
                        } else {
                            convert_node(child, tree)
                        }
                    }
                    TemplateNode::Text(_) => convert_node(child, tree),
                };
                if let Some(cid) = child_id {
                    tree.add_child(id, cid);
                }
            }
            Some(id)
        }
    }
}

fn extract_attributes(
    attributes: &[(String, String)],
    tag: &str,
) -> (Style, Option<String>, Option<String>) {
    let mut style = default_style_for_tag(tag);
    let mut node_id = None;
    let mut node_class = None;
    for (key, value) in attributes {
        match key.as_str() {
            "style" => {
                let inline = parse_inline_style(value);
                style = merge_styles(style, inline);
            }
            "id" => {
                node_id = Some(value.clone());
            }
            "class" => {
                node_class = Some(value.clone());
            }
            "direction" | "orientation" => {
                let dir = match value.as_str() {
                    "row" | "horizontal" => gg_ui::FlexDirection::Row,
                    _ => gg_ui::FlexDirection::Column,
                };
                style.layout.direction = dir;
            }
            "gap" => {
                if let Ok(g) = value.parse::<f32>() {
                    style.layout.gap = g;
                }
            }
            "padding" => {
                let px = value.trim().trim_end_matches("px").trim().parse().unwrap_or(0.0);
                style.layout.padding = px;
            }
            _ => {}
        }
    }
    (style, node_id, node_class)
}

/// 根据标签名返回默认样式
fn default_style_for_tag(tag: &str) -> Style {
    match tag {
        "Button" => Style::new()
            .with_background_color(Color::new(0.2, 0.2, 0.2, 1.0))
            .with_border_color(Color::new(0.5, 0.5, 0.5, 1.0))
            .with_border_width(1.0)
            .with_corner_radius(4.0)
            .with_font(FontStyle::new())
            .with_layout(LayoutStyle::new().with_padding(6.0)),
        "Panel" => Style::new()
            .with_background_color(Color::new(0.15, 0.15, 0.15, 1.0))
            .with_border_color(Color::new(0.3, 0.3, 0.3, 1.0))
            .with_border_width(1.0)
            .with_layout(LayoutStyle::new().with_padding(8.0)),
        "Input" => Style::new()
            .with_background_color(Color::new(0.1, 0.1, 0.1, 1.0))
            .with_border_color(Color::new(0.4, 0.4, 0.4, 1.0))
            .with_border_width(1.0)
            .with_font(FontStyle::new())
            .with_layout(LayoutStyle::new().with_padding(4.0)),
        "ScrollView" => Style::new()
            .with_overflow(Overflow::Clip)
            .with_layout(LayoutStyle::new().with_direction(gg_ui::FlexDirection::Column)),
        "Layout" => Style::new()
            .with_layout(LayoutStyle::new().with_direction(gg_ui::FlexDirection::Column)),
        "Stack" => Style::new()
            .with_layout(LayoutStyle::new().with_direction(gg_ui::FlexDirection::Column)),
        "Text" => Style::new().with_font(FontStyle::new()),
        _ => Style::new(),
    }
}

fn merge_styles(base: Style, overlay: Style) -> Style {
    let mut result = base;
    if overlay.background_color.is_some() {
        result.background_color = overlay.background_color;
    }
    if overlay.border_color.is_some() {
        result.border_color = overlay.border_color;
    }
    if overlay.border_width > 0.0 {
        result.border_width = overlay.border_width;
    }
    if overlay.corner_radius > 0.0 {
        result.corner_radius = overlay.corner_radius;
    }
    if overlay.font.is_some() {
        result.font = overlay.font;
    }
    if overlay.layout.padding != 0.0 {
        result.layout.padding = overlay.layout.padding;
    }
    if overlay.layout.margin != 0.0 {
        result.layout.margin = overlay.layout.margin;
    }
    match overlay.layout.width {
        SizeValue::Px(_) => result.layout.width = overlay.layout.width,
        _ => {}
    }
    match overlay.layout.height {
        SizeValue::Px(_) => result.layout.height = overlay.layout.height,
        _ => {}
    }
    result
}

fn extract_text_content(children: &[TemplateNode]) -> String {
    let mut result = String::new();
    for child in children {
        if let TemplateNode::Text(text) = child {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(text);
        }
    }
    result
}

/// 解析内联样式字符串为 Style
///
/// 支持格式: "key: value; key: value;"
///
/// 已知属性映射:
/// - "color" → 字体颜色
/// - "background-color" / "background" → 背景色
/// - "font-size" → 字体大小
/// - "padding" → 内边距
/// - "margin" → 外边距
/// - "border" → 边框（格式: "1px solid red"）
/// - "width" / "height" → 尺寸
pub fn parse_inline_style(style: &str) -> Style {
    let mut result = Style::new();
    for part in style.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut kv = part.splitn(2, ':');
        let key = kv.next().unwrap_or("").trim();
        let value = kv.next().unwrap_or("").trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        apply_style_property(&mut result, key, value);
    }
    result
}

fn apply_style_property(style: &mut Style, key: &str, value: &str) {
    match key {
        "color" => {
            let color = parse_color(value);
            let font = style.font.take().unwrap_or_else(FontStyle::new);
            style.font = Some(font.with_color(color));
        }
        "background-color" | "background" => {
            style.background_color = Some(parse_color(value));
        }
        "font-size" => {
            let size = parse_px_value(value);
            let font = style.font.take().unwrap_or_else(FontStyle::new);
            style.font = Some(font.with_size(size));
        }
        "padding" => {
            style.layout.padding = parse_px_value(value);
        }
        "margin" => {
            style.layout.margin = parse_px_value(value);
        }
        "border" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if !parts.is_empty() {
                style.border_width = parse_px_value(parts[0]);
            }
            if parts.len() >= 3 {
                style.border_color = Some(parse_color(parts[2]));
            }
        }
        "width" => {
            style.layout.width = SizeValue::Px(parse_px_value(value));
        }
        "height" => {
            style.layout.height = SizeValue::Px(parse_px_value(value));
        }
        _ => {}
    }
}

/// 解析颜色字符串为 Color
///
/// 支持命名颜色（red, green, blue, white, black, transparent）
/// 和十六进制颜色（#RGB, #RRGGBB, #RRGGBBAA）。
pub fn parse_color(value: &str) -> Color {
    match value.trim().to_lowercase().as_str() {
        "red" => Color::RED,
        "green" => Color::GREEN,
        "blue" => Color::BLUE,
        "white" => Color::WHITE,
        "black" => Color::BLACK,
        "transparent" => Color::TRANSPARENT,
        hex if hex.starts_with('#') => parse_hex_color(hex),
        _ => Color::WHITE,
    }
}

fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0) as f32 / 255.0;
            Color::new(r, g, b, 1.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            Color::new(r, g, b, 1.0)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(0) as f32 / 255.0;
            Color::new(r, g, b, a)
        }
        _ => Color::WHITE,
    }
}

fn parse_px_value(value: &str) -> f32 {
    value.trim().trim_end_matches("px").trim().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Text;
    use gg_render::{SurfaceInfo, TextureId};
    use std::path::Path;

    struct MockRenderer {
        surface_info: SurfaceInfo,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                surface_info: SurfaceInfo::new(800, 600, "test"),
            }
        }
    }

    impl Renderer for MockRenderer {
        fn begin_frame(&mut self) -> gg_core::GResult<()> {
            Ok(())
        }

        fn end_frame(&mut self) -> gg_core::GResult<()> {
            Ok(())
        }

        fn draw(&mut self, _context: &RenderContext) -> gg_core::GResult<()> {
            Ok(())
        }

        fn present(&mut self) -> gg_core::GResult<()> {
            Ok(())
        }

        fn load_texture(&mut self, _path: &Path) -> gg_core::GResult<TextureId> {
            Ok(TextureId::new(1))
        }

        fn resize(&mut self, width: u32, height: u32) {
            self.surface_info.width = width;
            self.surface_info.height = height;
        }

        fn surface_info(&self) -> &SurfaceInfo {
            &self.surface_info
        }
    }

    #[test]
    fn test_template_node_to_ui_tree_text() {
        let node = TemplateNode::Text("Hello".to_string());
        let tree = template_node_to_ui_tree(&node);
        assert!(tree.root().is_some());
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert_eq!(root.label, "text");
        if let UiNodeData::Text { ref content } = root.data {
            assert_eq!(content, "Hello");
        } else {
            panic!("Expected Text node data");
        }
    }

    #[test]
    fn test_template_node_to_ui_tree_empty_text() {
        let node = TemplateNode::Text(String::new());
        let tree = template_node_to_ui_tree(&node);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_template_node_to_ui_tree_layout() {
        let node = TemplateNode::Element {
            tag: "Layout".to_string(),
            attributes: vec![("id".to_string(), "main-layout".to_string())],
            children: vec![TemplateNode::Text("Child".to_string())],
        };
        let tree = template_node_to_ui_tree(&node);
        assert!(tree.root().is_some());
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert_eq!(root.label, "main-layout");
        assert!(matches!(root.data, UiNodeData::Container));
        assert_eq!(root.children.len(), 1);
    }

    #[test]
    fn test_template_node_to_ui_tree_text_element() {
        let node = TemplateNode::Element {
            tag: "Text".to_string(),
            attributes: vec![("id".to_string(), "title".to_string())],
            children: vec![TemplateNode::Text("Hello World".to_string())],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        if let UiNodeData::Text { ref content } = root.data {
            assert_eq!(content, "Hello World");
        } else {
            panic!("Expected Text node data");
        }
    }

    #[test]
    fn test_template_node_to_ui_tree_button() {
        let node = TemplateNode::Element {
            tag: "Button".to_string(),
            attributes: vec![("id".to_string(), "btn".to_string())],
            children: vec![],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert!(matches!(root.data, UiNodeData::Container));
        assert!(root.style.background_color.is_some());
        assert!(root.style.border_color.is_some());
    }

    #[test]
    fn test_template_node_to_ui_tree_image() {
        let node = TemplateNode::Element {
            tag: "Image".to_string(),
            attributes: vec![("id".to_string(), "img".to_string())],
            children: vec![],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        if let UiNodeData::Image { ref texture_id, ref size } = root.data {
            assert!(texture_id.is_none());
            assert!(size.is_none());
        } else {
            panic!("Expected Image node data");
        }
    }

    #[test]
    fn test_template_node_to_ui_tree_nested() {
        let node = TemplateNode::Element {
            tag: "Layout".to_string(),
            attributes: vec![],
            children: vec![
                TemplateNode::Element {
                    tag: "Text".to_string(),
                    attributes: vec![],
                    children: vec![TemplateNode::Text("Hello".to_string())],
                },
                TemplateNode::Element {
                    tag: "Button".to_string(),
                    attributes: vec![],
                    children: vec![],
                },
            ],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn test_template_node_to_ui_tree_unknown_tag() {
        let node = TemplateNode::Element {
            tag: "CustomWidget".to_string(),
            attributes: vec![],
            children: vec![],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert!(matches!(root.data, UiNodeData::Container));
    }

    #[test]
    fn test_parse_inline_style_color() {
        let style = parse_inline_style("color: red");
        assert!(style.font.is_some());
        let font = style.font.as_ref().unwrap();
        assert_eq!(font.color, Color::RED);
    }

    #[test]
    fn test_parse_inline_style_background() {
        let style = parse_inline_style("background-color: blue");
        assert_eq!(style.background_color, Some(Color::BLUE));
    }

    #[test]
    fn test_parse_inline_style_background_shorthand() {
        let style = parse_inline_style("background: green");
        assert_eq!(style.background_color, Some(Color::GREEN));
    }

    #[test]
    fn test_parse_inline_style_font_size() {
        let style = parse_inline_style("font-size: 24px");
        assert!(style.font.is_some());
        let font = style.font.as_ref().unwrap();
        assert_eq!(font.size, 24.0);
    }

    #[test]
    fn test_parse_inline_style_padding() {
        let style = parse_inline_style("padding: 10px");
        assert_eq!(style.layout.padding, 10.0);
    }

    #[test]
    fn test_parse_inline_style_margin() {
        let style = parse_inline_style("margin: 5px");
        assert_eq!(style.layout.margin, 5.0);
    }

    #[test]
    fn test_parse_inline_style_border() {
        let style = parse_inline_style("border: 2px solid red");
        assert_eq!(style.border_width, 2.0);
        assert_eq!(style.border_color, Some(Color::RED));
    }

    #[test]
    fn test_parse_inline_style_width_height() {
        let style = parse_inline_style("width: 100px; height: 200px");
        assert_eq!(style.layout.width, SizeValue::Px(100.0));
        assert_eq!(style.layout.height, SizeValue::Px(200.0));
    }

    #[test]
    fn test_parse_inline_style_multiple() {
        let style = parse_inline_style("color: white; background-color: black; font-size: 16px; padding: 8px");
        assert!(style.font.is_some());
        let font = style.font.as_ref().unwrap();
        assert_eq!(font.color, Color::WHITE);
        assert_eq!(font.size, 16.0);
        assert_eq!(style.background_color, Some(Color::BLACK));
        assert_eq!(style.layout.padding, 8.0);
    }

    #[test]
    fn test_parse_inline_style_empty() {
        let style = parse_inline_style("");
        assert!(style.background_color.is_none());
        assert!(style.font.is_none());
    }

    #[test]
    fn test_parse_inline_style_unknown_property() {
        let style = parse_inline_style("unknown: value");
        assert!(style.background_color.is_none());
    }

    #[test]
    fn test_parse_color_named() {
        assert_eq!(parse_color("red"), Color::RED);
        assert_eq!(parse_color("green"), Color::GREEN);
        assert_eq!(parse_color("blue"), Color::BLUE);
        assert_eq!(parse_color("white"), Color::WHITE);
        assert_eq!(parse_color("black"), Color::BLACK);
        assert_eq!(parse_color("transparent"), Color::TRANSPARENT);
    }

    #[test]
    fn test_parse_color_hex_6() {
        let color = parse_color("#ff0000");
        assert_eq!(color, Color::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_parse_color_hex_3() {
        let color = parse_color("#f00");
        assert_eq!(color, Color::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_parse_color_hex_8() {
        let color = parse_color("#ff000080");
        assert_eq!(color, Color::new(1.0, 0.0, 0.0, 128.0 / 255.0));
    }

    #[test]
    fn test_parse_color_unknown() {
        assert_eq!(parse_color("unknown"), Color::WHITE);
    }

    #[test]
    fn test_gui_renderer_adapter_new() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let adapter = GuiRendererAdapter::new(renderer);
        assert_eq!(adapter.viewport_width(), 800);
        assert_eq!(adapter.viewport_height(), 600);
    }

    #[test]
    fn test_gui_renderer_adapter_set_viewport_size() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.set_viewport_size(1024, 768);
        assert_eq!(adapter.viewport_width(), 1024);
        assert_eq!(adapter.viewport_height(), 768);
    }

    #[test]
    fn test_gui_renderer_adapter_render() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        let text = Text::new("test", "Hello");
        let component: Arc<dyn VxComponent> = Arc::new(text);
        adapter.render(component);
    }

    #[test]
    fn test_gui_renderer_adapter_process_events_noop() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert!(events.is_empty());
    }

    #[test]
    fn test_gui_renderer_adapter_update_noop() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.update();
    }

    #[test]
    fn test_push_and_process_mouse_click() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::MouseClick {
            x: 10.0,
            y: 20.0,
            button: crate::MouseButton::Left,
        });
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            GuiEvent::MouseClick { x, y, button: crate::MouseButton::Left } if *x == 10.0 && *y == 20.0
        ));
    }

    #[test]
    fn test_push_and_process_key_press() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::KeyPress {
            key: crate::Key::Enter,
            modifiers: crate::KeyModifiers::default(),
        });
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            GuiEvent::KeyPress { key: crate::Key::Enter, .. }
        ));
    }

    #[test]
    fn test_push_and_process_text_input() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::TextInput {
            text: "hello".to_string(),
        });
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            GuiEvent::TextInput { text } if text == "hello"
        ));
    }

    #[test]
    fn test_push_and_process_mouse_move() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::MouseMove { x: 50.0, y: 60.0 });
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            GuiEvent::MouseMove { x, y } if *x == 50.0 && *y == 60.0
        ));
    }

    #[test]
    fn test_multiple_events_batch_processing() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::MouseClick {
            x: 1.0,
            y: 2.0,
            button: crate::MouseButton::Left,
        });
        adapter.push_event(GuiEvent::MouseMove { x: 3.0, y: 4.0 });
        adapter.push_event(GuiEvent::TextInput {
            text: "a".to_string(),
        });
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_drain_events_clears_queue() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::MouseClick {
            x: 1.0,
            y: 2.0,
            button: crate::MouseButton::Left,
        });
        adapter.process_events(None);
        let first = adapter.drain_events();
        assert_eq!(first.len(), 1);
        let second = adapter.drain_events();
        assert!(second.is_empty());
    }

    #[test]
    fn test_process_events_drains_pending() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.push_event(GuiEvent::MouseClick {
            x: 1.0,
            y: 2.0,
            button: crate::MouseButton::Left,
        });
        adapter.process_events(None);
        adapter.push_event(GuiEvent::MouseMove { x: 5.0, y: 6.0 });
        adapter.process_events(None);
        let events = adapter.drain_events();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_focused_node_default_none() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let adapter = GuiRendererAdapter::new(renderer);
        assert!(adapter.focused_node().is_none());
    }

    #[test]
    fn test_set_focused_node() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);
        adapter.set_focused_node(Some(42));
        assert_eq!(adapter.focused_node(), Some(42));
        adapter.set_focused_node(None);
        assert!(adapter.focused_node().is_none());
    }

    #[test]
    fn test_find_node_at_empty_tree() {
        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let adapter = GuiRendererAdapter::new(renderer);
        assert!(adapter.find_node_at(10.0, 10.0).is_none());
    }

    #[test]
    fn test_find_node_at_with_layout() {
        use gg_ui::LayoutResult;

        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);

        let style = Style::new();
        let node_id = adapter.ui_tree.create_node("button", style, UiNodeData::Container);
        adapter.ui_tree.set_root(node_id);
        if let Some(node) = adapter.ui_tree.get_mut(node_id) {
            node.layout_result = Some(LayoutResult::new(0.0, 0.0, 100.0, 50.0));
        }

        assert_eq!(adapter.find_node_at(50.0, 25.0), Some(node_id));
        assert_eq!(adapter.find_node_at(150.0, 25.0), None);
        assert_eq!(adapter.find_node_at(50.0, 75.0), None);
    }

    #[test]
    fn test_find_node_at_invisible_node() {
        use gg_ui::LayoutResult;

        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);

        let style = Style::new();
        let node_id = adapter.ui_tree.create_node("hidden", style, UiNodeData::Container);
        adapter.ui_tree.set_root(node_id);
        if let Some(node) = adapter.ui_tree.get_mut(node_id) {
            node.layout_result = Some(LayoutResult::new(0.0, 0.0, 100.0, 50.0));
            node.visible = false;
        }

        assert!(adapter.find_node_at(50.0, 25.0).is_none());
    }

    #[test]
    fn test_find_node_at_nested() {
        use gg_ui::LayoutResult;

        let renderer = Rc::new(RefCell::new(MockRenderer::new()));
        let mut adapter = GuiRendererAdapter::new(renderer);

        let parent_style = Style::new();
        let child_style = Style::new();
        let parent_id = adapter.ui_tree.create_node("panel", parent_style, UiNodeData::Container);
        let child_id = adapter.ui_tree.create_node("button", child_style, UiNodeData::Container);
        adapter.ui_tree.add_child(parent_id, child_id);
        adapter.ui_tree.set_root(parent_id);

        if let Some(node) = adapter.ui_tree.get_mut(parent_id) {
            node.layout_result = Some(LayoutResult::new(0.0, 0.0, 200.0, 200.0));
        }
        if let Some(node) = adapter.ui_tree.get_mut(child_id) {
            node.layout_result = Some(LayoutResult::new(10.0, 10.0, 80.0, 40.0));
        }

        assert_eq!(adapter.find_node_at(50.0, 30.0), Some(child_id));
        assert_eq!(adapter.find_node_at(5.0, 5.0), Some(parent_id));
    }

    #[test]
    fn test_template_node_style_attribute() {
        let node = TemplateNode::Element {
            tag: "Text".to_string(),
            attributes: vec![
                ("id".to_string(), "styled".to_string()),
                ("style".to_string(), "color: red; font-size: 24px".to_string()),
            ],
            children: vec![TemplateNode::Text("Styled Text".to_string())],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert!(root.style.font.is_some());
        let font = root.style.font.as_ref().unwrap();
        assert_eq!(font.color, Color::RED);
        assert_eq!(font.size, 24.0);
    }

    #[test]
    fn test_template_node_class_attribute() {
        let node = TemplateNode::Element {
            tag: "Layout".to_string(),
            attributes: vec![
                ("class".to_string(), "container".to_string()),
                ("id".to_string(), "main".to_string()),
            ],
            children: vec![],
        };
        let tree = template_node_to_ui_tree(&node);
        let root = tree.get(tree.root().unwrap()).unwrap();
        assert_eq!(root.label, "main");
    }
}
