//! 瓦片地图插件模块

use gg_core::{
    GResult,
    plugin::{Plugin, PluginRegistrar},
};

use crate::{
    resources::TileCollisionState,
    systems::{TileCollisionSystem, TilemapRenderSystem},
};

/// 瓦片地图插件
///
/// 提供瓦片地图的渲染和碰撞检测功能。
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "tilemap"
    }

    /// 构建瓦片地图插件
    ///
    /// 注册瓦片地图渲染系统、碰撞检测系统和碰撞状态资源。
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(TilemapRenderSystem::new()));
        registrar.register_system(Box::new(TileCollisionSystem::new()));
        registrar.insert_resource(TileCollisionState::default());
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
