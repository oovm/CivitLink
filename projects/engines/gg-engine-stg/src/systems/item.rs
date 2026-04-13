//! 道具系统
//! 处理道具下落、出界销毁和敌人死亡掉落

use crate::components::*;
use crate::item::{Item, ItemSpawner, ItemType};
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 道具系统
/// 处理道具下落、出界销毁和敌人死亡掉落
pub struct ItemSystem;

impl System for ItemSystem {
    fn name(&self) -> &str {
        "item_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (_, screen_h) = world.get_resource::<ScreenBounds>().map(|b| (b.width, b.height)).unwrap_or((800.0, 600.0));

        let entities: Vec<Entity> = world.query::<Item>().map(|(e, _)| e).collect();
        let mut to_despawn: Vec<Entity> = Vec::new();
        for entity in entities {
            let fall_speed = world.get_component::<Item>(entity).map(|i| i.fall_speed).unwrap_or(1.0);
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dy = fall_speed;
                velocity.dx = 0.0;
            }
            let out_of_bounds = world.get_component::<Transform>(entity).map(|t| t.y > screen_h + 50.0).unwrap_or(false);
            if out_of_bounds {
                to_despawn.push(entity);
            }
        }
        for entity in to_despawn {
            let _ = world.despawn(entity);
        }

        let enemy_entities: Vec<Entity> = world.query::<Enemy>().map(|(e, _)| e).collect();
        let mut spawn_items: Vec<(f32, f32, ItemType)> = Vec::new();
        for entity in &enemy_entities {
            let is_dead = world.get_component::<Health>(*entity).map(|h| h.current == 0).unwrap_or(false);
            if !is_dead {
                continue;
            }

            let drop_rate = world.get_resource::<ItemSpawner>().map(|s| s.drop_rate).unwrap_or(0.3);
            let roll = {
                use std::cell::Cell;
                thread_local! {
                    static SEED: Cell<u64> = Cell::new(54321);
                }
                SEED.with(|s| {
                    let mut seed = s.get();
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    s.set(seed);
                    (seed >> 33) as f64 / (1u64 << 31) as f64
                })
            };

            if roll < drop_rate as f64 {
                let pos = world.get_component::<Transform>(*entity).map(|t| (t.x, t.y));
                let item_type = world.get_resource::<ItemSpawner>().map(|s| s.random_item_type()).unwrap_or(ItemType::PowerUp);
                if let Some((x, y)) = pos {
                    spawn_items.push((x, y, item_type));
                }
            }
        }

        for (x, y, item_type) in spawn_items {
            let item_entity = world.spawn().id();
            let _ = world.add_component(item_entity, Transform { x, y, rotation: 0.0, scale: 1.0 });
            let _ = world.add_component(item_entity, Velocity { dx: 0.0, dy: 1.0, ax: 0.0, ay: 0.0 });
            let _ = world.add_component(
                item_entity,
                Sprite { path: format!("{:?}", item_type).to_lowercase(), width: 16.0, height: 16.0, visible: true },
            );
            let _ = world.add_component(
                item_entity,
                Collider { collider_type: ColliderType::Circle { radius: 12.0 }, layer: 8, mask: 1 },
            );
            let _ = world.add_component(item_entity, Item { item_type, value: 1, fall_speed: 1.0 });
        }

        Ok(())
    }
}
