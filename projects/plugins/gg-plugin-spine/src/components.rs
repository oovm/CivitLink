//! Spine 骨骼动画核心组件模块

use gg_ecs::Component;
use serde::{Deserialize, Serialize};

/// 骨骼变换
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoneTransform {
    /// X 偏移
    pub x: f32,
    /// Y 偏移
    pub y: f32,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// X 缩放
    pub scale_x: f32,
    /// Y 缩放
    pub scale_y: f32,
}

impl Default for BoneTransform {
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

/// Spine 骨骼组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineSkeleton {
    /// 骨骼数据资源名称
    pub data_name: String,
    /// 当前动画名称
    pub current_animation: Option<String>,
    /// 动画混合时间（秒）
    pub mix_duration: f32,
    /// 骨骼局部变换列表
    pub bone_transforms: Vec<BoneTransform>,
    /// 骨骼世界变换列表
    pub world_transforms: Vec<BoneTransform>,
}

impl Component for SpineSkeleton {}

/// 动画轨道状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrack {
    /// 动画名称
    pub animation_name: String,
    /// 当前播放时间
    pub time: f32,
    /// 动画总时长
    pub duration: f32,
    /// 是否循环播放
    pub looping: bool,
}

/// Spine 动画状态组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineAnimationState {
    /// 动画轨道列表
    pub tracks: Vec<AnimationTrack>,
}

impl Component for SpineAnimationState {}
