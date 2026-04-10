//! 氛围滤镜状态和管理模块
//! 提供滤镜状态组件、滤镜系统和滤镜管理器

use gg_core::GResult;
use gg_ecs::{Component, System, World};
use gg_galgame_schema::components::{AmbientFilter, SceneBackground};

/// 氛围滤镜过渡状态组件
///
/// 附加到实体上表示正在进行的氛围滤镜过渡，
/// 包含源滤镜、目标滤镜和过渡进度等信息。
pub struct FilterState {
    /// 源滤镜
    pub source_filter: Option<AmbientFilter>,
    /// 目标滤镜
    pub target_filter: Option<AmbientFilter>,
    /// 过渡进度（0.0 到 1.0）
    pub progress: f32,
    /// 过渡总时长（秒）
    pub duration_secs: f32,
    /// 已经过时间（秒）
    pub elapsed_secs: f32,
    /// 过渡是否已完成
    pub is_complete: bool,
}

impl Component for FilterState {}

impl FilterState {
    /// 创建新的滤镜过渡状态
    ///
    /// 指定源滤镜、目标滤镜和过渡时长。
    pub fn new(source: Option<AmbientFilter>, target: Option<AmbientFilter>, duration_secs: f32) -> Self {
        Self {
            source_filter: source,
            target_filter: target,
            progress: 0.0,
            duration_secs,
            elapsed_secs: 0.0,
            is_complete: false,
        }
    }

    /// 更新滤镜过渡进度
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

    /// 获取当前插值后的滤镜
    ///
    /// 根据进度在源滤镜和目标滤镜之间进行线性插值：
    /// - Darken/Warm/Cool: 线性插值 intensity
    /// - Blur: 线性插值 radius
    /// - Tint: 线性插值 color 各分量
    /// - 如果源和目标类型不同，当进度 >= 0.5 时返回目标滤镜
    pub fn current_filter(&self) -> Option<AmbientFilter> {
        let t = self.progress;

        match (&self.source_filter, &self.target_filter) {
            (None, None) => None,
            (Some(source), None) => {
                if t >= 1.0 {
                    None
                }
                else {
                    Some(Self::interpolate_towards_none(source, t))
                }
            }
            (None, Some(target)) => {
                if t >= 1.0 {
                    Some(target.clone())
                }
                else {
                    Some(Self::interpolate_from_none(target, t))
                }
            }
            (Some(source), Some(target)) => {
                if t >= 1.0 {
                    Some(target.clone())
                }
                else {
                    Some(Self::interpolate_filters(source, target, t))
                }
            }
        }
    }

    /// 在滤镜和 None 之间插值（滤镜淡出）
    fn interpolate_towards_none(filter: &AmbientFilter, t: f32) -> AmbientFilter {
        let fade = 1.0 - t;
        match filter {
            AmbientFilter::Darken { intensity } => AmbientFilter::Darken { intensity: *intensity * fade },
            AmbientFilter::Warm { intensity } => AmbientFilter::Warm { intensity: *intensity * fade },
            AmbientFilter::Cool { intensity } => AmbientFilter::Cool { intensity: *intensity * fade },
            AmbientFilter::Blur { radius } => AmbientFilter::Blur { radius: *radius * fade },
            AmbientFilter::Tint { color } => {
                AmbientFilter::Tint { color: [color[0] * fade, color[1] * fade, color[2] * fade, color[3] * fade] }
            }
        }
    }

    /// 在 None 和滤镜之间插值（滤镜淡入）
    fn interpolate_from_none(filter: &AmbientFilter, t: f32) -> AmbientFilter {
        match filter {
            AmbientFilter::Darken { intensity } => AmbientFilter::Darken { intensity: *intensity * t },
            AmbientFilter::Warm { intensity } => AmbientFilter::Warm { intensity: *intensity * t },
            AmbientFilter::Cool { intensity } => AmbientFilter::Cool { intensity: *intensity * t },
            AmbientFilter::Blur { radius } => AmbientFilter::Blur { radius: *radius * t },
            AmbientFilter::Tint { color } => {
                AmbientFilter::Tint { color: [color[0] * t, color[1] * t, color[2] * t, color[3] * t] }
            }
        }
    }

    /// 在两个同类型滤镜之间插值
    fn interpolate_filters(source: &AmbientFilter, target: &AmbientFilter, t: f32) -> AmbientFilter {
        match (source, target) {
            (AmbientFilter::Darken { intensity: s }, AmbientFilter::Darken { intensity: e }) => {
                AmbientFilter::Darken { intensity: s + (e - s) * t }
            }
            (AmbientFilter::Warm { intensity: s }, AmbientFilter::Warm { intensity: e }) => {
                AmbientFilter::Warm { intensity: s + (e - s) * t }
            }
            (AmbientFilter::Cool { intensity: s }, AmbientFilter::Cool { intensity: e }) => {
                AmbientFilter::Cool { intensity: s + (e - s) * t }
            }
            (AmbientFilter::Blur { radius: s }, AmbientFilter::Blur { radius: e }) => {
                AmbientFilter::Blur { radius: s + (e - s) * t }
            }
            (AmbientFilter::Tint { color: s }, AmbientFilter::Tint { color: e }) => AmbientFilter::Tint {
                color: [s[0] + (e[0] - s[0]) * t, s[1] + (e[1] - s[1]) * t, s[2] + (e[2] - s[2]) * t, s[3] + (e[3] - s[3]) * t],
            },
            _ => {
                if t >= 0.5 {
                    target.clone()
                }
                else {
                    source.clone()
                }
            }
        }
    }
}

/// 氛围滤镜系统
///
/// 每帧遍历所有带 FilterState 的实体，
/// 更新滤镜过渡进度，完成后更新 SceneBackground 的 ambient_filter。
pub struct FilterSystem;

impl FilterSystem {
    /// 创建新的氛围滤镜系统
    pub fn new() -> Self {
        Self
    }
}

impl System for FilterSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "filter"
    }

    /// 执行氛围滤镜系统逻辑
    ///
    /// 更新所有 FilterState 的进度，
    /// 对已完成的滤镜过渡更新 SceneBackground 的 ambient_filter 并移除 FilterState。
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let delta_secs = 1.0 / 60.0;

        let mut completed_entities = Vec::new();
        let mut completed_target: Option<Option<AmbientFilter>> = None;

        let entities: Vec<_> = world.entities().iter().copied().collect();
        for entity in entities {
            if let Some(state) = world.get_component_mut::<FilterState>(entity) {
                state.update(delta_secs);
                if state.is_complete {
                    completed_target = Some(state.target_filter.clone());
                    completed_entities.push(entity);
                }
            }
        }

        if let Some(target_filter) = completed_target {
            let bg_entities: Vec<_> = world.entities().iter().copied().collect();
            for entity in bg_entities {
                if let Some(background) = world.get_component_mut::<SceneBackground>(entity) {
                    background.ambient_filter = target_filter;
                    break;
                }
            }
        }

        for entity in completed_entities {
            world.remove_component::<FilterState>(entity);
        }

        Ok(())
    }
}

/// 氛围滤镜管理器
///
/// 提供静态方法用于应用和移除氛围滤镜。
pub struct FilterManager;

impl FilterManager {
    /// 应用氛围滤镜
    ///
    /// 查找 SceneBackground 组件，获取当前滤镜作为源，
    /// 创建 FilterState 实体进行过渡。
    pub fn apply_filter(world: &mut World, filter: AmbientFilter, duration_secs: f32) -> GResult<()> {
        let current_filter = Self::get_current_filter(world);

        let entity = world.spawn().id();
        let state = FilterState::new(current_filter, Some(filter), duration_secs);
        world.add_component(entity, state)?;
        Ok(())
    }

    /// 移除氛围滤镜
    ///
    /// 查找 SceneBackground 组件，获取当前滤镜作为源，
    /// 创建 FilterState 实体过渡到 None。
    pub fn remove_filter(world: &mut World, duration_secs: f32) -> GResult<()> {
        let current_filter = Self::get_current_filter(world);

        let entity = world.spawn().id();
        let state = FilterState::new(current_filter, None, duration_secs);
        world.add_component(entity, state)?;
        Ok(())
    }

    /// 获取当前 SceneBackground 的 ambient_filter
    fn get_current_filter(world: &World) -> Option<AmbientFilter> {
        for &entity in world.entities().iter() {
            if let Some(background) = world.get_component::<SceneBackground>(entity) {
                return background.ambient_filter.clone();
            }
        }
        None
    }
}
