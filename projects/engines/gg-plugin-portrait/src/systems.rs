//! 立绘系统模块
//! 实现立绘渲染系统和立绘动画系统

use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use gg_galgame_schema::components::PortraitState;
use gg_plugin_dialogue::schema::DeltaTime;
use gg_render::{Color, DrawCommand, RenderContext, TextureId, Transform};

use crate::{animation::PortraitAnimationState, layout::PortraitLayout};

/// 非说话立绘的透明度
const DIMMED_OPACITY: f32 = 0.6;

/// 立绘渲染系统
///
/// 负责每帧更新立绘的渲染状态：
/// - 遍历所有带 `PortraitState` 的实体
/// - 按 `z_order` 排序
/// - 计算立绘位置
/// - 应用高亮效果（说话角色全亮度，非说话角色降低透明度）
pub struct PortraitRenderSystem {
    /// 屏幕宽度
    pub screen_width: f32,
    /// 屏幕高度
    pub screen_height: f32,
}

impl PortraitRenderSystem {
    /// 创建新的立绘渲染系统
    ///
    /// # 参数
    ///
    /// - `screen_width` - 屏幕宽度
    /// - `screen_height` - 屏幕高度
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self { screen_width, screen_height }
    }

    /// 将立绘渲染指令提交到渲染上下文
    ///
    /// 遍历所有带 `PortraitState` 的实体，
    /// 按 `z_order` 排序后为每个立绘生成 `DrawCommand::Sprite`。
    ///
    /// # 参数
    ///
    /// - `world` - ECS 世界
    /// - `context` - 渲染上下文
    pub fn render_to_context(&self, world: &World, context: &mut RenderContext) -> GResult<()> {
        let entities: Vec<Entity> = world.entities().iter().copied().collect();
        let mut portraits: Vec<(Entity, PortraitState)> = Vec::new();
        for entity in entities {
            if let Some(state) = world.get_component::<PortraitState>(entity) {
                portraits.push((entity, state.clone()));
            }
        }
        portraits.sort_by_key(|(_, state)| state.z_order);
        let layout = PortraitLayout::new(self.screen_width, self.screen_height);
        for (entity, state) in &portraits {
            let (x, y) = layout.calculate_position(&state.position);
            let (offset_x, offset_y) =
                world.get_component::<PortraitAnimationState>(*entity).map(|anim| anim.current_offset()).unwrap_or((0.0, 0.0));
            let final_x = x + offset_x * self.screen_width;
            let final_y = y + offset_y * self.screen_height;
            let transform = Transform { position: [final_x, final_y], z_index: state.z_order as f32, ..Transform::IDENTITY };
            let opacity = world
                .get_component::<PortraitAnimationState>(*entity)
                .map(|anim| anim.current_opacity())
                .unwrap_or(state.opacity);
            let tint = Color::new(1.0, 1.0, 1.0, opacity);
            let portrait_width = state.scale * state.texture_width;
            let portrait_height = state.scale * state.texture_height;
            context.draw(DrawCommand::Sprite {
                texture_id: state.texture_id,
                transform,
                size: [portrait_width, portrait_height],
                tint,
                clip_rect: None,
            });
        }
        Ok(())
    }
}

impl System for PortraitRenderSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "portrait_render"
    }

    /// 执行立绘渲染系统逻辑
    ///
    /// 执行流程：
    /// 1. 收集所有带 `PortraitState` 的实体
    /// 2. 按 `z_order` 排序
    /// 3. 使用 `PortraitLayout` 计算位置
    /// 4. 应用高亮效果（说话角色透明度 1.0，非说话角色透明度降低）
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.entities().iter().copied().collect();

        let mut portraits: Vec<(Entity, PortraitState)> = Vec::new();
        for entity in entities {
            if let Some(state) = world.get_component::<PortraitState>(entity) {
                portraits.push((entity, state.clone()));
            }
        }

        portraits.sort_by_key(|(_, state)| state.z_order);

        let layout = PortraitLayout::new(self.screen_width, self.screen_height);

        let has_speaker = portraits.iter().any(|(_, state)| state.is_speaking);

        for (entity, state) in &portraits {
            let _position = layout.calculate_position(&state.position);

            if let Some(comp) = world.get_component_mut::<PortraitState>(*entity) {
                if has_speaker {
                    comp.opacity = if comp.is_speaking { 1.0 } else { DIMMED_OPACITY };
                }
            }
        }

        Ok(())
    }
}

/// 立绘动画系统
///
/// 负责每帧更新立绘动画进度：
/// - 从 World 的 DeltaTime 资源读取真实帧间隔时间
/// - 遍历所有带 `PortraitAnimationState` 的实体
/// - 更新动画进度
/// - 动画完成后移除动画组件
pub struct PortraitAnimationSystem {}

impl PortraitAnimationSystem {
    /// 创建新的立绘动画系统
    pub fn new() -> Self {
        Self {}
    }
}

impl System for PortraitAnimationSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "portrait_animation"
    }

    /// 执行立绘动画系统逻辑
    ///
    /// 执行流程：
    /// 1. 从 World 的 DeltaTime 资源读取真实帧间隔时间
    /// 2. 收集所有带 `PortraitAnimationState` 的实体
    /// 3. 更新每个动画的进度
    /// 4. 收集已完成动画的实体 ID
    /// 5. 移除已完成动画的 `PortraitAnimationState` 组件
    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let delta = world.get_resource::<DeltaTime>().map(|d| d.secs).unwrap_or(1.0 / 60.0);

        let entities: Vec<Entity> = world.entities().iter().copied().collect();

        let mut completed: Vec<Entity> = Vec::new();
        for entity in entities {
            if let Some(anim) = world.get_component_mut::<PortraitAnimationState>(entity) {
                anim.update(delta);
                if anim.is_complete {
                    completed.push(entity);
                }
            }
        }

        for entity in completed {
            world.remove_component::<PortraitAnimationState>(entity);
        }

        Ok(())
    }
}
