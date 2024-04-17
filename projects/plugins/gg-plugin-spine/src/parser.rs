//! Spine JSON 数据解析模块

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use gg_core::{GError, GErrorKind, GResult};

use crate::{
    components::BoneTransform,
    resources::{AnimationDef, BoneDef, SlotAttachment, SpineData},
};

/// Spine JSON 根数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonData {
    /// 骨骼列表
    pub bones: Vec<SpineJsonBone>,
    /// 插槽列表
    pub slots: Option<Vec<SpineJsonSlot>>,
    /// 皮肤列表
    pub skins: Option<Vec<SpineJsonSkin>>,
    /// 动画列表
    pub animations: Option<HashMap<String, SpineJsonAnimation>>,
}

/// Spine JSON 骨骼数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonBone {
    /// 骨骼名称
    pub name: String,
    /// 父骨骼名称
    pub parent: Option<String>,
    /// X 偏移
    pub x: Option<f32>,
    /// Y 偏移
    pub y: Option<f32>,
    /// 旋转角度（度）
    pub rotation: Option<f32>,
    /// X 缩放
    #[serde(rename = "scaleX")]
    pub scale_x: Option<f32>,
    /// Y 缩放
    #[serde(rename = "scaleY")]
    pub scale_y: Option<f32>,
    /// 骨骼长度
    pub length: Option<f32>,
}

/// Spine JSON 插槽数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonSlot {
    /// 插槽名称
    pub name: String,
    /// 关联骨骼名称
    pub bone: String,
    /// 默认附件名称
    pub attachment: Option<String>,
    /// 插槽顺序
    pub order: Option<i32>,
}

/// Spine JSON 皮肤数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonSkin {
    /// 皮肤名称
    pub name: String,
    /// 附件映射（插槽名 → 附件名 → 附件数据）
    pub attachments: Option<HashMap<String, HashMap<String, SpineJsonAttachment>>>,
}

/// Spine JSON 附件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonAttachment {
    /// 附件类型
    #[serde(rename = "type")]
    pub attachment_type: Option<String>,
    /// 附件名称
    pub name: Option<String>,
    /// 纹理路径
    pub path: Option<String>,
    /// X 偏移
    pub x: Option<f32>,
    /// Y 偏移
    pub y: Option<f32>,
    /// 宽度
    pub width: Option<f32>,
    /// 高度
    pub height: Option<f32>,
    /// 旋转角度
    pub rotation: Option<f32>,
}

/// Spine JSON 动画数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonAnimation {
    /// 骨骼动画时间线
    pub bones: Option<HashMap<String, SpineJsonBoneTimeline>>,
    /// 插槽动画时间线
    pub slots: Option<HashMap<String, SpineJsonSlotTimeline>>,
}

/// Spine JSON 骨骼动画时间线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonBoneTimeline {
    /// 位移时间线
    pub translate: Option<Vec<SpineJsonTimelineKeyframe>>,
    /// 旋转时间线
    pub rotate: Option<Vec<SpineJsonTimelineKeyframe>>,
    /// 缩放时间线
    pub scale: Option<Vec<SpineJsonTimelineKeyframe>>,
}

/// Spine JSON 插槽动画时间线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonSlotTimeline {
    /// 附件时间线
    pub attachment: Option<Vec<SpineJsonAttachmentKeyframe>>,
}

/// Spine JSON 时间线关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonTimelineKeyframe {
    /// 时间点（秒）
    pub time: f32,
    /// X 值
    pub x: Option<f32>,
    /// Y 值
    pub y: Option<f32>,
    /// 角度值（度）
    pub angle: Option<f32>,
    /// X 缩放值
    #[serde(rename = "scaleX")]
    pub scale_x: Option<f32>,
    /// Y 缩放值
    #[serde(rename = "scaleY")]
    pub scale_y: Option<f32>,
}

/// Spine JSON 附件关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineJsonAttachmentKeyframe {
    /// 时间点（秒）
    pub time: f32,
    /// 附件名称
    pub name: Option<String>,
}

/// Spine 数据解析器
pub struct SpineParser;

impl SpineParser {
    /// 从 JSON 字符串解析 Spine 数据
    pub fn parse(json: &str) -> GResult<SpineData> {
        let json_data: SpineJsonData = serde_json::from_str(json)
            .map_err(|e| GError { kind: GErrorKind::Asset, message: format!("Failed to parse Spine JSON: {}", e) })?;

        let bone_name_map: HashMap<String, usize> =
            json_data.bones.iter().enumerate().map(|(i, b)| (b.name.clone(), i)).collect();

        let bones: Vec<BoneDef> = json_data
            .bones
            .iter()
            .map(|b| {
                let parent_index = b.parent.as_ref().and_then(|p| bone_name_map.get(p)).copied();
                let rotation_rad = b.rotation.unwrap_or(0.0).to_radians();
                BoneDef {
                    name: b.name.clone(),
                    parent_index,
                    transform: BoneTransform {
                        x: b.x.unwrap_or(0.0),
                        y: b.y.unwrap_or(0.0),
                        rotation: rotation_rad,
                        scale_x: b.scale_x.unwrap_or(1.0),
                        scale_y: b.scale_y.unwrap_or(1.0),
                    },
                }
            })
            .collect();

        let animations: Vec<AnimationDef> = json_data
            .animations
            .as_ref()
            .map(|anim_map| {
                anim_map
                    .iter()
                    .map(|(name, anim)| {
                        let duration = Self::compute_animation_duration(anim);
                        AnimationDef { name: name.clone(), duration }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut attachments: HashMap<String, Vec<SlotAttachment>> = HashMap::new();
        if let Some(skins) = &json_data.skins {
            for skin in skins {
                if let Some(skin_attachments) = &skin.attachments {
                    for (slot_name, slot_attachments) in skin_attachments {
                        let entry = attachments.entry(slot_name.clone()).or_default();
                        for (_attach_name, attach_data) in slot_attachments {
                            let attachment = SlotAttachment {
                                name: attach_data.name.clone().unwrap_or_default(),
                                texture_path: attach_data.path.clone().unwrap_or_default(),
                                offset_x: attach_data.x.unwrap_or(0.0),
                                offset_y: attach_data.y.unwrap_or(0.0),
                                width: attach_data.width.unwrap_or(0.0),
                                height: attach_data.height.unwrap_or(0.0),
                            };
                            entry.push(attachment);
                        }
                    }
                }
            }
        }

        Ok(SpineData { bones, animations, attachments })
    }

    /// 计算动画时长
    fn compute_animation_duration(anim: &SpineJsonAnimation) -> f32 {
        let mut max_time = 0.0f32;

        if let Some(bones) = &anim.bones {
            for timeline in bones.values() {
                if let Some(keys) = &timeline.translate {
                    for key in keys {
                        max_time = max_time.max(key.time);
                    }
                }
                if let Some(keys) = &timeline.rotate {
                    for key in keys {
                        max_time = max_time.max(key.time);
                    }
                }
                if let Some(keys) = &timeline.scale {
                    for key in keys {
                        max_time = max_time.max(key.time);
                    }
                }
            }
        }

        if let Some(slots) = &anim.slots {
            for timeline in slots.values() {
                if let Some(keys) = &timeline.attachment {
                    for key in keys {
                        max_time = max_time.max(key.time);
                    }
                }
            }
        }

        max_time
    }
}
