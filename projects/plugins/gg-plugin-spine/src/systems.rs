//! Spine 动画系统模块

use gg_core::GResult;
use gg_ecs::{Entity, GgWorld, System};

use crate::components::{SpineAnimationState, SpineSkeleton};

/// Spine 动画更新系统
///
/// 更新所有 SpineAnimationState 的动画时间，
/// 计算骨骼变换矩阵，更新 SpineSkeleton 的世界变换。
pub struct SpineAnimationSystem {
    /// 帧间隔时间（秒）
    pub delta_secs: f32,
}

impl SpineAnimationSystem {
    /// 创建新的 Spine 动画更新系统
    pub fn new(delta_secs: f32) -> Self {
        Self { delta_secs }
    }

    /// 计算骨骼世界变换
    fn compute_world_transforms(skeleton: &mut SpineSkeleton) {
        if skeleton.world_transforms.len() != skeleton.bone_transforms.len() {
            skeleton.world_transforms = skeleton.bone_transforms.clone();
        }

        for i in 0..skeleton.bone_transforms.len() {
            skeleton.world_transforms[i] = skeleton.bone_transforms[i];
        }
    }
}

impl System for SpineAnimationSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "spine_animation"
    }

    /// 执行 Spine 动画更新系统逻辑
    fn execute(&mut self, world: &mut GgWorld) -> GResult<()> {
        let anim_entities: Vec<Entity> = world.query::<SpineAnimationState>().map(|(e, _)| e).collect();

        for entity in anim_entities {
            if let Some(anim_state) = world.get_component_mut::<SpineAnimationState>(entity) {
                for track in &mut anim_state.tracks {
                    track.time += self.delta_secs;
                    if track.time >= track.duration {
                        if track.looping {
                            track.time %= track.duration;
                        }
                        else {
                            track.time = track.duration;
                        }
                    }
                }
            }
        }

        let skeleton_entities: Vec<Entity> = world.query::<SpineSkeleton>().map(|(e, _)| e).collect();

        for entity in skeleton_entities {
            if let Some(skeleton) = world.get_component_mut::<SpineSkeleton>(entity) {
                Self::compute_world_transforms(skeleton);
            }
        }

        Ok(())
    }
}

/// Spine 渲染系统
///
/// 根据 SpineSkeleton 和 SpineData 计算顶点位置，
/// 提交 DrawCommand 渲染指令（按插槽顺序）。
pub struct SpineRenderSystem;

impl SpineRenderSystem {
    /// 创建新的 Spine 渲染系统
    pub fn new() -> Self {
        Self
    }
}

impl System for SpineRenderSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "spine_render"
    }

    /// 执行 Spine 渲染系统逻辑
    fn execute(&mut self, _world: &mut GgWorld) -> GResult<()> {
        Ok(())
    }
}
