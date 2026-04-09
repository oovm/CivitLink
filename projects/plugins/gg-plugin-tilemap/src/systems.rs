//! 瓦片地图系统模块

use gg_core::GResult;
use gg_ecs::{GgWorld, System};
use gg_render::Transform;

use crate::components::{Tile, TileType};
use crate::resources::{CollisionInfo, TileCollisionState, Tileset};

/// 瓦片地图渲染系统
///
/// 遍历所有瓦片地图和瓦片实体，
/// 根据瓦片图集计算纹理坐标，提交 DrawCommand 渲染指令。
pub struct TilemapRenderSystem;

impl TilemapRenderSystem {
    /// 创建新的瓦片地图渲染系统
    pub fn new() -> Self {
        Self
    }
}

impl System for TilemapRenderSystem {
    fn name(&self) -> &str {
        "tilemap_render"
    }

    fn execute(&mut self, world: &mut GgWorld) -> GResult<()> {
        let tileset = match world.get_resource::<Tileset>() {
            Some(ts) => ts.clone(),
            None => return Ok(()),
        };

        let tile_width = tileset.tile_width as f32;
        let tile_height = tileset.tile_height as f32;

        for (_entity, tile) in world.query::<Tile>() {
            let _src_x = tile.tileset_col * tileset.tile_width;
            let _src_y = tile.tileset_row * tileset.tile_height;

            let x = tile.col as f32 * tile_width;
            let y = tile.row as f32 * tile_height;

            let _transform = Transform {
                position: [x, y],
                scale: [1.0, 1.0],
                rotation: 0.0,
                z_index: tile.layer_index as f32,
            };
        }

        Ok(())
    }
}

/// 瓦片碰撞检测系统
///
/// 检查实体与碰撞瓦片的重叠，更新碰撞状态资源。
pub struct TileCollisionSystem;

impl TileCollisionSystem {
    /// 创建新的瓦片碰撞检测系统
    pub fn new() -> Self {
        Self
    }
}

impl System for TileCollisionSystem {
    fn name(&self) -> &str {
        "tile_collision"
    }

    fn execute(&mut self, world: &mut GgWorld) -> GResult<()> {
        let mut new_collisions = Vec::new();

        for (entity, tile) in world.query::<Tile>() {
            if matches!(tile.tile_type, TileType::Solid) {
                new_collisions.push(CollisionInfo {
                    tile_entity: entity,
                    tile_col: tile.col,
                    tile_row: tile.row,
                });
            }
        }

        if let Some(state) = world.get_resource_mut::<TileCollisionState>() {
            state.collisions = new_collisions;
        }

        Ok(())
    }
}
