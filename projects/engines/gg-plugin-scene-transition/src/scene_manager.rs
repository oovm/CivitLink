//! 场景管理器模块
//! 提供场景切换、背景查询和氛围滤镜设置的高层接口

use gg_core::GResult;
use gg_ecs::World;
use gg_galgame_schema::components::{AmbientFilter, SceneBackground, TransitionType};

use crate::{filter::FilterManager, transition::TransitionManager};

/// 场景管理器
///
/// 提供场景切换、背景查询和氛围滤镜设置的静态方法，
/// 作为场景转场插件的高层接口。
pub struct SceneManager;

impl SceneManager {
    /// 切换场景
    ///
    /// 使用指定的转场效果切换到新背景，可选地同时应用氛围滤镜。
    pub fn change_scene(
        world: &mut World,
        background_path: String,
        transition: TransitionType,
        ambient_filter: Option<AmbientFilter>,
    ) -> GResult<()> {
        TransitionManager::start_transition(world, background_path, transition)?;

        if let Some(filter) = ambient_filter {
            FilterManager::apply_filter(world, filter, 1.0)?;
        }

        Ok(())
    }

    /// 获取当前背景资源路径
    ///
    /// 查找 SceneBackground 组件并返回其 asset_path。
    pub fn get_current_background(world: &World) -> Option<String> {
        for &entity in world.entities().iter() {
            if let Some(background) = world.get_component::<SceneBackground>(entity) {
                return background.asset_path.clone();
            }
        }
        None
    }

    /// 设置氛围滤镜
    ///
    /// 通过 FilterManager 应用新的氛围滤镜，指定过渡时长。
    pub fn set_ambient_filter(world: &mut World, filter: AmbientFilter, duration_secs: f32) -> GResult<()> {
        FilterManager::apply_filter(world, filter, duration_secs)
    }
}
