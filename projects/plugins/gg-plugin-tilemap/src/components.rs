//! 瓦片地图核心组件模块

use gg_ecs::Component;
use serde::{Deserialize, Serialize};

/// 斜坡方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SlopeDirection {
    /// 左上到右下
    LeftUp,
    /// 右上到左下
    RightUp,
}

/// 瓦片类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileType {
    /// 普通瓦片
    Normal,
    /// 碰撞瓦片
    Solid,
    /// 斜坡瓦片
    Slope {
        /// 斜坡方向
        direction: SlopeDirection,
    },
    /// 触发器瓦片
    Trigger {
        /// 关联脚本 ID
        script_id: String,
    },
}

/// 图层定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayer {
    /// 图层名称
    pub name: String,
    /// 图层 Z 排序值
    pub z_order: i32,
    /// 图层可见性
    pub visible: bool,
    /// 图层透明度
    pub opacity: f32,
}

/// 瓦片地图组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tilemap {
    /// 地图宽度（瓦片数）
    pub width: u32,
    /// 地图高度（瓦片数）
    pub height: u32,
    /// 瓦片宽度（像素）
    pub tile_width: u32,
    /// 瓦片高度（像素）
    pub tile_height: u32,
    /// 图层列表
    pub layers: Vec<TileLayer>,
}

impl Component for Tilemap {}

/// 瓦片组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    /// 瓦片在地图中的列索引
    pub col: u32,
    /// 瓦片在地图中的行索引
    pub row: u32,
    /// 所属图层索引
    pub layer_index: u32,
    /// 瓦片在图集中的列索引
    pub tileset_col: u32,
    /// 瓦片在图集中的行索引
    pub tileset_row: u32,
    /// 瓦片类型
    pub tile_type: TileType,
}

impl Component for Tile {}
