//! 动画瓦片模块
//! 提供帧动画瓦片的自动切换功能

use gg_core::GResult;
use gg_ecs::System;
use serde::{Deserialize, Serialize};

/// 动画瓦片组件
///
/// 附加到瓦片实体上，使瓦片按帧序列自动切换图集坐标。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimatedTile {
    /// 动画帧列表（每帧为图集中的列索引和行索引）
    pub frames: Vec<(u32, u32)>,
    /// 帧间隔时间（秒）
    pub frame_duration_secs: f32,
    /// 当前帧索引
    pub current_frame: usize,
    /// 已过时间（秒）
    pub elapsed: f32,
}

/// 瓦片动画系统
///
/// 每帧更新所有 AnimatedTile 组件的当前帧和已过时间。
pub struct TileAnimationSystem;

impl TileAnimationSystem {
    /// 创建新的瓦片动画系统
    pub fn new() -> Self {
        Self
    }
}

impl System for TileAnimationSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "tile_animation"
    }

    /// 执行瓦片动画系统逻辑
    ///
    /// 使用默认帧间隔更新所有 AnimatedTile 组件的当前帧。
    fn execute(&mut self, world: &mut gg_ecs::World) -> GResult<()> {
        let delta_secs = 1.0 / 60.0;

        for &entity in world.entities() {
            if let Some(anim) = world.get_component_mut::<AnimatedTile>(entity) {
                if anim.frames.is_empty() {
                    continue;
                }
                anim.elapsed += delta_secs;
                while anim.elapsed >= anim.frame_duration_secs {
                    anim.elapsed -= anim.frame_duration_secs;
                    anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
                }
            }
        }
        Ok(())
    }
}
