//! 场景转场插件模块
//! 实现 Plugin trait，负责场景转场系统的初始化和关闭

use gg_core::plugin::{Plugin, PluginRegistrar};
use gg_core::GResult;

use crate::systems::TransitionSystem;

/// 场景转场插件
///
/// 负责初始化场景转场系统，
/// 提供场景切换时的转场效果和氛围滤镜过渡功能。
pub struct SceneTransitionPlugin;

impl Plugin for SceneTransitionPlugin {
    /// 返回插件名称
    fn name(&self) -> &str {
        "scene-transition"
    }

    /// 构建场景转场插件
    ///
    /// 注册转场系统。
    fn build(&self, registrar: &mut PluginRegistrar) {
        registrar.register_system(Box::new(TransitionSystem::new()));
    }

    /// 返回插件依赖列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// 初始化场景转场系统
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭场景转场系统，清理资源
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
