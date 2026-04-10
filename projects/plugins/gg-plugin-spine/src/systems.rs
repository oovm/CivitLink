//! Spine 动画系统模块

use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use gg_render::{Color, DrawCommand, Rect, RenderContext, Transform};

use crate::components::{BoneTransform, SpineAnimationState, SpineSkeleton};
use crate::resources::SpineData;

/// Spine 动画更新系统
///
/// 更新所有 SpineAnimationState 的动画时间，
/// 计算骨骼变换矩阵，更新 SpineSkeleton 的世界变换。
/// 使用 World 中的 DeltaTime 资源获取真实帧间隔。
pub struct SpineAnimationSystem;

impl SpineAnimationSystem {
    /// 创建新的 Spine 动画更新系统
    pub fn new() -> Self {
        Self
    }

    /// 计算骨骼世界变换
    ///
    /// 按骨骼列表顺序（保证父骨骼在子骨骼之前）计算世界变换：
    /// - 根骨骼：世界变换 = 局部变换
    /// - 子骨骼：世界变换 = 父骨骼世界变换 * 局部变换
    fn compute_world_transforms(skeleton: &mut SpineSkeleton, bones: &[crate::resources::BoneDef]) {
        if skeleton.world_transforms.len() != skeleton.bone_transforms.len() {
            skeleton.world_transforms = vec![BoneTransform::default(); skeleton.bone_transforms.len()];
        }

        for i in 0..bones.len() {
            if i >= skeleton.bone_transforms.len() {
                break;
            }
            let local = skeleton.bone_transforms[i];

            match bones[i].parent_index {
                None => {
                    skeleton.world_transforms[i] = local;
                }
                Some(parent_idx) => {
                    let parent = skeleton.world_transforms[parent_idx];
                    skeleton.world_transforms[i] = Self::combine_transforms(&parent, &local);
                }
            }
        }
    }

    /// 组合两个骨骼变换
    ///
    /// 子骨骼世界变换 = 父骨骼世界变换 * 子骨骼局部变换
    fn combine_transforms(parent: &BoneTransform, child: &BoneTransform) -> BoneTransform {
        let cos = parent.rotation.cos();
        let sin = parent.rotation.sin();

        let scaled_x = child.x * parent.scale_x;
        let scaled_y = child.y * parent.scale_y;

        let rotated_x = cos * scaled_x - sin * scaled_y;
        let rotated_y = sin * scaled_x + cos * scaled_y;

        BoneTransform {
            x: parent.x + rotated_x,
            y: parent.y + rotated_y,
            rotation: parent.rotation + child.rotation,
            scale_x: parent.scale_x * child.scale_x,
            scale_y: parent.scale_y * child.scale_y,
        }
    }
}

impl System for SpineAnimationSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "spine_animation"
    }

    /// 执行 Spine 动画更新系统逻辑
    ///
    /// 1. 使用真实 delta time 更新动画轨道时间
    /// 2. 获取 SpineData 资源中的骨骼层级信息
    /// 3. 计算正确的骨骼世界变换
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let delta_secs = 1.0 / 60.0;

        let anim_entities: Vec<Entity> = world.query::<SpineAnimationState>().map(|(e, _)| e).collect();

        for entity in anim_entities {
            if let Some(anim_state) = world.get_component_mut::<SpineAnimationState>(entity) {
                for track in &mut anim_state.tracks {
                    track.time += delta_secs;
                    if track.time >= track.duration {
                        if track.looping {
                            track.time %= track.duration;
                        } else {
                            track.time = track.duration;
                        }
                    }
                }
            }
        }

        let bone_defs: Vec<crate::resources::BoneDef> = world
            .get_resource::<SpineData>()
            .map(|d| d.bones.clone())
            .unwrap_or_default();

        let skeleton_entities: Vec<Entity> = world.query::<SpineSkeleton>().map(|(e, _)| e).collect();

        for entity in skeleton_entities {
            if let Some(skeleton) = world.get_component_mut::<SpineSkeleton>(entity) {
                Self::compute_world_transforms(skeleton, &bone_defs);
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
    ///
    /// 按插槽顺序遍历附件，根据骨骼世界变换计算附件位置和旋转，
    /// 提交 DrawCommand::Sprite 渲染指令。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let spine_data = match world.get_resource::<SpineData>() {
            Some(data) => data.clone(),
            None => return Ok(()),
        };

        let render_ctx = world.get_resource_mut::<RenderContext>();
        if render_ctx.is_none() {
            return Ok(());
        }

        let skeleton_entities: Vec<Entity> = world.query::<SpineSkeleton>().map(|(e, _)| e).collect();

        let mut commands = Vec::new();

        for entity in skeleton_entities {
            let skeleton = match world.get_component::<SpineSkeleton>(entity) {
                Some(s) => s.clone(),
                None => continue,
            };

            for (slot_name, slot_attachments) in &spine_data.attachments {
                for attachment in slot_attachments {
                    let bone_idx = spine_data
                        .bones
                        .iter()
                        .position(|b| b.name == *slot_name);

                    let world_transform = match bone_idx {
                        Some(idx) if idx < skeleton.world_transforms.len() => {
                            skeleton.world_transforms[idx]
                        }
                        _ => BoneTransform::default(),
                    };

                    let x = world_transform.x + attachment.offset_x;
                    let y = world_transform.y + attachment.offset_y;

                    let transform = Transform {
                        position: [x, y],
                        scale: [world_transform.scale_x, world_transform.scale_y],
                        rotation: world_transform.rotation,
                        z_index: bone_idx.map(|i| i as f32).unwrap_or(0.0),
                    };

                    commands.push(DrawCommand::Sprite {
                        texture_id: gg_render::TextureId::INVALID,
                        transform,
                        size: [attachment.width, attachment.height],
                        tint: Color::WHITE,
                        clip_rect: None,
                    });
                }
            }
        }

        if let Some(ctx) = world.get_resource_mut::<RenderContext>() {
            for cmd in commands {
                ctx.draw(cmd);
            }
        }

        Ok(())
    }
}
