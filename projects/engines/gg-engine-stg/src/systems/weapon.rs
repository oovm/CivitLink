//! 武器系统
//! 处理玩家武器射击逻辑

use crate::bullet_pattern::BulletLifetime;
use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 武器系统
/// 处理玩家武器射击逻辑
pub struct WeaponSystem;

impl System for WeaponSystem {
    fn name(&self) -> &str {
        "weapon_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let shoot = world.get_resource::<InputState>().map(|s| s.shoot_pressed).unwrap_or(false);
        if !shoot {
            return Ok(());
        }

        let tick = world.tick();
        let entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for entity in entities {
            let (fire_rate, bullet_speed, power_level, last_fire_frame, pos_x, pos_y) = {
                let weapon = match world.get_component::<Weapon>(entity) {
                    Some(w) => (w.fire_rate, w.bullet_speed, w.last_fire_frame),
                    None => continue,
                };
                let power = world.get_component::<Player>(entity).map(|p| p.power_level).unwrap_or(1);
                let transform = match world.get_component::<Transform>(entity) {
                    Some(t) => (t.x, t.y),
                    None => continue,
                };
                (weapon.0, weapon.1, power, weapon.2, transform.0, transform.1)
            };

            let interval = (60.0 / fire_rate) as u32;
            if tick.saturating_sub(last_fire_frame) < interval {
                continue;
            }

            if let Some(weapon) = world.get_component_mut::<Weapon>(entity) {
                weapon.last_fire_frame = tick;
            }

            let bullet_count = power_level.min(5);
            let spread = (bullet_count as f32 - 1.0) * 5.0;
            for i in 0..bullet_count {
                let angle =
                    if bullet_count > 1 { -spread / 2.0 + (spread / (bullet_count as f32 - 1.0)) * (i as f32) } else { 0.0 };
                let rad = angle.to_radians();
                let dx = rad.sin() * bullet_speed;
                let dy = -rad.cos() * bullet_speed;
                spawn_player_bullet(world, pos_x, pos_y, dx, dy, 1);
            }
        }
        Ok(())
    }
}

/// 生成玩家子弹
fn spawn_player_bullet(world: &mut World, x: f32, y: f32, dx: f32, dy: f32, damage: u32) {
    let bullet_entity = world.spawn().id();
    let _ = world.add_component(bullet_entity, Transform { x, y, rotation: 0.0, scale: 1.0 });
    let _ = world.add_component(bullet_entity, Velocity { dx, dy, ax: 0.0, ay: 0.0 });
    let _ = world.add_component(
        bullet_entity,
        Sprite { path: "player_bullet.png".to_string(), width: 8.0, height: 16.0, visible: true },
    );
    let _ =
        world.add_component(bullet_entity, Collider { collider_type: ColliderType::Circle { radius: 4.0 }, layer: 2, mask: 4 });
    let _ = world
        .add_component(bullet_entity, Bullet { bullet_type: "player".to_string(), damage, shooter_type: ShooterType::Player });
    let _ = world.add_component(bullet_entity, BulletLifetime { remaining_frames: 180, max_frames: 180 });
}
