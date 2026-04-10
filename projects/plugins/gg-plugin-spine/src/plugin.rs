//! Spine 动画插件模块

use gg_core::{
    GResult,
    plugin::{Plugin, PluginRegistrar},
};

use crate::systems::{SpineAnimationSystem, SpineRenderSystem};

/// Spine 动画插件
///
/// 提供 Spine 骨骼动画的播放、更新和渲染功能。
pub struct SpinePlugin;

impl SpinePlugin {
    /// 创建新的 Spine 动画插件
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for SpinePlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "spine"
    }

    /// 构建 Spine 动画插件
    ///
    /// 注册 Spine 动画更新系统和渲染系统。
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(SpineAnimationSystem::new()));
        registrar.register_system(Box::new(SpineRenderSystem::new()));
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        vec!["render"]
    }

    /// 初始化 Spine 动画插件
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭 Spine 动画插件
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
