//! 出界回收系统
//! 回收超出屏幕边界的子弹和敌人

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 出界回收系统
/// 回收超出屏幕边界的子弹和敌人
pub struct BoundarySystem;

impl System for BoundarySystem {
    fn name(&self) -> &str {
        "boundary_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (screen_w, screen_h) = world.get_resource::<ScreenBounds>().map(|b| (b.width, b.height)).unwrap_or((800.0, 600.0));

        let margin = 100.0;
        let entities = world.entities();
        let mut to_despawn: Vec<Entity> = Vec::new();
        for &entity in &entities {
            let is_bullet = world.get_component::<Bullet>(entity).is_some();
            let is_enemy = world.get_component::<Enemy>(entity).is_some();
            if !is_bullet && !is_enemy {
                continue;
            }

            let out_of_bounds = world
                .get_component::<Transform>(entity)
                .map(|t| t.x < -margin || t.x > screen_w + margin || t.y < -margin || t.y > screen_h + margin)
                .unwrap_or(false);
            if out_of_bounds {
                to_despawn.push(entity);
            }
        }
        for entity in to_despawn {
            let _ = world.despawn(entity);
        }
        Ok(())
    }
}
