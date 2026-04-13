//! Boss AI 系统
//! 处理 Boss 阶段切换和移动

use crate::boss::{Boss, BossMovePattern, PhaseTransition};
use crate::bullet_pattern::BulletEmitter;
use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

/// Boss AI 系统
/// 处理 Boss 阶段切换和移动
pub struct BossAISystem;

impl System for BossAISystem {
    fn name(&self) -> &str {
        "boss_ai_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<Boss>().map(|(e, _)| e).collect();
        for entity in entities {
            let (current_phase_index, phases_len, health_current, health_max, phase_elapsed, remaining_invuln) = {
                let boss = match world.get_component::<Boss>(entity) {
                    Some(b) => b,
                    None => continue,
                };
                let health = world.get_component::<Health>(entity).map(|h| (h.current, h.max)).unwrap_or((0, 0));
                (
                    boss.current_phase_index,
                    boss.phases.len(),
                    health.0,
                    health.1,
                    boss.phase_elapsed_frames,
                    boss.remaining_invulnerable_frames,
                )
            };

            if current_phase_index >= phases_len {
                continue;
            }

            let should_transition = {
                let boss = match world.get_component::<Boss>(entity) {
                    Some(b) => b,
                    None => continue,
                };
                let phase = &boss.phases[current_phase_index];
                match &phase.transition {
                    PhaseTransition::HealthBelow(pct) => health_max > 0 && (health_current as f32 / health_max as f32) < *pct,
                    PhaseTransition::TimeElapsed(secs) => phase_elapsed >= (*secs * 60.0) as u32,
                    PhaseTransition::Manual => false,
                }
            };

            if should_transition && current_phase_index + 1 < phases_len {
                if let Some(boss) = world.get_component_mut::<Boss>(entity) {
                    boss.current_phase_index += 1;
                    boss.remaining_invulnerable_frames = boss.invulnerable_frames;
                    boss.phase_elapsed_frames = 0;
                    boss.horizontal_offset = 0.0;
                    boss.circular_angle = 0.0;
                }
                let new_patterns = {
                    let boss = world.get_component::<Boss>(entity);
                    boss.and_then(|b| {
                        let idx = b.current_phase_index;
                        if idx < b.phases.len() { Some(b.phases[idx].bullet_patterns.clone()) } else { None }
                    })
                };
                if let Some(patterns) = new_patterns {
                    if let Some(emitter) = world.get_component_mut::<BulletEmitter>(entity) {
                        emitter.patterns = patterns;
                        emitter.current_pattern_index = 0;
                        emitter.current_pattern_fire_count = 0;
                        emitter.fire_timer = 0;
                    }
                }
            }

            if remaining_invuln > 0 {
                if let Some(boss) = world.get_component_mut::<Boss>(entity) {
                    boss.remaining_invulnerable_frames = boss.remaining_invulnerable_frames.saturating_sub(1);
                }
            }

            if let Some(boss) = world.get_component_mut::<Boss>(entity) {
                boss.phase_elapsed_frames += 1;
            }

            let move_pattern = {
                let boss = match world.get_component::<Boss>(entity) {
                    Some(b) => b,
                    None => continue,
                };
                if boss.current_phase_index < boss.phases.len() {
                    boss.phases[boss.current_phase_index].move_pattern.clone()
                }
                else {
                    BossMovePattern::Stationary
                }
            };

            match move_pattern {
                BossMovePattern::Stationary => {
                    if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                        velocity.dx = 0.0;
                        velocity.dy = 0.0;
                    }
                }
                BossMovePattern::Horizontal { range, speed } => {
                    let (offset, direction) = {
                        let boss = match world.get_component::<Boss>(entity) {
                            Some(b) => (b.horizontal_offset, b.horizontal_direction),
                            None => continue,
                        };
                        (boss.0, boss.1)
                    };
                    let new_offset = offset + direction * speed;
                    let new_direction = if new_offset.abs() > range { -direction } else { direction };
                    if let Some(boss) = world.get_component_mut::<Boss>(entity) {
                        boss.horizontal_offset = new_offset;
                        boss.horizontal_direction = new_direction;
                    }
                    if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                        velocity.dx = direction * speed;
                        velocity.dy = 0.0;
                    }
                }
                BossMovePattern::Circular { radius, angular_speed } => {
                    let angle = {
                        let boss = match world.get_component::<Boss>(entity) {
                            Some(b) => b.circular_angle,
                            None => continue,
                        };
                        boss
                    };
                    let new_angle = angle + angular_speed;
                    if let Some(boss) = world.get_component_mut::<Boss>(entity) {
                        boss.circular_angle = new_angle;
                    }
                    if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                        velocity.dx = new_angle.cos() * radius * angular_speed;
                        velocity.dy = new_angle.sin() * radius * angular_speed;
                    }
                }
                BossMovePattern::Chase { speed } => {
                    let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
                    let mut player_pos = None;
                    for pe in player_entities {
                        if let Some(pt) = world.get_component::<Transform>(pe) {
                            player_pos = Some((pt.x, pt.y));
                            break;
                        }
                    }
                    let (tx, ty) = match world.get_component::<Transform>(entity) {
                        Some(t) => (t.x, t.y),
                        None => continue,
                    };
                    if let Some((px, py)) = player_pos {
                        let dx = px - tx;
                        let dy = py - ty;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 0.0 {
                            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                                velocity.dx = dx / dist * speed;
                                velocity.dy = dy / dist * speed;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
