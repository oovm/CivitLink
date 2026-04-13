//! 弹幕发射器系统
//! 根据弹幕模式生成敌人子弹

use crate::bullet_pattern::{BulletEmitter, BulletLifetime, BulletPattern, PatternType};
use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// 弹幕发射器系统
/// 根据弹幕模式生成敌人子弹
pub struct BulletEmitterSystem;

impl BulletEmitterSystem {
    /// 创建新的弹幕发射器系统
    pub fn new() -> Self {
        Self
    }

    /// 根据弹幕模式生成子弹
    pub fn emit_pattern(world: &mut World, emitter_entity: Entity, pattern: &BulletPattern, spiral_angle: &mut f32) {
        let (pos_x, pos_y) = match world.get_component::<Transform>(emitter_entity) {
            Some(t) => (t.x, t.y),
            None => return,
        };

        let player_pos = {
            let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
            let mut pos = (pos_x, pos_y + 1.0);
            for pe in player_entities {
                if let Some(pt) = world.get_component::<Transform>(pe) {
                    pos = (pt.x, pt.y);
                    break;
                }
            }
            pos
        };

        match &pattern.pattern_type {
            PatternType::Single => {
                let angle = pattern.angle_offset;
                let dx = angle.sin() * pattern.speed;
                let dy = angle.cos() * pattern.speed;
                for _ in 0..pattern.count {
                    spawn_enemy_bullet(world, pos_x, pos_y, dx, dy, 1);
                }
            }
            PatternType::Spread { spread_angle } => {
                let count = pattern.count.max(1);
                let base_angle = pattern.angle_offset;
                for i in 0..count {
                    let t = if count > 1 { i as f32 / (count - 1) as f32 } else { 0.5 };
                    let angle = base_angle - spread_angle / 2.0 + spread_angle * t;
                    let dx = angle.sin() * pattern.speed;
                    let dy = angle.cos() * pattern.speed;
                    spawn_enemy_bullet(world, pos_x, pos_y, dx, dy, 1);
                }
            }
            PatternType::Ring => {
                let count = pattern.count.max(1);
                for i in 0..count {
                    let angle = pattern.angle_offset + (2.0 * std::f32::consts::PI * i as f32) / count as f32;
                    let dx = angle.sin() * pattern.speed;
                    let dy = angle.cos() * pattern.speed;
                    spawn_enemy_bullet(world, pos_x, pos_y, dx, dy, 1);
                }
            }
            PatternType::Spiral { angular_speed } => {
                let angle = *spiral_angle + pattern.angle_offset;
                let dx = angle.sin() * pattern.speed;
                let dy = angle.cos() * pattern.speed;
                spawn_enemy_bullet(world, pos_x, pos_y, dx, dy, 1);
                *spiral_angle += angular_speed;
            }
            PatternType::Aimed => {
                let dx = player_pos.0 - pos_x;
                let dy = player_pos.1 - pos_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > 0.0 {
                    let vx = dx / dist * pattern.speed;
                    let vy = dy / dist * pattern.speed;
                    for _ in 0..pattern.count {
                        spawn_enemy_bullet(world, pos_x, pos_y, vx, vy, 1);
                    }
                }
            }
            PatternType::Custom { angles } => {
                for &angle in angles {
                    let a = angle + pattern.angle_offset;
                    let dx = a.sin() * pattern.speed;
                    let dy = a.cos() * pattern.speed;
                    spawn_enemy_bullet(world, pos_x, pos_y, dx, dy, 1);
                }
            }
        }
    }
}

/// 生成敌人子弹
pub fn spawn_enemy_bullet(world: &mut World, x: f32, y: f32, dx: f32, dy: f32, damage: u32) {
    let bullet_entity = world.spawn().id();
    let _ = world.add_component(bullet_entity, Transform { x, y, rotation: 0.0, scale: 1.0 });
    let _ = world.add_component(bullet_entity, Velocity { dx, dy, ax: 0.0, ay: 0.0 });
    let _ = world
        .add_component(bullet_entity, Sprite { path: "enemy_bullet.png".to_string(), width: 8.0, height: 8.0, visible: true });
    let _ =
        world.add_component(bullet_entity, Collider { collider_type: ColliderType::Circle { radius: 4.0 }, layer: 4, mask: 2 });
    let _ = world
        .add_component(bullet_entity, Bullet { bullet_type: "enemy".to_string(), damage, shooter_type: ShooterType::Enemy });
    let _ = world.add_component(bullet_entity, BulletLifetime { remaining_frames: 600, max_frames: 600 });
}

impl System for BulletEmitterSystem {
    fn name(&self) -> &str {
        "bullet_emitter_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<BulletEmitter>().map(|(e, _)| e).collect();
        for entity in entities {
            let (active, fire_timer, current_index, _fire_count, fire_limit, spiral_angle, patterns_len) = {
                let emitter = match world.get_component::<BulletEmitter>(entity) {
                    Some(e) => e,
                    None => continue,
                };
                (
                    emitter.active,
                    emitter.fire_timer,
                    emitter.current_pattern_index,
                    emitter.current_pattern_fire_count,
                    emitter.pattern_fire_limit,
                    emitter.spiral_angle,
                    emitter.patterns.len(),
                )
            };

            if !active || patterns_len == 0 {
                continue;
            }

            let new_timer = fire_timer + 1;
            if let Some(emitter) = world.get_component_mut::<BulletEmitter>(entity) {
                emitter.fire_timer = new_timer;
            }

            let interval = {
                let emitter = match world.get_component::<BulletEmitter>(entity) {
                    Some(e) => e,
                    None => continue,
                };
                if current_index >= emitter.patterns.len() {
                    continue;
                }
                emitter.patterns[current_index].interval
            };

            if new_timer >= interval {
                let pattern = {
                    let emitter = match world.get_component::<BulletEmitter>(entity) {
                        Some(e) => e,
                        None => continue,
                    };
                    if current_index < emitter.patterns.len() { Some(emitter.patterns[current_index].clone()) } else { None }
                };

                if let Some(pattern) = pattern {
                    let mut spiral = spiral_angle;
                    Self::emit_pattern(world, entity, &pattern, &mut spiral);
                    if let Some(emitter) = world.get_component_mut::<BulletEmitter>(entity) {
                        emitter.fire_timer = 0;
                        emitter.current_pattern_fire_count += 1;
                        emitter.spiral_angle = spiral;
                        if fire_limit > 0 && emitter.current_pattern_fire_count >= fire_limit {
                            emitter.current_pattern_index = (current_index + 1) % patterns_len;
                            emitter.current_pattern_fire_count = 0;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
