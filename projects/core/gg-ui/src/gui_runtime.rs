use std::sync::{Arc, RwLock};

use crate::{
    gui_event::{EventContext, EventPhase, GuiEvent},
    widget::{DirtyFlag, Widget},
};

/// GUI 渲染器特质
pub trait GuiRenderer: Send + Sync {
    /// 渲染组件树
    fn render(&mut self, component: Arc<dyn Widget>);

    /// 处理事件，接收根组件用于三阶段事件分发
    fn process_events(&mut self, root: Option<&Arc<RwLock<dyn Widget>>>);

    /// 更新渲染器
    fn update(&mut self);

    /// 设置视口大小
    fn set_viewport_size(&mut self, width: u32, height: u32);
}

/// GUI 运行时，采用事件驱动模型
pub struct GuiRuntime {
    /// 渲染器
    renderer: Arc<RwLock<dyn GuiRenderer>>,
    /// 根组件
    root_component: Option<Arc<RwLock<dyn Widget>>>,
}

impl GuiRuntime {
    /// 创建新的 GUI 运行时
    pub fn new<T: GuiRenderer + 'static>(renderer: T) -> Self {
        Self { renderer: Arc::new(RwLock::new(renderer)), root_component: None }
    }

    /// 设置根组件
    pub fn set_root_component(&mut self, component: Arc<RwLock<dyn Widget>>) {
        self.root_component = Some(component);
    }

    /// 接收外部事件，标记相关节点为脏
    pub fn process_event(&mut self, event: &GuiEvent) {
        if let Some(component) = &self.root_component {
            let mut ctx = EventContext::new(EventPhase::AtTarget);
            if let Ok(mut comp) = component.write() {
                comp.handle_event(event, &mut ctx);
            }
            self.propagate_dirty_from_children(component);
        }
    }

    /// 仅在存在脏节点时执行重绘，无脏节点则直接返回
    pub fn commit_render(&mut self) {
        if !self.has_dirty_components() {
            return;
        }

        self.update();

        self.clear_all_dirty();
    }

    /// 手动标记指定节点为脏
    pub fn mark_dirty(&mut self, component_id: &str, flag: DirtyFlag) {
        if let Some(root) = &self.root_component {
            self.mark_dirty_recursive(root, component_id, flag);
        }
    }

    /// 检查是否有脏组件
    pub fn has_dirty_components(&self) -> bool {
        if let Some(component) = &self.root_component {
            if let Ok(comp) = component.read() {
                if comp.is_dirty() {
                    return true;
                }
            }
            self.has_dirty_children_recursive(component)
        }
        else {
            false
        }
    }

    /// 执行更新
    pub fn update(&mut self) {
        if let Ok(mut renderer) = self.renderer.write() {
            renderer.process_events(self.root_component.as_ref());
        }

        if let Some(component) = &self.root_component {
            if let Ok(mut comp) = component.write() {
                comp.on_update();
            }
        }

        if let Some(component) = &self.root_component {
            if let Ok(comp) = component.read() {
                let cached_id = comp.get_id().to_string();
                if let Ok(mut renderer) = self.renderer.write() {
                    let arc_comp: Arc<dyn Widget> = Arc::new(ComponentWrapper { inner: Arc::clone(component), cached_id });
                    renderer.render(arc_comp);
                }
            }
        }

        if let Ok(mut renderer) = self.renderer.write() {
            renderer.update();
        }
    }

    /// 递归标记脏节点
    fn mark_dirty_recursive(&self, component: &Arc<RwLock<dyn Widget>>, target_id: &str, flag: DirtyFlag) -> bool {
        let mut found = false;
        if let Ok(mut comp) = component.write() {
            if comp.get_id() == target_id {
                comp.mark_dirty(flag);
                found = true;
            }
        }

        let children = if let Ok(comp) = component.read() { comp.children() } else { Vec::new() };

        for child in &children {
            if self.mark_dirty_recursive(child, target_id, flag) {
                found = true;
            }
        }

        if found {
            if let Ok(mut comp) = component.write() {
                if comp.get_id() != target_id {
                    comp.on_update();
                }
            }
        }

        found
    }

    /// 从子节点向上传播脏标记到父节点
    fn propagate_dirty_from_children(&self, component: &Arc<RwLock<dyn Widget>>) {
        let children = if let Ok(comp) = component.read() { comp.children() } else { Vec::new() };

        let mut child_has_dirty = false;
        for child in &children {
            self.propagate_dirty_from_children(child);
            if let Ok(comp) = child.read() {
                if comp.is_dirty() {
                    child_has_dirty = true;
                }
            }
        }

        if child_has_dirty {
            if let Ok(mut comp) = component.write() {
                comp.on_update();
            }
        }
    }

    /// 递归检查子组件是否有脏标记
    fn has_dirty_children_recursive(&self, component: &Arc<RwLock<dyn Widget>>) -> bool {
        let children = if let Ok(comp) = component.read() { comp.children() } else { Vec::new() };

        for child in &children {
            if let Ok(comp) = child.read() {
                if comp.is_dirty() {
                    return true;
                }
            }
            if self.has_dirty_children_recursive(child) {
                return true;
            }
        }

        false
    }

    /// 清除所有组件的脏标记
    fn clear_all_dirty(&self) {
        if let Some(component) = &self.root_component {
            self.clear_dirty_recursive(component);
        }
    }

    /// 递归清除脏标记
    fn clear_dirty_recursive(&self, component: &Arc<RwLock<dyn Widget>>) {
        if let Ok(mut comp) = component.write() {
            comp.clear_dirty();
        }

        let children = if let Ok(comp) = component.read() { comp.children() } else { Vec::new() };

        for child in &children {
            self.clear_dirty_recursive(child);
        }
    }
}

/// 组件包装器，用于将 RwLock 包装的组件转换为 Arc<dyn Widget>
pub struct ComponentWrapper {
    /// 内部组件
    pub inner: Arc<RwLock<dyn Widget>>,
    /// 缓存的组件 ID
    pub cached_id: String,
}

impl Widget for ComponentWrapper {
    fn render_template(&self) -> oak_voc::TemplateNode {
        if let Ok(comp) = self.inner.read() { comp.render_template() } else { oak_voc::TemplateNode::text(String::new()) }
    }

    fn script_setup(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.script_setup();
        }
    }

    fn get_style(&self) -> Option<&str> {
        None
    }

    fn get_id(&self) -> &str {
        &self.cached_id
    }

    fn handle_event(&mut self, event: &GuiEvent, ctx: &mut EventContext) {
        if let Ok(mut comp) = self.inner.write() {
            comp.handle_event(event, ctx);
        }
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Widget>>> {
        if let Ok(comp) = self.inner.read() { comp.children() } else { Vec::new() }
    }

    fn on_mount(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_mount();
        }
    }

    fn on_update(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_update();
        }
    }

    fn on_cleanup(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.on_cleanup();
        }
    }

    fn is_dirty(&self) -> bool {
        if let Ok(comp) = self.inner.read() { comp.is_dirty() } else { false }
    }

    fn clear_dirty(&mut self) {
        if let Ok(mut comp) = self.inner.write() {
            comp.clear_dirty();
        }
    }

    fn get_dirty_flags(&self) -> DirtyFlag {
        if let Ok(comp) = self.inner.read() { comp.get_dirty_flags() } else { DirtyFlag::NONE }
    }

    fn mark_dirty(&mut self, flag: DirtyFlag) {
        if let Ok(mut comp) = self.inner.write() {
            comp.mark_dirty(flag);
        }
    }

    fn build(&mut self, tree: &mut crate::node::UiTree) -> gg_core::GResult<crate::node::UiNodeId> {
        if let Ok(mut comp) = self.inner.write() {
            comp.build(tree)
        }
        else {
            Err(gg_error::GError::new("Failed to acquire write lock"))
        }
    }

    fn update(&self, tree: &mut crate::node::UiTree) {
        if let Ok(comp) = self.inner.read() {
            comp.update(tree);
        }
    }

    fn node_id(&self) -> Option<crate::node::UiNodeId> {
        if let Ok(comp) = self.inner.read() { comp.node_id() } else { None }
    }
}

/// 平台特定的 GUI 工厂
pub trait GuiFactory {
    /// 创建 GUI 渲染器
    fn create_renderer(&self) -> Arc<dyn GuiRenderer>;

    /// 创建基础组件
    fn create_component(&self, component_type: &str) -> Arc<RwLock<dyn Widget>>;
}
