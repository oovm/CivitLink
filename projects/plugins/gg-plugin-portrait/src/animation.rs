//! 立绘动画模块
//! 提供立绘动画状态组件和立绘管理器，负责立绘的显示、隐藏、表情切换和高亮管理

use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::{Entity, World};
use gg_galgame_schema::components::{PortraitPosition, PortraitState, SlideDirection, TransitionType};
use gg_render::TextureId;

/// 立绘动画状态组件
///
/// 跟踪单个立绘实体的动画进度，包括淡入淡出、交叉溶解和滑动等过渡效果。
pub struct PortraitAnimationState {
    /// 动画类型
    pub animation_type: TransitionType,
    /// 动画进度（0.0 到 1.0）
    pub progress: f32,
    /// 动画总时长（秒）
    pub duration_secs: f32,
    /// 已经过时间（秒）
    pub elapsed_secs: f32,
    /// 动画是否已完成
    pub is_complete: bool,
    /// 是否为淡出动画
    pub is_fading_out: bool,
}

impl PortraitAnimationState {
    /// 创建新的立绘动画状态
    ///
    /// 从 `TransitionType` 中提取动画时长，并初始化进度为零。
    /// 对于 `TransitionType::None`，动画将立即标记为完成。
    pub fn new(animation_type: TransitionType) -> Self {
        let duration_secs = match &animation_type {
            TransitionType::None => 0.0,
            TransitionType::Fade { duration_secs } => *duration_secs,
            TransitionType::CrossDissolve { duration_secs } => *duration_secs,
            TransitionType::Slide { duration_secs, .. } => *duration_secs,
        };
        let is_complete = matches!(animation_type, TransitionType::None);
        Self { animation_type, progress: 0.0, duration_secs, elapsed_secs: 0.0, is_complete, is_fading_out: false }
    }

    /// 创建淡出动画状态
    ///
    /// 使用指定的过渡类型创建动画，并将 `is_fading_out` 设置为 `true`。
    /// 淡出时透明度从 1 减少到 0。
    ///
    /// # 参数
    ///
    /// - `animation_type` - 过渡动画类型
    pub fn new_fade_out(animation_type: TransitionType) -> Self {
        let mut state = Self::new(animation_type);
        state.is_fading_out = true;
        state
    }

    /// 更新动画进度
    ///
    /// 根据经过的时间推进动画进度，当进度达到 1.0 时标记为完成。
    /// 对于零时长的动画，直接标记为完成。
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

    /// 获取当前透明度
    ///
    /// 对于淡入淡出和交叉溶解动画，返回当前进度作为透明度值。
    /// 淡入时透明度从 0 增长到 1（即 progress），
    /// 淡出时透明度从 1 减少到 0（即 1.0 - progress）。
    /// 对于其他动画类型，返回 1.0。
    pub fn current_opacity(&self) -> f32 {
        match &self.animation_type {
            TransitionType::Fade { .. } => {
                if self.is_fading_out {
                    1.0 - self.progress
                } else {
                    self.progress
                }
            }
            TransitionType::CrossDissolve { .. } => self.progress,
            _ => 1.0,
        }
    }

    /// 获取当前偏移量
    ///
    /// 对于滑动动画，返回归一化的偏移量（0.0 到 1.0 范围），
    /// 调用方需乘以屏幕尺寸以获得实际像素偏移。
    /// 动画完成时偏移量为零。
    pub fn current_offset(&self) -> (f32, f32) {
        if self.is_complete {
            return (0.0, 0.0);
        }
        match &self.animation_type {
            TransitionType::Slide { direction, .. } => {
                let remaining = 1.0 - self.progress;
                match direction {
                    SlideDirection::Left => (remaining, 0.0),
                    SlideDirection::Right => (-remaining, 0.0),
                    SlideDirection::Up => (0.0, remaining),
                    SlideDirection::Down => (0.0, -remaining),
                }
            }
            _ => (0.0, 0.0),
        }
    }
}

/// 立绘管理器
///
/// 提供立绘的显示、隐藏、表情切换和高亮管理等静态方法。
/// 所有方法均通过操作 ECS World 中的实体和组件来实现。
pub struct PortraitManager;

impl PortraitManager {
    /// 显示立绘
    ///
    /// 在 World 中创建新实体，添加 `PortraitState` 和 `PortraitAnimationState` 组件。
    /// 立绘初始状态为非高亮、缩放 1.0、透明度 1.0。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    /// - `character_id` - 角色 ID
    /// - `expression` - 表情标签
    /// - `position` - 立绘位置
    /// - `transition` - 过渡动画类型
    ///
    /// # 返回
    ///
    /// 新创建的实体 ID
    pub fn show_portrait(
        world: &mut World,
        character_id: String,
        expression: String,
        position: PortraitPosition,
        transition: TransitionType,
    ) -> GResult<Entity> {
        let entity = world.spawn().id();
        let state = PortraitState {
            character_id,
            current_expression: expression,
            position,
            scale: 1.0,
            opacity: 1.0,
            is_speaking: false,
            z_order: 0,
            texture_id: TextureId::INVALID,
            texture_width: 200.0,
            texture_height: 400.0,
        };
        world.add_component(entity, state)?;
        let animation = PortraitAnimationState::new(transition);
        world.add_component(entity, animation)?;
        Ok(entity)
    }

    /// 隐藏立绘
    ///
    /// 查找带有对应 `character_id` 的 `PortraitState` 实体，
    /// 为其添加退出动画。动画完成后由动画系统移除组件。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    /// - `character_id` - 角色 ID
    /// - `transition` - 过渡动画类型
    pub fn hide_portrait(world: &mut World, character_id: String, transition: TransitionType) -> GResult<()> {
        let entities: Vec<Entity> = world.entities().iter().copied().collect();
        let mut found = None;
        for entity in entities {
            if let Some(state) = world.get_component::<PortraitState>(entity) {
                if state.character_id == character_id {
                    found = Some(entity);
                    break;
                }
            }
        }
        if let Some(entity) = found {
            let animation = PortraitAnimationState::new(transition);
            world.add_component(entity, animation)?;
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Plugin, message: format!("Portrait not found for character: {}", character_id) })
        }
    }

    /// 切换立绘表情
    ///
    /// 查找带有对应 `character_id` 的 `PortraitState` 实体，
    /// 更新其 `current_expression` 字段，并添加过渡动画。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    /// - `character_id` - 角色 ID
    /// - `expression` - 新表情标签
    /// - `transition` - 过渡动画类型
    pub fn change_expression(
        world: &mut World,
        character_id: String,
        expression: String,
        transition: TransitionType,
    ) -> GResult<()> {
        let entities: Vec<Entity> = world.entities().iter().copied().collect();
        let mut found = None;
        for entity in entities {
            if let Some(state) = world.get_component::<PortraitState>(entity) {
                if state.character_id == character_id {
                    found = Some(entity);
                    break;
                }
            }
        }
        if let Some(entity) = found {
            if let Some(state) = world.get_component_mut::<PortraitState>(entity) {
                state.current_expression = expression;
            }
            let animation = PortraitAnimationState::new(transition);
            world.add_component(entity, animation)?;
            Ok(())
        }
        else {
            Err(GError { kind: GErrorKind::Plugin, message: format!("Portrait not found for character: {}", character_id) })
        }
    }

    /// 高亮说话角色
    ///
    /// 设置对应 `character_id` 的 `PortraitState` 的 `is_speaking` 为 `true`，
    /// 同时将所有其他立绘的 `is_speaking` 设置为 `false`。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    /// - `character_id` - 说话角色 ID
    pub fn highlight_speaker(world: &mut World, character_id: &str) -> GResult<()> {
        let entities: Vec<Entity> = world.entities().iter().copied().collect();
        for entity in entities {
            if let Some(state) = world.get_component_mut::<PortraitState>(entity) {
                state.is_speaking = state.character_id == character_id;
            }
        }
        Ok(())
    }

    /// 清除所有高亮
    ///
    /// 将所有 `PortraitState` 的 `is_speaking` 设置为 `false`。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    pub fn clear_highlights(world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.entities().iter().copied().collect();
        for entity in entities {
            if let Some(state) = world.get_component_mut::<PortraitState>(entity) {
                state.is_speaking = false;
            }
        }
        Ok(())
    }
}
