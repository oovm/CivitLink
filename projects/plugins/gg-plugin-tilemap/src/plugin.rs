//! 瓦片地图插件模块

use gg_core::{
    GResult,
    plugin::{Plugin, PluginRegistrar},
};
use gg_render::RenderContext;

use crate::{
    animation::TileAnimationSystem,
    camera::CameraViewport,
    resources::TileCollisionState,
    systems::{TileCollisionSystem, TilemapRenderSystem},
};

/// 瓦片地图插件
///
/// 提供瓦片地图的渲染、碰撞检测、视口裁剪和动画瓦片功能。
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "tilemap"
    }

    /// 构建瓦片地图插件
    ///
    /// 注册以下资源和系统：
    /// - 资源：TileCollisionState、CameraViewport
    /// - 系统：TilemapRenderSystem、TileCollisionSystem、TileAnimationSystem
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(TilemapRenderSystem::new()));
        registrar.register_system(Box::new(TileCollisionSystem::new()));
        registrar.register_system(Box::new(TileAnimationSystem::new()));
        registrar.insert_resource(TileCollisionState::default());
        registrar.insert_resource(CameraViewport::default());
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        vec!["render"]
    }

    /// 初始化瓦片地图插件
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭瓦片地图插件
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
