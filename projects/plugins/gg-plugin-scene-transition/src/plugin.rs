//! 场景转场插件模块
//! 实现 Plugin trait，负责场景转场系统的初始化和关闭

use gg_core::plugin::Plugin;
use gg_core::GResult;

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

    /// 初始化场景转场系统
    fn initialize(&self) -> GResult<()> {
        Ok(())
    }

    /// 关闭场景转场系统，清理资源
    fn shutdown(&self) -> GResult<()> {
        Ok(())
    }
}
