//! Spine 动画资源类型模块

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{components::BoneTransform, skin::Skin};

/// 骨骼定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneDef {
    /// 骨骼名称
    pub name: String,
    /// 父骨骼索引（None 表示根骨骼）
    pub parent_index: Option<usize>,
    /// 默认局部变换
    pub transform: BoneTransform,
}

/// 插槽附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotAttachment {
    /// 附件名称
    pub name: String,
    /// 关联纹理路径
    pub texture_path: String,
    /// 附件区域 X 偏移
    pub offset_x: f32,
    /// 附件区域 Y 偏移
    pub offset_y: f32,
    /// 附件区域宽度
    pub width: f32,
    /// 附件区域高度
    pub height: f32,
}

/// 动画定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationDef {
    /// 动画名称
    pub name: String,
    /// 动画时长（秒）
    pub duration: f32,
}

/// Spine 数据资源
#[derive(Debug, Clone)]
pub struct SpineData {
    /// 骨骼定义列表
    pub bones: Vec<BoneDef>,
    /// 动画定义列表
    pub animations: Vec<AnimationDef>,
    /// 插槽附件映射（插槽名 → 附件列表）
    pub attachments: HashMap<String, Vec<SlotAttachment>>,
    /// 可用皮肤列表
    pub skins: Vec<Skin>,
}
