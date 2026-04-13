//! TMX 地图加载器模块
//! 提供 Tiled Map Editor JSON 格式的地图数据加载功能

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::World;
use gg_render::TextureId;
use serde::{Deserialize, Serialize};

use crate::{
    components::{MapObject, Tile, TileLayer, TileType},
    resources::Tileset,
};

/// TMX 地图根数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmxMapData {
    /// 地图宽度（瓦片数）
    pub width: u32,
    /// 地图高度（瓦片数）
    pub height: u32,
    /// 瓦片宽度（像素）
    pub tilewidth: u32,
    /// 瓦片高度（像素）
    pub tileheight: u32,
    /// 图层列表
    pub layers: Vec<TmxLayer>,
    /// 瓦片图集列表
    pub tilesets: Vec<TmxTileset>,
}

/// TMX 图层数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmxLayer {
    /// 图层名称
    pub name: String,
    /// 图层类型 ("tilelayer" 或 "objectgroup")
    #[serde(rename = "type")]
    pub layer_type: String,
    /// 图层可见性
    pub visible: bool,
    /// 图层透明度
    pub opacity: f32,
    /// 瓦片数据（GID 列表，仅 tilelayer）
    pub data: Option<Vec<u32>>,
    /// 对象列表（仅 objectgroup）
    pub objects: Option<Vec<TmxObject>>,
    /// 图层偏移 X
    pub offsetx: Option<f32>,
    /// 图层偏移 Y
    pub offsety: Option<f32>,
}

/// TMX 瓦片图集数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmxTileset {
    /// 第一个 GID
    pub firstgid: u32,
    /// 图集名称
    pub name: String,
    /// 图集中瓦片宽度
    pub tilewidth: u32,
    /// 图集中瓦片高度
    pub tileheight: u32,
    /// 图集列数
    pub columns: u32,
    /// 图集中瓦片总数
    pub tilecount: u32,
    /// 图集纹理路径
    pub image: Option<String>,
    /// 纹理宽度
    pub imagewidth: Option<u32>,
    /// 纹理高度
    pub imageheight: Option<u32>,
    /// 瓦片属性列表
    pub tiles: Option<Vec<TmxTileProperty>>,
}

/// TMX 瓦片属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmxTileProperty {
    /// 瓦片 ID
    pub id: u32,
    /// 瓦片类型
    #[serde(rename = "type")]
    pub tile_type: Option<String>,
    /// 自定义属性
    pub properties: Option<HashMap<String, TmxPropertyValue>>,
}

/// TMX 属性值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmxPropertyValue {
    /// 属性类型
    #[serde(rename = "type")]
    pub value_type: Option<String>,
    /// 属性值
    pub value: Option<serde_json::Value>,
}

/// TMX 对象数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmxObject {
    /// 对象名称
    pub name: Option<String>,
    /// 对象类型
    #[serde(rename = "type")]
    pub object_type: Option<String>,
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
    /// 宽度
    pub width: Option<f32>,
    /// 高度
    pub height: Option<f32>,
    /// 自定义属性
    pub properties: Option<HashMap<String, TmxPropertyValue>>,
}

/// TMX 地图加载器
pub struct TmxLoader;

impl TmxLoader {
    /// 从 JSON 字符串加载 TMX 地图到 World
    pub fn load_from_json(world: &mut World, json: &str) -> GResult<()> {
        let map_data: TmxMapData = serde_json::from_str(json)
            .map_err(|e| GError { kind: GErrorKind::Asset, message: format!("Failed to parse TMX JSON: {}", e) })?;

        let tilemap_entity = world.spawn().id();

        let mut tilemap = crate::components::Tilemap {
            width: map_data.width,
            height: map_data.height,
            tile_width: map_data.tilewidth,
            tile_height: map_data.tileheight,
            layers: Vec::new(),
        };

        for tmx_tileset in &map_data.tilesets {
            let columns = tmx_tileset.columns;
            let rows =
                if tmx_tileset.tilecount > 0 && columns > 0 { (tmx_tileset.tilecount + columns - 1) / columns } else { 0 };
            let tileset = Tileset {
                texture_id: TextureId::INVALID,
                tile_width: tmx_tileset.tilewidth,
                tile_height: tmx_tileset.tileheight,
                columns,
                rows,
            };
            world.insert_resource(tileset);
        }

        let mut layer_index = 0u32;
        for tmx_layer in &map_data.layers {
            match tmx_layer.layer_type.as_str() {
                "tilelayer" => {
                    let tile_layer = TileLayer {
                        name: tmx_layer.name.clone(),
                        z_order: layer_index as i32,
                        visible: tmx_layer.visible,
                        opacity: tmx_layer.opacity,
                    };

                    tilemap.layers.push(tile_layer);

                    if let Some(ref data) = tmx_layer.data {
                        let first_gid = map_data.tilesets.first().map(|t| t.firstgid).unwrap_or(1);
                        let columns = map_data.tilesets.first().map(|t| t.columns).unwrap_or(1);

                        for (i, &gid) in data.iter().enumerate() {
                            if gid == 0 {
                                continue;
                            }

                            let local_id = gid - first_gid;
                            let tileset_col = local_id % columns;
                            let tileset_row = local_id / columns;
                            let col = (i as u32) % map_data.width;
                            let row = (i as u32) / map_data.width;

                            let tile = Tile { col, row, layer_index, tileset_col, tileset_row, tile_type: TileType::Normal };
                            let tile_entity = world.spawn().id();
                            world.add_component(tile_entity, tile)?;
                        }
                    }

                    layer_index += 1;
                }
                "objectgroup" => {
                    if let Some(ref objects) = tmx_layer.objects {
                        for tmx_obj in objects {
                            let obj_entity = world.spawn().id();
                            let map_obj = MapObject {
                                name: tmx_obj.name.clone().unwrap_or_default(),
                                object_type: tmx_obj.object_type.clone().unwrap_or_default(),
                                x: tmx_obj.x,
                                y: tmx_obj.y,
                                width: tmx_obj.width.unwrap_or(0.0),
                                height: tmx_obj.height.unwrap_or(0.0),
                                properties: convert_properties(&tmx_obj.properties),
                            };
                            world.add_component(obj_entity, map_obj)?;
                        }
                    }
                }
                _ => {}
            }
        }

        world.add_component(tilemap_entity, tilemap)?;

        Ok(())
    }
}

/// 将 TMX 属性映射转换为组件属性映射
fn convert_properties(
    tmx_props: &Option<HashMap<String, TmxPropertyValue>>,
) -> HashMap<String, crate::components::ObjectPropertyValue> {
    let mut result = HashMap::new();
    if let Some(props) = tmx_props {
        for (key, val) in props {
            let value_str = match &val.value {
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            };
            result.insert(key.clone(), crate::components::ObjectPropertyValue { value: value_str });
        }
    }
    result
}
