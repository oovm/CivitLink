//! 转场系统模块
//! 实现转场系统，负责每帧更新转场进度并完成转场

use gg_core::GResult;
use gg_ecs::{System, World};
use gg_galgame_schema::components::{SlideDirection, TransitionType};
use gg_render::{DrawCommand, RenderContext, TransitionKind};

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

    /// 将转场渲染指令提交到渲染上下文
    ///
    /// 遍历所有带 `TransitionState` 的实体，
    /// 为正在进行的转场生成 `DrawCommand::Transition`。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    /// - `context` - 渲染上下文
    pub fn render_to_context(&self, world: &World, context: &mut RenderContext) -> GResult<()> {
        let entities: Vec<_> = world.entities().iter().copied().collect();
        for entity in entities {
            if let Some(state) = world.get_component::<TransitionState>(entity) {
                if !state.is_complete {
                    let kind = match &state.transition_type {
                        TransitionType::Fade { .. } => TransitionKind::Fade,
                        TransitionType::CrossDissolve { .. } => TransitionKind::CrossDissolve,
                        TransitionType::Slide { direction, .. } => match direction {
                            SlideDirection::Left => TransitionKind::SlideLeft,
                            SlideDirection::Right => TransitionKind::SlideRight,
                            SlideDirection::Up => TransitionKind::SlideUp,
                            SlideDirection::Down => TransitionKind::SlideDown,
                        },
                        TransitionType::None => continue,
                    };
                    context.draw(DrawCommand::Transition {
                        old_texture: None,
                        new_texture: None,
                        progress: state.progress,
                        kind,
                    });
                }
            }
        }
        Ok(())
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
