//! 转场系统模块
//! 实现转场系统，负责每帧更新转场进度并完成转场

use gg_core::GResult;
use gg_ecs::{System, World};

use crate::transition::{TransitionManager, TransitionState};

/// 转场系统
///
/// 每帧遍历所有带 TransitionState 的实体，
/// 更新转场进度，对已完成的转场调用 TransitionManager::complete_transition。
pub struct TransitionSystem {
    /// 上一帧的时间戳（秒）
    pub last_time_secs: f64,
}

impl TransitionSystem {
    /// 创建新的转场系统
    pub fn new() -> Self {
        Self {
            last_time_secs: 0.0,
        }
    }
}

impl System for TransitionSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "transition"
    }

    /// 执行转场系统逻辑
    ///
    /// 计算帧间时间差，更新所有 TransitionState 的进度，
    /// 对已完成的转场执行完成处理。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let delta_secs = 1.0 / 60.0;

        let mut has_complete = false;

        let entities: Vec<_> = world.entities().iter().copied().collect();
        for entity in entities {
            if let Some(state) = world.get_component_mut::<TransitionState>(entity) {
                state.update(delta_secs);
                if state.is_complete {
                    has_complete = true;
                }
            }
        }

        if has_complete {
            TransitionManager::complete_transition(world)?;
        }

        Ok(())
    }
}
