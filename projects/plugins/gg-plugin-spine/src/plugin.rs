//! Spine 动画插件模块

use gg_core::plugin::{Plugin, PluginRegistrar};
use gg_core::GResult;

use crate::systems::{SpineAnimationSystem, SpineRenderSystem};

/// Spine 动画插件
///
/// 提供 Spine 骨骼动画的播放、更新和渲染功能。
pub struct SpinePlugin {
    /// 默认帧间隔时间（秒）
    pub delta_secs: f32,
}

impl SpinePlugin {
    /// 创建新的 Spine 动画插件
    pub fn new(delta_secs: f32) -> Self {
        Self { delta_secs }
    }
}

impl Plugin for SpinePlugin {
    fn name(&self) -> &str {
        "spine"
    }

    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(SpineAnimationSystem::new(self.delta_secs)));
        registrar.register_system(Box::new(SpineRenderSystem::new()));
    }

    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
