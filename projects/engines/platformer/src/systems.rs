//! Platformer 引擎系统定义
//! 定义游戏中使用的各种 ECS 系统

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use gg_render::{Color, DrawCommand, Rect};
use std::collections::HashSet;

/// 输入状态资源
///
/// 存储当前帧的按键状态，由引擎事件循环写入，由输入系统读取。
#[derive(Debug, Default)]
pub struct InputState {
    /// 左方向键是否按下
    pub left_pressed: bool,
    /// 右方向键是否按下
    pub right_pressed: bool,
    /// 跳跃键是否按下
    pub jump_pressed: bool,
}

/// 输入系统
/// 处理玩家输入，从 World 的 InputState 资源读取按键状态
pub struct InputSystem {
    /// 移动速度
    move_speed: f32,
    /// 跳跃力度
    jump_force: f32,
}

impl InputSystem {
    /// 创建新的输入系统
    pub fn new(move_speed: f32, jump_force: f32) -> Self {
        Self {
            move_speed,
            jump_force,
        }
    }
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "input_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (left, right, jump) = world
            .get_resource::<InputState>()
            .map(|s| (s.left_pressed, s.right_pressed, s.jump_pressed))
            .unwrap_or((false, false, false));

        let entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for entity in entities {
            let move_speed = self.move_speed;
            let jump_force = self.jump_force;
            let on_ground = world.get_component::<PhysicsBody>(entity).map(|pb| pb.on_ground).unwrap_or(false);
            let can_jump = world.get_component::<Player>(entity).map(|p| p.can_jump).unwrap_or(false);
            if left {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = -move_speed;
                }
            } else if right {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = move_speed;
                }
            } else {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = 0.0;
                }
            }
            if jump && can_jump && on_ground {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dy = -jump_force;
                }
                if let Some(player) = world.get_component_mut::<Player>(entity) {
                    player.jump_count = 1;
                }
            }
        }
        Ok(())
    }
}

/// 物理系统
/// 处理物理模拟和碰撞响应
pub struct PhysicsSystem {
    /// 重力加速度
    gravity: f32,
    /// 最大下落速度
    max_fall_speed: f32,
    /// 地面摩擦系数
    ground_friction: f32,
    /// 空中摩擦力
    air_friction: f32,
}

impl PhysicsSystem {
    /// 创建新的物理系统
    pub fn new(gravity: f32, max_fall_speed: f32, ground_friction: f32, air_friction: f32) -> Self {
        Self {
            gravity,
            max_fall_speed,
            ground_friction,
            air_friction,
        }
    }
}

impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "physics_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<PhysicsBody>().map(|(e, _)| e).collect();
        for entity in entities {
            let gravity = self.gravity;
            let max_fall_speed = self.max_fall_speed;
            let ground_friction = self.ground_friction;
            let air_friction = self.air_friction;
            let (affected_by_gravity, is_static, on_ground, gravity_scale) = {
                let pb = match world.get_component::<PhysicsBody>(entity) {
                    Some(pb) => pb,
                    None => continue,
                };
                (pb.affected_by_gravity, pb.is_static, pb.on_ground, pb.gravity_scale)
            };
            if affected_by_gravity && !is_static {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dy += gravity * gravity_scale;
                    if velocity.dy > max_fall_speed {
                        velocity.dy = max_fall_speed;
                    }
                }
            }
            if on_ground {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx *= ground_friction;
                }
            } else {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx *= air_friction;
                }
            }
        }
        Ok(())
    }
}

/// 移动系统
/// 处理实体移动
pub struct MovementSystem {
    /// 屏幕宽度
    screen_width: f32,
    /// 屏幕高度
    screen_height: f32,
}

impl MovementSystem {
    /// 创建新的移动系统
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
    }
}

impl System for MovementSystem {
    fn name(&self) -> &str {
        "movement_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<Velocity>().map(|(e, _)| e).collect();
        for entity in entities {
            let is_static = world.get_component::<PhysicsBody>(entity).map(|pb| pb.is_static).unwrap_or(false);
            if is_static {
                continue;
            }
            let (dx, dy) = world.get_component::<Velocity>(entity).map(|v| (v.dx, v.dy)).unwrap_or((0.0, 0.0));
            let screen_width = self.screen_width;
            let screen_height = self.screen_height;
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.x += dx;
                transform.y += dy;
                if transform.x < 0.0 {
                    transform.x = 0.0;
                }
                if transform.x > screen_width {
                    transform.x = screen_width;
                }
                if transform.y < 0.0 {
                    transform.y = 0.0;
                }
                if transform.y > screen_height {
                    transform.y = screen_height;
                }
            }
        }
        Ok(())
    }
}

/// 碰撞系统
/// 处理碰撞检测和响应
pub struct CollisionSystem {
    /// 碰撞层配置
    collision_layers: u32,
}

impl CollisionSystem {
    /// 创建新的碰撞系统
    pub fn new(collision_layers: u32) -> Self {
        Self {
            collision_layers,
        }
    }

    /// 检查两个碰撞体是否碰撞
    fn check_collision(&self, collider1: &Collider, transform1: &Transform, collider2: &Collider, transform2: &Transform) -> bool {
        // 检查碰撞层
        if (collider1.layer & collider2.mask) == 0 || (collider2.layer & collider1.mask) == 0 {
            return false;
        }

        // 根据碰撞体类型检查碰撞
        match (&collider1.collider_type, &collider2.collider_type) {
            (ColliderType::Circle { radius: r1 }, ColliderType::Circle { radius: r2 }) => {
                let dx = transform1.x - transform2.x;
                let dy = transform1.y - transform2.y;
                let distance = (dx * dx + dy * dy).sqrt();
                distance < (r1 + r2)
            }
            (ColliderType::Rectangle { width: w1, height: h1 }, ColliderType::Rectangle { width: w2, height: h2 }) => {
                let left1 = transform1.x - w1 / 2.0;
                let right1 = transform1.x + w1 / 2.0;
                let top1 = transform1.y - h1 / 2.0;
                let bottom1 = transform1.y + h1 / 2.0;

                let left2 = transform2.x - w2 / 2.0;
                let right2 = transform2.x + w2 / 2.0;
                let top2 = transform2.y - h2 / 2.0;
                let bottom2 = transform2.y + h2 / 2.0;

                right1 > left2 && left1 < right2 && bottom1 > top2 && top1 < bottom2
            }
            _ => false,
        }
    }
}

impl System for CollisionSystem {
    fn name(&self) -> &str {
        "collision_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities = world.entities().to_vec();

        let physics_entities: Vec<Entity> = world.query::<PhysicsBody>().map(|(e, _)| e).collect();
        for entity in physics_entities {
            if let Some(physics_body) = world.get_component_mut::<PhysicsBody>(entity) {
                physics_body.on_ground = false;
            }
        }

        let mut grid: std::collections::HashMap<(i32, i32), Vec<Entity>> = std::collections::HashMap::new();
        let cell_size = 64.0;

        for &entity in &entities {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                if world.get_component::<Collider>(entity).is_some() {
                    let cell_x = (transform.x / cell_size) as i32;
                    let cell_y = (transform.y / cell_size) as i32;
                    grid.entry((cell_x, cell_y)).or_default().push(entity);
                }
            }
        }

        for ((cell_x, cell_y), cell_entities) in &grid {
            for i in 0..cell_entities.len() {
                for j in i + 1..cell_entities.len() {
                    let entity1 = cell_entities[i];
                    let entity2 = cell_entities[j];
                    self.check_and_handle_collision(world, entity1, entity2);
                }
            }

            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let neighbor_cell = (cell_x + dx, cell_y + dy);
                    if let Some(neighbor_entities) = grid.get(&neighbor_cell) {
                        for &entity1 in cell_entities {
                            for &entity2 in neighbor_entities {
                                self.check_and_handle_collision(world, entity1, entity2);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl CollisionSystem {
    /// 检查并处理两个实体之间的碰撞
    fn check_and_handle_collision(&self, world: &mut World, entity1: Entity, entity2: Entity) {
        let has_collision = {
            let collider1 = world.get_component::<Collider>(entity1);
            let transform1 = world.get_component::<Transform>(entity1);
            let collider2 = world.get_component::<Collider>(entity2);
            let transform2 = world.get_component::<Transform>(entity2);
            match (collider1, transform1, collider2, transform2) {
                (Some(c1), Some(t1), Some(c2), Some(t2)) => self.check_collision(c1, t1, c2, t2),
                _ => false,
            }
        };
        if has_collision {
            self.handle_collision(world, entity1, entity2);
        }
    }

    /// 处理碰撞响应
    fn handle_collision(&self, world: &mut World, entity1: Entity, entity2: Entity) {
        let is_player_platform = world.get_component::<Player>(entity1).is_some()
            && world.get_component::<Platform>(entity2).is_some();

        if is_player_platform {
            let player_above = {
                let t1 = world.get_component::<Transform>(entity1);
                let t2 = world.get_component::<Transform>(entity2);
                t1.and_then(|t1| t2.map(|t2| t1.y < t2.y))
            };

            let collider2_height = world.get_component::<Collider>(entity2).and_then(|c| {
                if let ColliderType::Rectangle { height, .. } = &c.collider_type {
                    Some(*height)
                } else {
                    None
                }
            });

            let transform2_y = world.get_component::<Transform>(entity2).map(|t| t.y);

            if let Some(true) = player_above {
                if let Some(physics_body) = world.get_component_mut::<PhysicsBody>(entity1) {
                    physics_body.on_ground = true;
                    physics_body.ground_normal = (0.0, 1.0);
                }
                if let (Some(h2), Some(t2y)) = (collider2_height, transform2_y) {
                    if let Some(transform) = world.get_component_mut::<Transform>(entity1) {
                        transform.y = t2y - h2 / 2.0 - 16.0;
                    }
                }
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity1) {
                    velocity.dy = 0.0;
                }
            }
        }

        let is_player_collectible = world.get_component::<Player>(entity1).is_some()
            && world.get_component::<Collectible>(entity2).is_some();

        if is_player_collectible {
            if let Some(collectible) = world.get_component_mut::<Collectible>(entity2) {
                if !collectible.collected {
                    collectible.collected = true;
                }
            }
        }
    }
}

/// 收集系统
/// 处理物品收集逻辑
pub struct CollectibleSystem {
    /// 分数
    score: u32,
    /// 已计分的实体集合
    scored_entities: HashSet<Entity>,
}

impl CollectibleSystem {
    /// 创建新的收集系统
    pub fn new() -> Self {
        Self {
            score: 0,
            scored_entities: HashSet::new(),
        }
    }

    /// 获取当前分数
    pub fn score(&self) -> u32 {
        self.score
    }
}

impl System for CollectibleSystem {
    fn name(&self) -> &str {
        "collectible_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<Collectible>().map(|(e, _)| e).collect();
        for entity in entities {
            let collected = world.get_component::<Collectible>(entity).map(|c| c.collected).unwrap_or(false);
            if collected && !self.scored_entities.contains(&entity) {
                let value = world.get_component::<Collectible>(entity).map(|c| c.value).unwrap_or(0);
                self.score += value;
                self.scored_entities.insert(entity);
                if let Some(sprite) = world.get_component_mut::<Sprite>(entity) {
                    sprite.visible = false;
                }
            }
        }
        Ok(())
    }
}

/// AI 系统
/// 处理敌人 AI 行为
pub struct AISystem {
    /// 屏幕宽度
    screen_width: f32,
    /// 屏幕高度
    screen_height: f32,
}

impl AISystem {
    /// 创建新的 AI 系统
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
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
        } else {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = dx / distance * move_speed;
                velocity.dy = dy / distance * move_speed;
            }
        }
    }

    /// 处理追踪行为
    fn handle_chase(&self, entity: Entity, world: &mut World) {
        let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        let mut player_pos = None;
        for player_entity in player_entities {
            if let Some(player_transform) = world.get_component::<Transform>(player_entity) {
                player_pos = Some((player_transform.x, player_transform.y));
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

        if let Some((player_x, player_y)) = player_pos {
            let dx = player_x - transform_x;
            let dy = player_y - transform_y;
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
    fn handle_attack(&self, entity: Entity, world: &mut World) {
        let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        let mut nearest_player_pos = None;
        let mut nearest_distance = f32::MAX;
        for player_entity in player_entities {
            if let Some(player_transform) = world.get_component::<Transform>(player_entity) {
                let ai_transform = match world.get_component::<Transform>(entity) {
                    Some(t) => t,
                    None => return,
                };
                let dx = player_transform.x - ai_transform.x;
                let dy = player_transform.y - ai_transform.y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < nearest_distance {
                    nearest_distance = distance;
                    nearest_player_pos = Some((player_transform.x, player_transform.y));
                }
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

        if let Some((player_x, player_y)) = nearest_player_pos {
            let dx = player_x - transform_x;
            let dy = player_y - transform_y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 200.0 && distance > 0.0 {
                let dash_speed = move_speed * 2.5;
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = dx / distance * dash_speed;
                    velocity.dy = dy / distance * dash_speed;
                }
            }
        }
    }

    /// 处理躲避行为
    fn handle_evade(&self, _entity: Entity, _world: &mut World) {
    }
}

/// 渲染系统
/// 处理渲染逻辑
pub struct RenderSystem {
    /// 屏幕宽度
    screen_width: f32,
    /// 屏幕高度
    screen_height: f32,
}

impl RenderSystem {
    /// 创建新的渲染系统
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
    }
}

impl System for RenderSystem {
    fn name(&self) -> &str {
        "render_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
            buffer.clear();
        } else {
            world.insert_resource(DrawCommandBuffer::new());
        }

        let entities: Vec<Entity> = world.query::<Sprite>()
            .filter(|(_, sprite)| sprite.visible)
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            let (x, y, width, height) = {
                let sprite = match world.get_component::<Sprite>(entity) {
                    Some(s) => s,
                    None => continue,
                };
                if !sprite.visible {
                    continue;
                }
                let (w, h) = (sprite.width, sprite.height);
                if let Some(transform) = world.get_component::<Transform>(entity) {
                    (transform.x, transform.y, w, h)
                } else {
                    (0.0, 0.0, w, h)
                }
            };

            let command = DrawCommand::Rect {
                rect: Rect::new(x, y, width, height),
                color: Color::WHITE,
                corner_radius: 0.0,
            };

            if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
                buffer.push(command);
            }
        }

        Ok(())
    }
}
