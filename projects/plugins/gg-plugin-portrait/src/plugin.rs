//! 立绘系统插件模块
//! 实现 Plugin trait，负责立绘系统的初始化和关闭

use gg_core::plugin::{Plugin, PluginRegistrar};
use gg_core::GResult;

use crate::systems::{PortraitAnimationSystem, PortraitRenderSystem};

/// 立绘系统插件
///
/// 负责初始化立绘系统所需的系统（PortraitRenderSystem、PortraitAnimationSystem），
/// 并在关闭时清理这些资源。
pub struct PortraitPlugin {
    /// 屏幕宽度
    pub screen_width: f32,
    /// 屏幕高度
    pub screen_height: f32,
}

impl PortraitPlugin {
    /// 创建新的立绘系统插件
    ///
    /// # 参数
    ///
    /// - `screen_width` - 屏幕宽度
    /// - `screen_height` - 屏幕高度
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
    }
}

impl Plugin for PortraitPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "portrait"
    }

    /// 构建立绘系统插件
    ///
    /// 注册立绘渲染系统和立绘动画系统。
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(PortraitRenderSystem::new(
            self.screen_width,
            self.screen_height,
        )));
        registrar.register_system(Box::new(PortraitAnimationSystem::new(1.0 / 60.0)));
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// 初始化立绘系统资源
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭立绘系统，清理资源
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
