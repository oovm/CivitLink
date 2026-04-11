#![warn(missing_docs)]

//! GG 引擎角色组件模块
//! 提供通用的角色相关组件定义

use gg_render::TextureId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDef {
    /// 角色唯一标识
    pub id: String,
    /// 角色显示名称
    pub name: String,
    /// 默认资源路径
    pub default_asset_path: Option<String>,
    /// 表情标签到资源路径的映射
    pub expression_map: HashMap<String, String>,
    /// 角色名字颜色（RGBA）
    pub color: Option<[f32; 4]>,
}

/// 角色状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterState {
    /// 关联角色 ID
    pub character_id: String,
    /// 当前表情标签
    pub current_expression: String,
    /// 位置 X 坐标
    pub x: f32,
    /// 位置 Y 坐标
    pub y: f32,
    /// 缩放比例（默认 1.0）
    pub scale: f32,
    /// 透明度（默认 1.0）
    pub opacity: f32,
    /// 是否正在说话（用于高亮）
    pub is_speaking: bool,
    /// Z 轴排序
    pub z_order: i32,
    /// 角色纹理标识
    #[serde(skip)]
    pub texture_id: TextureId,
    /// 角色纹理宽度（像素）
    #[serde(skip)]
    pub texture_width: f32,
    /// 角色纹理高度（像素）
    #[serde(skip)]
    pub texture_height: f32,
}

/// 立绘位置枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterPosition {
    /// 左侧
    Left,
    /// 中央
    Center,
    /// 右侧
    Right,
    /// 自定义坐标
    Custom {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
}

/// 滑动方向枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlideDirection {
    /// 向左
    Left,
    /// 向右
    Right,
    /// 向上
    Up,
    /// 向下
    Down,
}

/// 过渡动画类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionType {
    /// 无动画
    None,
    /// 淡入淡出
    Fade {
        /// 持续时间（秒）
        duration_secs: f32,
    },
    /// 交叉溶解
    CrossDissolve {
        /// 持续时间（秒）
        duration_secs: f32,
    },
    /// 滑动
    Slide {
        /// 持续时间（秒）
        duration_secs: f32,
        /// 滑动方向
        direction: SlideDirection,
    },
}
