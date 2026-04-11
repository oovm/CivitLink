use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use gg_render::{RenderContext, Renderer};
use gg_ui::{
    EventContext, EventPhase, GuiEvent, GuiRenderer, LayoutEngine, UiRenderer, UiTree, Widget, template_node_to_ui_tree_mapped,
};

/// GUI 渲染器适配器
///
/// 将 GUI 运行时桥接到平台渲染后端，
/// 负责 Widget 的模板转换为 UiTree，
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
    focused_node: Option<gg_ui::UiNodeId>,
    /// UiNodeId 到子组件的映射（根组件通过 process_events 参数传入）
    component_map: HashMap<gg_ui::UiNodeId, Arc<RwLock<dyn Widget>>>,
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
    pub fn focused_node(&self) -> Option<gg_ui::UiNodeId> {
        self.focused_node
    }

    /// 设置聚焦的节点 ID
    ///
    /// 键盘事件（KeyPress / TextInput）将路由到聚焦节点。
    pub fn set_focused_node(&mut self, node_id: Option<gg_ui::UiNodeId>) {
        self.focused_node = node_id;
    }

    /// 对指定坐标执行命中测试，返回最深层可见节点
    ///
    /// 从根节点开始递归遍历，找到包含指定坐标的最前端可见节点。
    /// 如果没有节点命中或树为空，返回 `None`。
    pub fn find_node_at(&self, x: f32, y: f32) -> Option<gg_ui::UiNodeId> {
        let root_id = self.ui_tree.root()?;
        let mut hit: Option<gg_ui::UiNodeId> = None;
        self.hit_test_node(root_id, x, y, &mut hit);
        hit
    }

    /// 递归命中测试辅助方法
    fn hit_test_node(&self, node_id: gg_ui::UiNodeId, x: f32, y: f32, hit: &mut Option<gg_ui::UiNodeId>) {
        let node = match self.ui_tree.get(node_id) {
            Some(n) => n,
            None => return,
        };

        if !node.visible {
            return;
        }

        if let Some(ref layout) = node.layout_result {
            if x >= layout.x && x <= layout.x + layout.width && y >= layout.y && y <= layout.y + layout.height {
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
    pub fn find_node_path(&self, target_id: gg_ui::UiNodeId) -> Vec<gg_ui::UiNodeId> {
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
    fn render(&mut self, component: Arc<dyn Widget>) {
        let template = component.render_template();
        let comp_children = component.children();
        let mut component_map = HashMap::new();

        let tree = template_node_to_ui_tree_mapped(&template, &comp_children, &mut component_map);

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

    fn process_events(&mut self, root: Option<&Arc<RwLock<dyn Widget>>>) {
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
    fn dispatch_event(&self, event: &GuiEvent, root_comp: &Arc<RwLock<dyn Widget>>, path: &[gg_ui::UiNodeId]) {
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
        node_id: gg_ui::UiNodeId,
        root_id: gg_ui::UiNodeId,
        root_comp: &Arc<RwLock<dyn Widget>>,
        ctx: &mut EventContext,
    ) {
        if node_id == root_id {
            if let Ok(mut comp) = root_comp.write() {
                comp.handle_event(event, ctx);
            }
        }
        else if let Some(comp) = self.component_map.get(&node_id) {
            if let Ok(mut c) = comp.write() {
                c.handle_event(event, ctx);
            }
        }
    }
}
