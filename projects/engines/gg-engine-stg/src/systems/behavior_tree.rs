//! 行为树系统
//! 每帧评估实体的行为树，驱动复杂 AI 决策

use crate::{bullet_pattern::BulletEmitter, components::*};
use gg_core::GResult;
use gg_ecs::{Entity, System, World};

use super::{
    bullet_emitter::{BulletEmitterSystem, spawn_enemy_bullet},
    types::BehaviorStatus,
};

/// 行为树系统
/// 每帧评估实体的行为树，驱动复杂 AI 决策
pub struct BehaviorTreeSystem;

impl BehaviorTreeSystem {
    /// 创建新的行为树系统
    pub fn new() -> Self {
        Self
    }

    /// 评估指定节点的执行结果
    fn evaluate_node(&self, node_index: usize, entity: Entity, world: &mut World, nodes: &[BehaviorNode]) -> BehaviorStatus {
        if node_index >= nodes.len() {
            return BehaviorStatus::Failure;
        }
        let node = &nodes[node_index];
        match node.node_type {
            BehaviorNodeType::Sequence => self.evaluate_sequence(&node.children, entity, world, nodes),
            BehaviorNodeType::Selector => self.evaluate_selector(&node.children, entity, world, nodes),
            BehaviorNodeType::Condition => self.evaluate_condition(node, entity, world),
            BehaviorNodeType::Action => self.evaluate_action(node, entity, world),
        }
    }

    /// 评估序列节点：依次执行子节点，任一失败则整体失败
    fn evaluate_sequence(
        &self,
        children: &[usize],
        entity: Entity,
        world: &mut World,
        nodes: &[BehaviorNode],
    ) -> BehaviorStatus {
        for &child_index in children {
            let status = self.evaluate_node(child_index, entity, world, nodes);
            match status {
                BehaviorStatus::Failure => return BehaviorStatus::Failure,
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Success => {}
            }
        }
        BehaviorStatus::Success
    }

    /// 评估选择节点：依次执行子节点，任一成功则整体成功
    fn evaluate_selector(
        &self,
        children: &[usize],
        entity: Entity,
        world: &mut World,
        nodes: &[BehaviorNode],
    ) -> BehaviorStatus {
        for &child_index in children {
            let status = self.evaluate_node(child_index, entity, world, nodes);
            match status {
                BehaviorStatus::Success => return BehaviorStatus::Success,
                BehaviorStatus::Running => return BehaviorStatus::Running,
                BehaviorStatus::Failure => {}
            }
        }
        BehaviorStatus::Failure
    }

    /// 评估条件节点：根据条件函数名称检查条件是否满足
    fn evaluate_condition(&self, node: &BehaviorNode, entity: Entity, world: &mut World) -> BehaviorStatus {
        let condition_name = match &node.condition {
            Some(name) => name.as_str(),
            None => return BehaviorStatus::Failure,
        };
        let result = match condition_name {
            "is_player_nearby" => self.check_player_nearby(entity, world, 200.0),
            "health_below_50" => self.check_health_below(entity, world, 50),
            "health_below_25" => self.check_health_below(entity, world, 25),
            "is_attacking" => self.check_ai_behavior(entity, world, AIBehavior::Attack),
            "is_chasing" => self.check_ai_behavior(entity, world, AIBehavior::Chase),
            "is_evading" => self.check_ai_behavior(entity, world, AIBehavior::Evade),
            "is_patrolling" => self.check_ai_behavior(entity, world, AIBehavior::Patrol),
            _ => false,
        };
        if result { BehaviorStatus::Success } else { BehaviorStatus::Failure }
    }

    /// 评估动作节点：根据动作函数名称执行对应行为
    fn evaluate_action(&self, node: &BehaviorNode, entity: Entity, world: &mut World) -> BehaviorStatus {
        let action_name = match &node.action {
            Some(name) => name.as_str(),
            None => return BehaviorStatus::Failure,
        };
        match action_name {
            "chase" => {
                self.execute_chase(entity, world);
                BehaviorStatus::Success
            }
            "attack" => {
                self.execute_attack(entity, world);
                BehaviorStatus::Success
            }
            "evade" => {
                self.execute_evade(entity, world);
                BehaviorStatus::Success
            }
            "patrol" => {
                self.execute_patrol(entity, world);
                BehaviorStatus::Success
            }
            "set_chase" => {
                self.set_ai_behavior(entity, world, AIBehavior::Chase);
                BehaviorStatus::Success
            }
            "set_attack" => {
                self.set_ai_behavior(entity, world, AIBehavior::Attack);
                BehaviorStatus::Success
            }
            "set_evade" => {
                self.set_ai_behavior(entity, world, AIBehavior::Evade);
                BehaviorStatus::Success
            }
            "set_patrol" => {
                self.set_ai_behavior(entity, world, AIBehavior::Patrol);
                BehaviorStatus::Success
            }
            _ => BehaviorStatus::Failure,
        }
    }

    /// 检查玩家是否在指定距离内
    fn check_player_nearby(&self, entity: Entity, world: &mut World, distance: f32) -> bool {
        let (ex, ey) = match world.get_component::<Transform>(entity) {
            Some(t) => (t.x, t.y),
            None => return false,
        };
        let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for pe in player_entities {
            if let Some(pt) = world.get_component::<Transform>(pe) {
                let dx = pt.x - ex;
                let dy = pt.y - ey;
                if (dx * dx + dy * dy).sqrt() < distance {
                    return true;
                }
            }
        }
        false
    }

    /// 检查实体生命值是否低于指定百分比
    fn check_health_below(&self, entity: Entity, world: &mut World, percentage: u32) -> bool {
        let health = match world.get_component::<Health>(entity) {
            Some(h) => h,
            None => return false,
        };
        if health.max == 0 {
            return false;
        }
        (health.current * 100 / health.max) < percentage
    }

    /// 检查实体当前 AI 行为是否匹配
    fn check_ai_behavior(&self, entity: Entity, world: &mut World, expected: AIBehavior) -> bool {
        world.get_component::<AI>(entity).map(|ai| ai.behavior == expected).unwrap_or(false)
    }

    /// 执行追踪动作
    fn execute_chase(&self, entity: Entity, world: &mut World) {
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
            let speed = world.get_component::<AI>(entity).map(|a| a.move_speed).unwrap_or(1.0);
            (transform.0, transform.1, speed)
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

    /// 执行攻击动作
    fn execute_attack(&self, entity: Entity, world: &mut World) {
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
                spawn_enemy_bullet(world, pos_x, pos_y, 0.0, 2.0, 1);
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
            spawn_enemy_bullet(world, pos_x, pos_y, 0.0, 2.0, 1);
        }
    }

    /// 执行躲避动作
    fn execute_evade(&self, entity: Entity, world: &mut World) {
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

    /// 执行巡逻动作
    fn execute_patrol(&self, entity: Entity, world: &mut World) {
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

    /// 设置实体的 AI 行为模式
    fn set_ai_behavior(&self, entity: Entity, world: &mut World, behavior: AIBehavior) {
        if let Some(ai) = world.get_component_mut::<AI>(entity) {
            ai.behavior = behavior;
        }
    }
}

impl System for BehaviorTreeSystem {
    fn name(&self) -> &str {
        "behavior_tree_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<BehaviorTree>().map(|(e, _)| e).collect();
        for entity in entities {
            let root_index = {
                let tree = match world.get_component::<BehaviorTree>(entity) {
                    Some(t) => t,
                    None => continue,
                };
                if tree.nodes.is_empty() {
                    continue;
                }
                tree.active_node.unwrap_or(0)
            };
            let nodes = {
                let tree = match world.get_component::<BehaviorTree>(entity) {
                    Some(t) => t,
                    None => continue,
                };
                tree.nodes.clone()
            };
            let status = self.evaluate_node(root_index, entity, world, &nodes);
            if let Some(tree) = world.get_component_mut::<BehaviorTree>(entity) {
                if status == BehaviorStatus::Success || status == BehaviorStatus::Failure {
                    tree.active_node = None;
                }
            }
        }
        Ok(())
    }
}
