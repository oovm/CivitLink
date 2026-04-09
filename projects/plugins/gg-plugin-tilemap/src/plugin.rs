//! 瓦片地图插件模块

use gg_core::plugin::{Plugin, PluginRegistrar};
use gg_core::GResult;

use crate::resources::TileCollisionState;
use crate::systems::{TileCollisionSystem, TilemapRenderSystem};

/// 瓦片地图插件
///
/// 提供瓦片地图的渲染和碰撞检测功能。
pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn name(&self) -> &str {
        "tilemap"
    }

    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(TilemapRenderSystem::new()));
        registrar.register_system(Box::new(TileCollisionSystem::new()));
        registrar.insert_resource(TileCollisionState::default());
    }

    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
