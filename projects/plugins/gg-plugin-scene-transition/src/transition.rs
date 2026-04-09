//! 转场状态和管理模块
//! 提供转场状态组件和转场管理器

use gg_core::GResult;
use gg_ecs::{Component, Entity, World};
use gg_galgame_schema::components::{SceneBackground, SlideDirection, TransitionType};

/// 转场状态组件
///
/// 附加到实体上表示正在进行的场景转场，
/// 包含转场类型、进度和目标背景路径等信息。
pub struct TransitionState {
    /// 转场类型
    pub transition_type: TransitionType,
    /// 转场进度（0.0 到 1.0）
    pub progress: f32,
    /// 转场总时长（秒）
    pub duration_secs: f32,
    /// 已经过时间（秒）
    pub elapsed_secs: f32,
    /// 转场是否已完成
    pub is_complete: bool,
    /// 新背景资源路径
    pub new_background_path: Option<String>,
}

impl Component for TransitionState {}

impl TransitionState {
    /// 创建新的转场状态
    ///
    /// 从 TransitionType 中提取持续时间，并设置新背景路径。
    pub fn new(transition_type: TransitionType, new_background_path: Option<String>) -> Self {
        let duration_secs = match &transition_type {
            TransitionType::None => 0.0,
            TransitionType::Fade { duration_secs } => *duration_secs,
            TransitionType::CrossDissolve { duration_secs } => *duration_secs,
            TransitionType::Slide { duration_secs, .. } => *duration_secs,
        };
        Self {
            transition_type,
            progress: 0.0,
            duration_secs,
            elapsed_secs: 0.0,
            is_complete: false,
            new_background_path,
        }
    }

    /// 更新转场进度
    ///
    /// 根据经过的时间增量更新进度值，当进度达到 1.0 时标记为完成。
    pub fn update(&mut self, delta_secs: f32) {
        if self.is_complete {
            return;
        }
        self.elapsed_secs += delta_secs;
        if self.duration_secs <= 0.0 {
            self.progress = 1.0;
            self.is_complete = true;
            return;
        }
        self.progress = (self.elapsed_secs / self.duration_secs).min(1.0);
        if self.progress >= 1.0 {
            self.is_complete = true;
        }
    }

    /// 获取当前淡入淡出的透明度值
    ///
    /// 对于 Fade 和 CrossDissolve 类型返回进度值，
    /// 其他类型返回 1.0。
    pub fn current_alpha(&self) -> f32 {
        match &self.transition_type {
            TransitionType::Fade { .. } | TransitionType::CrossDissolve { .. } => self.progress,
            _ => 1.0,
        }
    }

    /// 获取当前滑动偏移量
    ///
    /// 对于 Slide 类型，根据滑动方向和进度返回 (x, y) 偏移量；
    /// 其他类型返回 (0.0, 0.0)。
    pub fn current_offset(&self) -> (f32, f32) {
        match &self.transition_type {
            TransitionType::Slide { direction, .. } => {
                let offset = 1.0 - self.progress;
                match direction {
                    SlideDirection::Left => (-offset, 0.0),
                    SlideDirection::Right => (offset, 0.0),
                    SlideDirection::Up => (0.0, -offset),
                    SlideDirection::Down => (0.0, offset),
                }
            }
            _ => (0.0, 0.0),
        }
    }
}

/// 转场管理器
///
/// 提供静态方法用于启动、查询和完成场景转场。
pub struct TransitionManager;

impl TransitionManager {
    /// 启动转场
    ///
    /// 在 World 中创建新实体并添加 TransitionState 组件，
    /// 返回创建的实体 ID。
    pub fn start_transition(
        world: &mut World,
        new_background_path: String,
        transition_type: TransitionType,
    ) -> GResult<Entity> {
        let entity = world.spawn();
        let state = TransitionState::new(transition_type, Some(new_background_path));
        world.add_component(entity, state)?;
        Ok(entity)
    }

    /// 检查是否有正在进行的转场
    ///
    /// 遍历所有实体，查找是否有未完成的 TransitionState 组件。
    pub fn is_transitioning(world: &World) -> bool {
        for &entity in world.entities().iter() {
            if let Some(state) = world.get_component::<TransitionState>(entity) {
                if !state.is_complete {
                    return true;
                }
            }
        }
        false
    }

    /// 完成转场
    ///
    /// 查找已完成的 TransitionState，更新 SceneBackground 的 asset_path，
    /// 然后移除 TransitionState 组件。
    pub fn complete_transition(world: &mut World) -> GResult<()> {
        let mut completed_entities = Vec::new();
        let mut new_path: Option<String> = None;

        for &entity in world.entities().iter() {
            if let Some(state) = world.get_component::<TransitionState>(entity) {
                if state.is_complete {
                    new_path = state.new_background_path.clone();
                    completed_entities.push(entity);
                }
            }
        }

        if let Some(path) = new_path {
            let bg_entities: Vec<_> = world.entities().iter().copied().collect();
            for entity in bg_entities {
                if let Some(background) = world.get_component_mut::<SceneBackground>(entity) {
                    background.asset_path = Some(path.clone());
                    break;
                }
            }
        }

        for entity in completed_entities {
            world.remove_component::<TransitionState>(entity);
        }

        Ok(())
    }
}
