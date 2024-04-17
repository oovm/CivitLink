#![warn(missing_docs)]

//! GG 引擎 UI 插件
//! 提供声明式 UI 系统与 ECS 的集成

use gg_core::{
    GResult,
    plugin::{Plugin, PluginRegistrar},
};
use gg_ecs::{Resource, System, World};
use gg_ui::{EventSystem, LayoutEngine, UiRenderer, UiTree};

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
/// 注册 UI 树资源、事件系统资源和 UI 更新/渲染系统。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "ui"
    }

    /// 构建 UI 插件
    ///
    /// 注册以下资源和系统：
    /// - 资源：UiTreeResource、EventSystemResource
    /// - 系统：UiUpdateSystem、UiRenderSystem
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.insert_resource(UiTreeResource::new());
        registrar.insert_resource(EventSystemResource::new());
        registrar.register_system(Box::new(UiUpdateSystem));
        registrar.register_system(Box::new(UiRenderSystem));
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
/// 当前阶段暂不执行实际渲染，待 RenderContext 成为 Resource 后完善。
pub struct UiRenderSystem;

impl System for UiRenderSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_render"
    }

    /// 执行 UI 渲染系统逻辑
    ///
    /// 从 World 获取 UiTreeResource，调用 UiRenderer 生成 DrawCommand。
    /// 当前阶段暂不执行实际渲染，待 RenderContext 成为 Resource 后完善。
    fn execute(&mut self, _world: &mut World) -> GResult<()> {
        Ok(())
    }
}
