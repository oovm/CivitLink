//! 瓦片地图资源类型模块

use gg_ecs::Entity;
use gg_render::TextureId;

/// 瓦片图集资源
#[derive(Debug, Clone)]
pub struct Tileset {
    /// 图集纹理 ID
    pub texture_id: TextureId,
    /// 单个瓦片宽度（像素）
    pub tile_width: u32,
    /// 单个瓦片高度（像素）
    pub tile_height: u32,
    /// 图集列数
    pub columns: u32,
    /// 图集行数
    pub rows: u32,
}

/// 碰撞信息
#[derive(Debug, Clone)]
pub struct CollisionInfo {
    /// 碰撞的瓦片实体
    pub tile_entity: Entity,
    /// 碰撞的瓦片列索引
    pub tile_col: u32,
    /// 碰撞的瓦片行索引
    pub tile_row: u32,
}

/// 瓦片碰撞状态资源
#[derive(Debug, Clone, Default)]
pub struct TileCollisionState {
    /// 当前帧的碰撞信息列表
    pub collisions: Vec<CollisionInfo>,
}
