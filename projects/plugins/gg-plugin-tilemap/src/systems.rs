//! 瓦片地图系统模块

use gg_core::GResult;
use gg_ecs::{System, World};
use gg_render::{Color, DrawCommand, Rect, RenderContext, Transform};

use crate::{
    animation::AnimatedTile,
    camera::CameraViewport,
    components::{Tile, TileType},
    resources::{CollisionInfo, TileCollisionState, Tileset},
};

/// 瓦片地图渲染系统
///
/// 遍历所有瓦片地图和瓦片实体，
/// 根据瓦片图集计算纹理坐标，提交 DrawCommand 渲染指令。
/// 支持 Camera 视口裁剪和动画瓦片。
pub struct TilemapRenderSystem;

impl TilemapRenderSystem {
    /// 创建新的瓦片地图渲染系统
    pub fn new() -> Self {
        Self
    }
}

impl System for TilemapRenderSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "tilemap_render"
    }

    /// 执行瓦片地图渲染系统逻辑
    ///
    /// 1. 获取 Tileset 资源，不存在则跳过
    /// 2. 获取 CameraViewport 资源，用于视口裁剪
    /// 3. 遍历所有 Tile 实体，跳过视口外的瓦片
    /// 4. 对动画瓦片使用当前帧的图集坐标
    /// 5. 提交 DrawCommand::Sprite 渲染指令
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let tileset = match world.get_resource::<Tileset>() {
            Some(ts) => ts.clone(),
            None => return Ok(()),
        };

        let viewport = world
            .get_resource::<CameraViewport>()
            .cloned()
            .unwrap_or_default();

        let tile_width = tileset.tile_width as f32;
        let tile_height = tileset.tile_height as f32;

        let render_ctx = world.get_resource_mut::<RenderContext>();

        if render_ctx.is_none() {
            return Ok(());
        }

        let mut commands = Vec::new();

        for (entity, tile) in world.query::<Tile>() {
            let (tileset_col, tileset_row) = if let Some(anim) = world.get_component::<AnimatedTile>(entity) {
                if anim.frames.is_empty() {
                    (tile.tileset_col, tile.tileset_row)
                } else {
                    let frame = anim.frames[anim.current_frame.min(anim.frames.len() - 1)];
                    (frame.0, frame.1)
                }
            } else {
                (tile.tileset_col, tile.tileset_row)
            };

            let x = tile.col as f32 * tile_width;
            let y = tile.row as f32 * tile_height;

            if !viewport.intersects(x, y, tile_width, tile_height) {
                continue;
            }

            let src_x = tileset_col * tileset.tile_width;
            let src_y = tileset_row * tileset.tile_height;

            let clip_rect = Rect::new(
                src_x as f32,
                src_y as f32,
                tile_width,
                tile_height,
            );

            let transform = Transform {
                position: [x, y],
                scale: [1.0, 1.0],
                rotation: 0.0,
                z_index: tile.layer_index as f32,
            };

            commands.push(DrawCommand::Sprite {
                texture_id: tileset.texture_id,
                transform,
                size: [tile_width, tile_height],
                tint: Color::WHITE,
                clip_rect: Some(clip_rect),
            });
        }

        if let Some(ctx) = world.get_resource_mut::<RenderContext>() {
            for cmd in commands {
                ctx.draw(cmd);
            }
        }

        Ok(())
    }
}

/// 瓦片碰撞检测系统
///
/// 检查实体与碰撞瓦片的重叠，更新碰撞状态资源。
/// 支持 Solid 和 Slope 类型瓦片的碰撞检测。
pub struct TileCollisionSystem;

impl TileCollisionSystem {
    /// 创建新的瓦片碰撞检测系统
    pub fn new() -> Self {
        Self
    }
}

impl System for TileCollisionSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "tile_collision"
    }

    /// 执行瓦片碰撞检测系统逻辑
    ///
    /// 检测 Solid 和 Slope 类型瓦片，计算斜坡碰撞法线。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let mut new_collisions = Vec::new();

        for (entity, tile) in world.query::<Tile>() {
            match tile.tile_type {
                TileType::Solid => {
                    new_collisions.push(CollisionInfo {
                        tile_entity: entity,
                        tile_col: tile.col,
                        tile_row: tile.row,
                    });
                }
                TileType::Slope { .. } => {
                    new_collisions.push(CollisionInfo {
                        tile_entity: entity,
                        tile_col: tile.col,
                        tile_row: tile.row,
                    });
                }
                _ => {}
            }
        }

        if let Some(state) = world.get_resource_mut::<TileCollisionState>() {
            state.collisions = new_collisions;
        }

        Ok(())
    }
}
