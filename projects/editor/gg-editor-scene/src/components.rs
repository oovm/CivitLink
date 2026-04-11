#![warn(missing_docs)]

//! 场景视图 ECS 组件定义
//!
//! 提供场景编辑器使用的标准 ECS 组件，包括 2D 变换、精灵渲染和矩形渲染组件。
//! 这些组件用于从 GameWorld 查询实体数据并渲染到场景视图中。

use gg_ecs::Component;
use gg_render::Color;

/// 2D 变换组件
///
/// 描述实体在二维世界空间中的位置、旋转和缩放。
/// 拥有此组件的实体才能在场景视图中被渲染和交互。
#[derive(Debug, Clone, Component)]
pub struct Transform2D {
    /// X 坐标（世界空间）
    pub x: f32,
    /// Y 坐标（世界空间）
    pub y: f32,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// X 缩放因子
    pub scale_x: f32,
    /// Y 缩放因子
    pub scale_y: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

/// 精灵渲染组件
///
/// 使用纹理渲染的实体组件，当前以填充矩形作为占位渲染。
#[derive(Debug, Clone, Component)]
pub struct SpriteRenderer {
    /// 纹理资源路径
    pub texture_path: String,
    /// 宽度（世界坐标单位）
    pub width: f32,
    /// 高度（世界坐标单位）
    pub height: f32,
    /// 填充颜色
    pub color: Color,
}

impl Default for SpriteRenderer {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            width: 50.0,
            height: 50.0,
            color: Color::WHITE,
        }
    }
}

/// 矩形渲染组件
///
/// 使用纯色矩形渲染的实体组件。
#[derive(Debug, Clone, Component)]
pub struct RectRenderer {
    /// 宽度（世界坐标单位）
    pub width: f32,
    /// 高度（世界坐标单位）
    pub height: f32,
    /// 填充颜色
    pub color: Color,
}

impl Default for RectRenderer {
    fn default() -> Self {
        Self {
            width: 50.0,
            height: 50.0,
            color: Color::new(0.5, 0.5, 0.5, 1.0),
        }
    }
}
