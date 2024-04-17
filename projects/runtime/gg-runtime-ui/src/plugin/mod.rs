pub mod binding;
pub mod focus;
pub mod input_bridge;
pub mod layout_system;
pub mod texture_registry;

use gg_core::{
    GResult,
    platform::{InputEvent, KeyState, PointerAction, PointerButton},
    plugin::{Plugin, PluginRegistrar},
};
use gg_ecs::{System, World};
use gg_render::RenderContext;
use gg_runtime::InputEvents;
use gg_ui::{EventSystem, LayoutEngine, UiEvent, UiRenderer, UiTree};

use self::{
    binding::{BindingRegistry, BindingResolver, BindingSystem, BindingValue, HashMapResolver},
    focus::FocusManager,
    input_bridge::{InputBridgeSystem, InputState},
    layout_system::{DirtyFlags, LayoutCacheResource, UiLayoutSystem},
    texture_registry::TextureRegistry,
};

/// UI 树资源
///
/// 将 gg-ui 的 UiTree 包装为 ECS 全局资源，
/// 以便通过 World 的资源系统进行存取。
pub struct UiTreeResource(pub UiTree);

impl UiTreeResource {
    /// 创建新的 UI 树资源
    pub fn new() -> Self {
        Self(UiTree::new())
    }
}

impl Default for UiTreeResource {
    fn default() -> Self {
        Self::new()
    }
}

/// 事件系统资源
///
/// 将 gg-ui 的 EventSystem 包装为 ECS 全局资源，
/// 以便通过 World 的资源系统进行存取。
pub struct EventSystemResource(pub EventSystem);

impl EventSystemResource {
    /// 创建新的事件系统资源
    pub fn new() -> Self {
        Self(EventSystem::new())
    }
}

impl Default for EventSystemResource {
    fn default() -> Self {
        Self::new()
    }
}

/// UI 插件
///
/// 将 gg-ui 组件库与 ECS 世界集成，
/// 注册 UI 树资源、事件系统资源、焦点管理器和 UI 更新/渲染/绑定系统。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "ui"
    }

    /// 构建 UI 插件
    ///
    /// 注册以下资源和系统：
    /// - 资源：UiTreeResource、EventSystemResource、FocusManager、BindingRegistry、InputState、DirtyFlags
    /// - 系统：UiInputSystem、UiUpdateSystem、UiRenderSystem、BindingSystem、InputBridgeSystem、UiLayoutSystem
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.insert_resource(UiTreeResource::new());
        registrar.insert_resource(EventSystemResource::new());
        registrar.insert_resource(FocusManager::new());
        registrar.insert_resource(BindingRegistry::new());
        registrar.insert_resource(TextureRegistry::new());
        registrar.insert_resource(InputState::new());
        registrar.insert_resource(DirtyFlags::new());
        registrar.insert_resource(LayoutCacheResource::new());
        registrar.register_system(Box::new(UiInputSystem));
        registrar.register_system(Box::new(UiUpdateSystem));
        registrar.register_system(Box::new(UiRenderSystem));
        registrar.register_system(Box::new(BindingSystem::new()));
        registrar.register_system(Box::new(InputBridgeSystem));
        registrar.register_system(Box::new(UiLayoutSystem));
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        vec!["render"]
    }

    /// 初始化 UI 插件
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭 UI 插件
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}

/// UI 输入系统
///
/// 将平台输入事件转换为 UI 事件并分发到 UI 节点。
/// 在 PreUpdate 阶段执行，优先于 UiUpdateSystem。
pub struct UiInputSystem;

impl System for UiInputSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_input"
    }

    /// 执行 UI 输入系统逻辑
    ///
    /// 从 World 获取 InputEvents 资源，将每个 InputEvent 转换为 UiEvent，
    /// 通过 EventSystem::dispatch_with_phases 三阶段分发指针事件到 UI 节点。
    /// 对于键盘事件，若存在焦点节点，则路由到焦点节点处理器。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let events = world.get_resource::<InputEvents>().map(|r| r.events.clone());
        let events = match events {
            Some(e) => e,
            None => return Ok(()),
        };

        if events.is_empty() {
            return Ok(());
        }

        let tree = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
        let tree = match tree {
            Some(t) => t,
            None => return Ok(()),
        };

        let focused = world.get_resource::<FocusManager>().and_then(|fm| fm.focused());

        let mut pointer_events: Vec<UiEvent> = Vec::new();
        let mut key_events: Vec<UiEvent> = Vec::new();

        for event in &events {
            match event {
                InputEvent::Pointer { action, button, position: (x, y) } => match action {
                    PointerAction::Down => {
                        if *button == Some(PointerButton::Left) {
                            pointer_events.push(UiEvent::Click { x: *x, y: *y });
                        }
                        pointer_events.push(UiEvent::MouseDown { x: *x, y: *y });
                    }
                    PointerAction::Up => {
                        pointer_events.push(UiEvent::MouseUp { x: *x, y: *y });
                    }
                    PointerAction::Move => {
                        pointer_events.push(UiEvent::MouseMove { x: *x, y: *y });
                    }
                    PointerAction::Scroll(_) => {}
                },
                InputEvent::Keyboard { key, state: KeyState::Pressed } => {
                    key_events.push(UiEvent::KeyInput { key: format!("{:?}", key) });
                }
                _ => {}
            }
        }

        if !pointer_events.is_empty() {
            if let Some(event_system) = world.get_resource_mut::<EventSystemResource>() {
                for ui_event in &pointer_events {
                    let (x, y) = match ui_event {
                        UiEvent::Click { x, y } => (*x, *y),
                        UiEvent::MouseMove { x, y } => (*x, *y),
                        UiEvent::MouseDown { x, y } => (*x, *y),
                        UiEvent::MouseUp { x, y } => (*x, *y),
                        UiEvent::Scroll { x, y, .. } => (*x, *y),
                        _ => continue,
                    };
                    if let Some(target_id) = EventSystem::hit_test(&tree, x, y) {
                        event_system.0.dispatch_with_phases(ui_event, &tree, target_id);
                    }
                }
            }
        }

        if !key_events.is_empty() {
            if let Some(focused_id) = focused {
                if let Some(event_system) = world.get_resource_mut::<EventSystemResource>() {
                    for key_event in &key_events {
                        event_system.0.dispatch_to_node(focused_id, key_event);
                    }
                }
            }
        }

        Ok(())
    }
}

/// UI 更新系统
///
/// 已由 UiLayoutSystem 接管布局计算职责，本系统现为空操作。
pub struct UiUpdateSystem;

impl System for UiUpdateSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_update"
    }

    /// 执行 UI 更新系统逻辑
    ///
    /// 本系统已不再执行布局计算，布局由 UiLayoutSystem 统一处理。
    fn execute(&mut self, _world: &mut World) -> GResult<()> {
        Ok(())
    }
}

/// UI 渲染系统
///
/// 每帧将 UI 树通过 UiRenderer 转换为 DrawCommand 渲染指令。
/// 在渲染前从 TextureRegistry 解析图片路径到纹理标识符。
pub struct UiRenderSystem;

impl System for UiRenderSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_render"
    }

    /// 执行 UI 渲染系统逻辑
    ///
    /// 从 World 获取 UiTreeResource 和 TextureRegistry，
    /// 将图片路径解析为纹理标识符后，委托 UiRenderer::render 生成绘制命令。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let texture_registry = world.get_resource::<TextureRegistry>().cloned();

        if let Some(tree_res) = world.get_resource_mut::<UiTreeResource>() {
            if let Some(ref registry) = texture_registry {
                Self::resolve_image_textures(&mut tree_res.0, registry);
            }
        }

        let tree = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
        let tree = match tree {
            Some(t) => t,
            None => return Ok(()),
        };

        if world.get_resource_mut::<RenderContext>().is_none() {
            return Ok(());
        }

        if let Some(ctx) = world.get_resource_mut::<RenderContext>() {
            UiRenderer::render(&tree, ctx);
        }

        Ok(())
    }
}

impl UiRenderSystem {
    /// 将 UI 树中所有 Image 节点的 image_path 解析为 texture_id
    fn resolve_image_textures(tree: &mut UiTree, registry: &TextureRegistry) {
        if let Some(root_id) = tree.root() {
            Self::resolve_node_textures(tree, root_id, registry);
        }
    }

    fn resolve_node_textures(tree: &mut UiTree, node_id: gg_ui::UiNodeId, registry: &TextureRegistry) {
        let image_path = tree.get(node_id).and_then(|n| n.style.image_path.clone());
        if let Some(path) = image_path {
            if let Some(node) = tree.get_mut(node_id) {
                if let gg_ui::UiNodeData::Image { ref mut texture_id, .. } = node.data {
                    *texture_id = registry.get(&path);
                }
            }
        }

        let children: Vec<gg_ui::UiNodeId> = tree.get(node_id).map(|n| n.children.clone()).unwrap_or_default();
        for child_id in children {
            Self::resolve_node_textures(tree, child_id, registry);
        }
    }
}
