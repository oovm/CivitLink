//! 子弹生命周期系统
//! 超时子弹自动回收

use crate::bullet_pattern::{BulletLifetime, BulletPool};
use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 子弹生命周期系统
/// 超时子弹自动回收
pub struct BulletLifetimeSystem;

impl System for BulletLifetimeSystem {
    fn name(&self) -> &str {
        "bullet_lifetime_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<BulletLifetime>().map(|(e, _)| e).collect();
        let mut to_recycle: Vec<Entity> = Vec::new();
        for entity in entities {
            let should_recycle = world
                .get_component_mut::<BulletLifetime>(entity)
                .map(|lt| {
                    lt.remaining_frames = lt.remaining_frames.saturating_sub(1);
                    lt.remaining_frames == 0
                })
                .unwrap_or(false);
            if should_recycle {
                to_recycle.push(entity);
            }
        }
        for entity in to_recycle {
            let entity_id = entity.index();
            let _ = world.despawn(entity);
            if let Some(pool) = world.get_resource_mut::<BulletPool>() {
                pool.available.push(entity_id);
            }
        }
        Ok(())
    }
}
