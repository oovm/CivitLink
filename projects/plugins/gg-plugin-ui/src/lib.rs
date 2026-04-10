#![warn(missing_docs)]

//! GG 引擎 UI 插件
//! 提供声明式 UI 系统与 ECS 的集成

pub mod binding;
pub mod focus;

use gg_core::{
    GResult,
    plugin::{Plugin, PluginRegistrar},
};
use gg_ecs::{Resource, System, World};
use gg_render::{Color, DrawCommand, RenderContext, Transform};
use gg_ui::{EventSystem, LayoutEngine, UiRenderer, UiTree};

use crate::binding::{BindingRegistry, BindingSystem};
use crate::focus::FocusManager;

/// UI 树资源
///
/// 将 gg-ui 的 UiTree 包装为 ECS 全局资源，
/// 以便通过 World 的资源系统进行存取。
pub struct UiTreeResource(pub UiTree);

impl Resource for UiTreeResource {}

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

impl Resource for EventSystemResource {}

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
    /// - 资源：UiTreeResource、EventSystemResource、FocusManager、BindingRegistry
    /// - 系统：UiUpdateSystem、UiRenderSystem、BindingSystem
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.insert_resource(UiTreeResource::new());
        registrar.insert_resource(EventSystemResource::new());
        registrar.insert_resource(FocusManager::new());
        registrar.insert_resource(BindingRegistry::new());
        registrar.register_system(Box::new(UiUpdateSystem));
        registrar.register_system(Box::new(UiRenderSystem));
        registrar.register_system(Box::new(BindingSystem::new()));
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

/// UI 更新系统
///
/// 每帧执行 UI 树的布局计算。
/// 通过克隆-计算-回写策略避免借用冲突。
pub struct UiUpdateSystem;

impl System for UiUpdateSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_update"
    }

    /// 执行 UI 更新系统逻辑
    ///
    /// 从 World 获取 UiTreeResource，克隆出 UiTree，
    /// 调用 LayoutEngine::compute 执行布局计算，
    /// 然后将计算结果回写到 World 中。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (width, height) = (800.0_f32, 600.0_f32);

        let tree_opt = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
        if let Some(mut tree) = tree_opt {
            LayoutEngine::compute(&mut tree, width, height);
            if let Some(world_tree) = world.get_resource_mut::<UiTreeResource>() {
                world_tree.0 = tree;
            }
        }

        Ok(())
    }
}

/// UI 渲染系统
///
/// 每帧将 UI 树转换为 DrawCommand 渲染指令。
pub struct UiRenderSystem;

impl System for UiRenderSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_render"
    }

    /// 执行 UI 渲染系统逻辑
    ///
    /// 从 World 获取 UiTreeResource，遍历 UI 节点，
    /// 为每个可见节点提交 DrawCommand::Rect 渲染指令。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let tree_opt = world.get_resource::<UiTreeResource>().map(|r| r.0.clone());
        let tree = match tree_opt {
            Some(t) => t,
            None => return Ok(()),
        };

        let render_ctx = world.get_resource_mut::<RenderContext>();
        if render_ctx.is_none() {
            return Ok(());
        }

        let mut commands = Vec::new();

        if let Some(root_id) = tree.root() {
            Self::render_node(&tree, root_id, &mut commands);
        }

        if let Some(ctx) = world.get_resource_mut::<RenderContext>() {
            for cmd in commands {
                ctx.draw(cmd);
            }
        }

        Ok(())
    }
}

impl UiRenderSystem {
    /// 递归渲染 UI 节点
    fn render_node(tree: &UiTree, node_id: gg_ui::UiNodeId, commands: &mut Vec<DrawCommand>) {
        let node = match tree.get(node_id) {
            Some(n) => n,
            None => return,
        };

        if !node.visible {
            return;
        }

        if let Some(ref layout) = node.layout_result {
            commands.push(DrawCommand::Rect {
                rect: gg_render::Rect::new(layout.x, layout.y, layout.width, layout.height),
                color: Color::TRANSPARENT,
                corner_radius: 0.0,
            });

            if let gg_ui::UiNodeData::Text { ref content } = node.data {
                commands.push(DrawCommand::Text {
                    text: content.clone(),
                    position: [layout.x, layout.y],
                    font_size: 16.0,
                    color: Color::WHITE,
                    max_width: Some(layout.width),
                });
            }
        }

        for &child_id in &node.children {
            Self::render_node(tree, child_id, commands);
        }
    }
}
