//! AI 系统
//! 处理敌人 AI 行为

use crate::bullet_pattern::BulletEmitter;
use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

use super::bullet_emitter::BulletEmitterSystem;

/// AI 系统
/// 处理敌人 AI 行为
pub struct AISystem;

impl AISystem {
    /// 创建新的 AI 系统
    pub fn new() -> Self {
        Self
    }
}

impl System for AISystem {
    fn name(&self) -> &str {
        "ai_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<AI>().map(|(e, _)| e).collect();
        for entity in entities {
            let behavior = world.get_component::<AI>(entity).map(|ai| ai.behavior);
            match behavior {
                Some(AIBehavior::Patrol) => self.handle_patrol(entity, world),
                Some(AIBehavior::Chase) => self.handle_chase(entity, world),
                Some(AIBehavior::Attack) => self.handle_attack(entity, world),
                Some(AIBehavior::Evade) => self.handle_evade(entity, world),
                None => {}
            }
        }
        Ok(())
    }
}

impl AISystem {
    /// 处理巡逻行为
    fn handle_patrol(&self, entity: Entity, world: &mut World) {
        let (transform_x, transform_y, patrol_path, current_index, move_speed) = {
            let transform = match world.get_component::<Transform>(entity) {
                Some(t) => (t.x, t.y),
                None => return,
            };
            let ai = match world.get_component::<AI>(entity) {
                Some(a) => (a.patrol_path.clone(), a.current_path_index, a.move_speed),
                None => return,
            };
            (transform.0, transform.1, ai.0, ai.1, ai.2)
        };

        if patrol_path.is_empty() {
            return;
        }
        let target = patrol_path[current_index];
        let dx = target.0 - transform_x;
        let dy = target.1 - transform_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 5.0 {
            if let Some(ai) = world.get_component_mut::<AI>(entity) {
                ai.current_path_index = (current_index + 1) % patrol_path.len();
            }
        }
        else if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
            velocity.dx = dx / distance * move_speed;
            velocity.dy = dy / distance * move_speed;
        }
    }

    /// 处理追踪行为
    fn handle_chase(&self, entity: Entity, world: &mut World) {
        let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        let mut player_pos = None;
        for pe in player_entities {
            if let Some(pt) = world.get_component::<Transform>(pe) {
                player_pos = Some((pt.x, pt.y));
                break;
            }
        }

        let (transform_x, transform_y, move_speed) = {
            let transform = match world.get_component::<Transform>(entity) {
                Some(t) => (t.x, t.y),
                None => return,
            };
            let ai = match world.get_component::<AI>(entity) {
                Some(a) => a.move_speed,
                None => return,
            };
            (transform.0, transform.1, ai)
        };

        if let Some((px, py)) = player_pos {
            let dx = px - transform_x;
            let dy = py - transform_y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 5.0 {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = dx / distance * move_speed;
                    velocity.dy = dy / distance * move_speed;
                }
            }
        }
    }

    /// 处理攻击行为
    /// 如果实体拥有 BulletEmitter 组件，则触发其弹幕发射；
    /// 否则回退为简单向下单发子弹
    fn handle_attack(&self, entity: Entity, world: &mut World) {
        let has_emitter = world.get_component::<BulletEmitter>(entity).is_some();
        if has_emitter {
            let (current_index, fire_timer, patterns_len, fire_limit, spiral_angle) = {
                let emitter = match world.get_component::<BulletEmitter>(entity) {
                    Some(e) => e,
                    None => return,
                };
                (
                    emitter.current_pattern_index,
                    emitter.fire_timer,
                    emitter.patterns.len(),
                    emitter.pattern_fire_limit,
                    emitter.spiral_angle,
                )
            };

            if patterns_len == 0 {
                let (pos_x, pos_y) = match world.get_component::<Transform>(entity) {
                    Some(t) => (t.x, t.y),
                    None => return,
                };
                super::bullet_emitter::spawn_enemy_bullet(world, pos_x, pos_y, 0.0, 2.0, 1);
                return;
            }

            let new_timer = fire_timer + 1;
            if let Some(emitter) = world.get_component_mut::<BulletEmitter>(entity) {
                emitter.fire_timer = new_timer;
            }

            let interval = {
                let emitter = match world.get_component::<BulletEmitter>(entity) {
                    Some(e) => e,
                    None => return,
                };
                if current_index >= emitter.patterns.len() {
                    return;
                }
                emitter.patterns[current_index].interval
            };

            if new_timer >= interval {
                let pattern = {
                    let emitter = match world.get_component::<BulletEmitter>(entity) {
                        Some(e) => e,
                        None => return,
                    };
                    if current_index < emitter.patterns.len() { Some(emitter.patterns[current_index].clone()) } else { None }
                };

                if let Some(pattern) = pattern {
                    let mut spiral = spiral_angle;
                    BulletEmitterSystem::emit_pattern(world, entity, &pattern, &mut spiral);
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
        else {
            let (pos_x, pos_y) = match world.get_component::<Transform>(entity) {
                Some(t) => (t.x, t.y),
                None => return,
            };
            super::bullet_emitter::spawn_enemy_bullet(world, pos_x, pos_y, 0.0, 2.0, 1);
        }
    }

    /// 处理躲避行为
    fn handle_evade(&self, entity: Entity, world: &mut World) {
        let bullet_entities: Vec<Entity> = world.query::<Bullet>().map(|(e, _)| e).collect();
        let mut evade_dx = 0.0f32;
        let mut evade_dy = 0.0f32;
        for be in bullet_entities {
            let is_player_bullet =
                world.get_component::<Bullet>(be).map(|b| matches!(b.shooter_type, ShooterType::Player)).unwrap_or(false);
            if !is_player_bullet {
                continue;
            }
            let (bx, by) = match world.get_component::<Transform>(be) {
                Some(t) => (t.x, t.y),
                None => continue,
            };
            let (tx, ty, speed) = {
                let t = match world.get_component::<Transform>(entity) {
                    Some(t) => (t.x, t.y),
                    None => return,
                };
                let s = world.get_component::<AI>(entity).map(|a| a.move_speed).unwrap_or(1.0);
                (t.0, t.1, s)
            };
            let dx = bx - tx;
            let dy = by - ty;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 80.0 && dist > 0.0 {
                evade_dx -= dx / dist * speed;
                evade_dy -= dy / dist * speed;
            }
        }
        if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
            velocity.dx = evade_dx;
            velocity.dy = evade_dy;
        }
    }
}
