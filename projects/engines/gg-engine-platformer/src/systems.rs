//! Platformer 引擎系统定义
//! 定义游戏中使用的各种 ECS 系统

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use gg_render::{Color, DrawCommand, Rect};
use std::collections::HashSet;

/// 输入系统
/// 处理玩家输入，从 World 的 InputState 资源读取动作状态，支持键盘和手柄
pub struct InputSystem {
    /// 移动速度
    move_speed: f32,
    /// 跳跃力度
    jump_force: f32,
}

impl InputSystem {
    /// 创建新的输入系统
    pub fn new(move_speed: f32, jump_force: f32) -> Self {
        Self { move_speed, jump_force }
    }
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "input_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (move_left, move_right, jump) = world
            .get_resource::<InputState>()
            .map(|s| {
                (
                    s.is_action_pressed(InputAction::MoveLeft),
                    s.is_action_pressed(InputAction::MoveRight),
                    s.is_action_pressed(InputAction::Jump),
                )
            })
            .unwrap_or((false, false, false));

        let (gamepad_jump, gamepad_stick_x, gamepad_connected) = world
            .get_resource::<InputState>()
            .map(|s| (s.is_action_pressed(InputAction::GamepadJump), s.gamepad_left_stick_x, s.gamepad_connected))
            .unwrap_or((false, 0.0, false));

        let entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for entity in entities {
            let move_speed = self.move_speed;
            let jump_force = self.jump_force;

            let (on_ground, jump_count, max_jump_count, invulnerable) = {
                let pb = world.get_component::<PhysicsBody>(entity);
                let player = world.get_component::<Player>(entity);
                match (pb, player) {
                    (Some(pb), Some(p)) => (pb.on_ground, p.jump_count, p.max_jump_count, p.invulnerable),
                    _ => continue,
                }
            };

            if gamepad_connected && gamepad_stick_x.abs() > 0.2 {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = gamepad_stick_x * move_speed;
                }
            }
            else if move_left {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = -move_speed;
                }
            }
            else if move_right {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = move_speed;
                }
            }
            else {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dx = 0.0;
                }
            }

            if jump || (gamepad_connected && gamepad_jump) {
                let can_jump = on_ground || jump_count < max_jump_count;
                if can_jump {
                    if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                        velocity.dy = -jump_force;
                    }
                    if let Some(player) = world.get_component_mut::<Player>(entity) {
                        player.jump_count += 1;
                        player.can_jump = false;
                    }
                }
            }

            if invulnerable {
                let _ = jump_count;
            }
        }
        Ok(())
    }
}

/// 物理系统
/// 处理物理模拟和碰撞响应，基于 DeltaTime 驱动，支持连续碰撞检测和物理材质
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
        Self { gravity, max_fall_speed, ground_friction, air_friction }
    }
}

impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "physics_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let dt = world.get_resource::<DeltaTime>().map(|d| d.seconds).unwrap_or(1.0 / 60.0);

        let entities: Vec<Entity> = world.query::<PhysicsBody>().map(|(e, _)| e).collect();
        for entity in entities {
            let gravity = self.gravity;
            let max_fall_speed = self.max_fall_speed;
            let ground_friction = self.ground_friction;
            let air_friction = self.air_friction;

            let (affected_by_gravity, is_static, on_ground, gravity_scale, ccd_enabled) = {
                let pb = match world.get_component::<PhysicsBody>(entity) {
                    Some(pb) => pb,
                    None => continue,
                };
                (pb.affected_by_gravity, pb.is_static, pb.on_ground, pb.gravity_scale, pb.ccd_enabled)
            };

            if affected_by_gravity && !is_static {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    velocity.dy += gravity * gravity_scale * dt * 60.0;
                    if velocity.dy > max_fall_speed {
                        velocity.dy = max_fall_speed;
                    }
                }
            }

            let physics_material_friction = world.get_component::<PhysicsMaterial>(entity).map(|m| m.friction);

            if on_ground {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    let effective_friction = physics_material_friction.unwrap_or(ground_friction);
                    let factor = (1.0 - effective_friction).powf(dt * 60.0);
                    velocity.dx *= factor;
                }
            }
            else {
                if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                    let factor = (1.0 - air_friction).powf(dt * 60.0);
                    velocity.dx *= factor;
                }
            }

            if !is_static {
                let (dx, dy) = world.get_component::<Velocity>(entity).map(|v| (v.dx, v.dy)).unwrap_or((0.0, 0.0));

                if ccd_enabled {
                    let speed = (dx * dx + dy * dy).sqrt();
                    let ccd_threshold = 8.0;
                    if speed > ccd_threshold {
                        let steps = (speed / ccd_threshold).ceil() as usize;
                        let step_dt = dt / steps as f32;
                        for _ in 0..steps {
                            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                                transform.x += dx * step_dt * 60.0;
                                transform.y += dy * step_dt * 60.0;
                            }
                        }
                    }
                    else if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                        transform.x += dx * dt * 60.0;
                        transform.y += dy * dt * 60.0;
                    }
                }
                else if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                    transform.x += dx * dt * 60.0;
                    transform.y += dy * dt * 60.0;
                }
            }

            if let Some(player) = world.get_component_mut::<Player>(entity) {
                if player.invulnerable {
                    player.invulnerable_timer -= dt;
                    if player.invulnerable_timer <= 0.0 {
                        player.invulnerable = false;
                        player.invulnerable_timer = 0.0;
                    }
                }
            }
        }
        Ok(())
    }
}

/// 碰撞系统
/// 处理碰撞检测和响应，使用空间网格优化并返回 CollisionInfo
pub struct CollisionSystem {
    #[allow(dead_code)]
    collision_layers: u32,
}

impl CollisionSystem {
    /// 创建新的碰撞系统
    pub fn new(collision_layers: u32) -> Self {
        Self { collision_layers }
    }

    /// 检查两个碰撞体是否碰撞，并返回碰撞信息
    fn check_collision(
        &self,
        collider1: &Collider,
        transform1: &Transform,
        collider2: &Collider,
        transform2: &Transform,
    ) -> Option<CollisionInfo> {
        if (collider1.layer & collider2.mask) == 0 || (collider2.layer & collider1.mask) == 0 {
            return None;
        }

        match (&collider1.collider_type, &collider2.collider_type) {
            (ColliderType::Circle { radius: r1 }, ColliderType::Circle { radius: r2 }) => {
                let dx = transform2.x - transform1.x;
                let dy = transform2.y - transform1.y;
                let distance = (dx * dx + dy * dy).sqrt();
                let sum_r = r1 + r2;

                if distance < sum_r && distance > 0.0 {
                    let nx = dx / distance;
                    let ny = dy / distance;
                    Some(CollisionInfo {
                        penetration_depth: sum_r - distance,
                        normal: (nx, ny),
                        point: (transform1.x + nx * r1, transform1.y + ny * r1),
                    })
                }
                else if distance == 0.0 {
                    Some(CollisionInfo { penetration_depth: sum_r, normal: (0.0, -1.0), point: (transform1.x, transform1.y) })
                }
                else {
                    None
                }
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

                if right1 > left2 && left1 < right2 && bottom1 > top2 && top1 < bottom2 {
                    let overlap_x = (right1 - left2).min(right2 - left1);
                    let overlap_y = (bottom1 - top2).min(bottom2 - top1);

                    let (penetration, normal) = if overlap_x < overlap_y {
                        let nx = if transform1.x < transform2.x { -1.0 } else { 1.0 };
                        (overlap_x, (nx, 0.0))
                    }
                    else {
                        let ny = if transform1.y < transform2.y { -1.0 } else { 1.0 };
                        (overlap_y, (0.0, ny))
                    };

                    Some(CollisionInfo {
                        penetration_depth: penetration,
                        normal,
                        point: ((transform1.x + transform2.x) / 2.0, (transform1.y + transform2.y) / 2.0),
                    })
                }
                else {
                    None
                }
            }
            (ColliderType::Circle { radius }, ColliderType::Rectangle { width, height }) => {
                Self::circle_rect_collision(transform1, *radius, transform2, *width, *height)
            }
            (ColliderType::Rectangle { width, height }, ColliderType::Circle { radius }) => {
                Self::circle_rect_collision(transform2, *radius, transform1, *width, *height).map(|mut info| {
                    info.normal = (-info.normal.0, -info.normal.1);
                    info
                })
            }
        }
    }

    /// 计算圆形与矩形碰撞体的碰撞信息
    fn circle_rect_collision(
        circle_transform: &Transform,
        radius: f32,
        rect_transform: &Transform,
        rect_width: f32,
        rect_height: f32,
    ) -> Option<CollisionInfo> {
        let half_w = rect_width / 2.0;
        let half_h = rect_height / 2.0;

        let closest_x = circle_transform.x.clamp(rect_transform.x - half_w, rect_transform.x + half_w);
        let closest_y = circle_transform.y.clamp(rect_transform.y - half_h, rect_transform.y + half_h);

        let dx = circle_transform.x - closest_x;
        let dy = circle_transform.y - closest_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < radius {
            let (nx, ny) = if distance > 0.0 { (dx / distance, dy / distance) } else { (0.0, -1.0) };
            Some(CollisionInfo { penetration_depth: radius - distance, normal: (nx, ny), point: (closest_x, closest_y) })
        }
        else {
            None
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
        let mut entity_count = 0usize;
        let cell_size = 64.0f32;

        for &entity in &entities {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                if world.get_component::<Collider>(entity).is_some() {
                    entity_count += 1;
                    let cell_x = (transform.x / cell_size) as i32;
                    let cell_y = (transform.y / cell_size) as i32;
                    grid.entry((cell_x, cell_y)).or_default().push(entity);
                }
            }
        }

        let dynamic_cell_size = if entity_count > 200 {
            128.0
        }
        else if entity_count < 50 {
            32.0
        }
        else {
            cell_size
        };
        let _ = dynamic_cell_size;

        let mut checked_pairs: HashSet<(Entity, Entity)> = HashSet::new();

        for ((cell_x, cell_y), cell_entities) in &grid {
            for i in 0..cell_entities.len() {
                for j in i + 1..cell_entities.len() {
                    let entity1 = cell_entities[i];
                    let entity2 = cell_entities[j];
                    let pair = if entity1 < entity2 { (entity1, entity2) } else { (entity2, entity1) };
                    if checked_pairs.insert(pair) {
                        self.check_and_handle_collision(world, entity1, entity2);
                    }
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
                                let pair = if entity1 < entity2 { (entity1, entity2) } else { (entity2, entity1) };
                                if checked_pairs.insert(pair) {
                                    self.check_and_handle_collision(world, entity1, entity2);
                                }
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
        let collision_info = {
            let collider1 = world.get_component::<Collider>(entity1);
            let transform1 = world.get_component::<Transform>(entity1);
            let collider2 = world.get_component::<Collider>(entity2);
            let transform2 = world.get_component::<Transform>(entity2);
            match (collider1, transform1, collider2, transform2) {
                (Some(c1), Some(t1), Some(c2), Some(t2)) => self.check_collision(c1, t1, c2, t2),
                _ => None,
            }
        };

        if let Some(info) = collision_info {
            self.handle_collision(world, entity1, entity2, &info);
        }
    }

    /// 处理碰撞响应
    fn handle_collision(&self, world: &mut World, entity1: Entity, entity2: Entity, info: &CollisionInfo) {
        let is_player1 = world.get_component::<Player>(entity1).is_some();
        let is_player2 = world.get_component::<Player>(entity2).is_some();
        let is_platform1 = world.get_component::<Platform>(entity1).is_some();
        let is_platform2 = world.get_component::<Platform>(entity2).is_some();
        let is_enemy1 = world.get_component::<Enemy>(entity1).is_some();
        let is_enemy2 = world.get_component::<Enemy>(entity2).is_some();
        let is_collectible1 = world.get_component::<Collectible>(entity1).is_some();
        let is_collectible2 = world.get_component::<Collectible>(entity2).is_some();

        if is_player1 && is_platform2 {
            self.handle_player_platform_collision(world, entity1, entity2, info);
        }
        else if is_player2 && is_platform1 {
            let reversed_info = CollisionInfo {
                penetration_depth: info.penetration_depth,
                normal: (-info.normal.0, -info.normal.1),
                point: info.point,
            };
            self.handle_player_platform_collision(world, entity2, entity1, &reversed_info);
        }

        if is_player1 && is_enemy2 {
            self.handle_player_enemy_collision(world, entity1, entity2, info);
        }
        else if is_player2 && is_enemy1 {
            let reversed_info = CollisionInfo {
                penetration_depth: info.penetration_depth,
                normal: (-info.normal.0, -info.normal.1),
                point: info.point,
            };
            self.handle_player_enemy_collision(world, entity2, entity1, &reversed_info);
        }

        if is_player1 && is_collectible2 {
            self.handle_player_collectible_collision(world, entity2);
        }
        else if is_player2 && is_collectible1 {
            self.handle_player_collectible_collision(world, entity1);
        }
    }

    /// 处理玩家与平台的碰撞
    fn handle_player_platform_collision(
        &self,
        world: &mut World,
        player_entity: Entity,
        platform_entity: Entity,
        info: &CollisionInfo,
    ) {
        let dt = world.get_resource::<DeltaTime>().map(|d| d.seconds).unwrap_or(1.0 / 60.0);

        let is_one_way = world.get_component::<Platform>(platform_entity).map(|p| p.is_one_way).unwrap_or(false);

        let is_active = world.get_component::<Platform>(platform_entity).map(|p| p.is_active).unwrap_or(true);

        if !is_active {
            return;
        }

        if is_one_way {
            let player_dy = world.get_component::<Velocity>(player_entity).map(|v| v.dy).unwrap_or(0.0);
            if player_dy < 0.0 {
                return;
            }
            if info.normal.1 >= 0.0 {
                return;
            }
        }

        let is_on_top = info.normal.1 < -0.5;

        if is_on_top {
            if let Some(physics_body) = world.get_component_mut::<PhysicsBody>(player_entity) {
                physics_body.on_ground = true;
                physics_body.ground_normal = (info.normal.0, info.normal.1);
            }

            if let Some(player) = world.get_component_mut::<Player>(player_entity) {
                player.jump_count = 0;
                player.can_jump = true;
            }

            if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                if velocity.dy > 0.0 {
                    velocity.dy = 0.0;
                }
            }

            if let Some(transform) = world.get_component_mut::<Transform>(player_entity) {
                transform.y -= info.penetration_depth * info.normal.1.abs();
            }

            let platform_move_dx = world.get_component::<Platform>(platform_entity).and_then(|p| {
                if matches!(p.platform_type, PlatformType::Moving) { Some(p.move_speed * p.move_direction) } else { None }
            });

            if let Some(move_dx) = platform_move_dx {
                if let Some(transform) = world.get_component_mut::<Transform>(player_entity) {
                    transform.x += move_dx * dt * 60.0;
                }
            }

            let platform_type = world.get_component::<Platform>(platform_entity).map(|p| p.platform_type);

            if let Some(PlatformType::Breakable) = platform_type {
                if let Some(platform) = world.get_component_mut::<Platform>(platform_entity) {
                    if platform.break_timer <= 0.0 {
                        platform.break_timer = platform.break_delay;
                    }
                }
            }

            if let Some(PlatformType::Disappearing) = platform_type {
                if let Some(platform) = world.get_component_mut::<Platform>(platform_entity) {
                    if platform.fade_timer <= 0.0 && platform.respawn_timer <= 0.0 {
                        platform.fade_timer = platform.fade_time;
                    }
                }
            }

            if let Some(PlatformType::Spring) = platform_type {
                let spring_force = world.get_component::<Platform>(platform_entity).map(|p| p.spring_force).unwrap_or(15.0);
                if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                    velocity.dy = -spring_force;
                }
                if let Some(player) = world.get_component_mut::<Player>(player_entity) {
                    player.jump_count = 0;
                    player.can_jump = true;
                }
                if let Some(platform) = world.get_component_mut::<Platform>(platform_entity) {
                    platform.spring_compressed = true;
                }
            }

            if let Some(PlatformType::Conveyor) = platform_type {
                let (conveyor_direction, conveyor_speed) = world
                    .get_component::<Platform>(platform_entity)
                    .map(|p| (p.conveyor_direction, p.conveyor_speed))
                    .unwrap_or((1.0, 3.0));
                if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                    velocity.dx += conveyor_direction * conveyor_speed * dt;
                }
            }

            if let Some(PlatformType::Ice) = platform_type {
                let ice_friction = world.get_component::<PhysicsMaterial>(platform_entity).map(|m| m.friction).unwrap_or(0.05);
                if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                    let factor = (1.0 - ice_friction).powf(dt * 60.0);
                    velocity.dx *= factor;
                }
            }
        }
        else {
            if let Some(transform) = world.get_component_mut::<Transform>(player_entity) {
                transform.x += info.normal.0 * info.penetration_depth;
                transform.y += info.normal.1 * info.penetration_depth;
            }
            if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                if info.normal.0.abs() > 0.5 {
                    velocity.dx = 0.0;
                }
            }
        }
    }

    /// 处理玩家与敌人的碰撞
    fn handle_player_enemy_collision(
        &self,
        world: &mut World,
        player_entity: Entity,
        enemy_entity: Entity,
        info: &CollisionInfo,
    ) {
        let player_invulnerable = world.get_component::<Player>(player_entity).map(|p| p.invulnerable).unwrap_or(false);

        let is_stomping = info.normal.1 < -0.5;

        if is_stomping {
            let bounce_force = world.get_component::<Enemy>(enemy_entity).map(|e| e.stomp_bounce_force).unwrap_or(8.0);

            if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                velocity.dy = -bounce_force;
            }

            if let Some(health) = world.get_component_mut::<Health>(enemy_entity) {
                health.current = 0;
            }
        }
        else {
            if player_invulnerable {
                return;
            }

            let enemy_damage = world.get_component::<Enemy>(enemy_entity).map(|e| e.damage).unwrap_or(1);

            if let Some(health) = world.get_component_mut::<Health>(player_entity) {
                if health.current > 0 {
                    health.current = health.current.saturating_sub(enemy_damage);
                }
            }

            if let Some(player) = world.get_component_mut::<Player>(player_entity) {
                player.invulnerable = true;
                player.invulnerable_timer = 1.5;
            }

            if let Some(velocity) = world.get_component_mut::<Velocity>(player_entity) {
                velocity.dx = -info.normal.0 * 5.0;
                velocity.dy = -5.0;
            }
        }
    }

    /// 处理玩家与可收集物品的碰撞
    fn handle_player_collectible_collision(&self, world: &mut World, collectible_entity: Entity) {
        if let Some(collectible) = world.get_component_mut::<Collectible>(collectible_entity) {
            if !collectible.collected {
                collectible.collected = true;
            }
        }
    }
}

/// 平台系统
/// 处理移动平台、易碎平台、消失平台、弹簧平台、传送带平台和冰面平台的行为
pub struct PlatformSystem {
    #[allow(dead_code)]
    screen_width: f32,
    #[allow(dead_code)]
    screen_height: f32,
}

impl PlatformSystem {
    /// 创建新的平台系统
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self { screen_width, screen_height }
    }
}

impl System for PlatformSystem {
    fn name(&self) -> &str {
        "platform_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let dt = world.get_resource::<DeltaTime>().map(|d| d.seconds).unwrap_or(1.0 / 60.0);

        let entities: Vec<Entity> = world.query::<Platform>().map(|(e, _)| e).collect();
        let mut entities_to_despawn: Vec<Entity> = Vec::new();

        for entity in entities {
            let platform_type = world.get_component::<Platform>(entity).map(|p| p.platform_type);

            match platform_type {
                Some(PlatformType::Moving) => {
                    self.handle_moving_platform(world, entity, dt);
                }
                Some(PlatformType::Breakable) => {
                    let should_despawn = self.handle_breakable_platform(world, entity, dt);
                    if should_despawn {
                        entities_to_despawn.push(entity);
                    }
                }
                Some(PlatformType::Disappearing) => {
                    self.handle_disappearing_platform(world, entity, dt);
                }
                Some(PlatformType::Spring) => {
                    self.handle_spring_platform(world, entity, dt);
                }
                Some(PlatformType::Conveyor) => {
                    self.handle_conveyor_platform(world, entity);
                }
                Some(PlatformType::Ice) => {
                    self.handle_ice_platform(world, entity);
                }
                _ => {}
            }
        }

        for entity in entities_to_despawn {
            let _ = world.despawn(entity);
        }

        let _ = (self.screen_width, self.screen_height);

        Ok(())
    }
}

impl PlatformSystem {
    /// 处理移动平台
    fn handle_moving_platform(&self, world: &mut World, entity: Entity, dt: f32) {
        let (move_start, move_end, move_speed, move_direction) = {
            let platform = match world.get_component::<Platform>(entity) {
                Some(p) => p,
                None => return,
            };
            (platform.move_start, platform.move_end, platform.move_speed, platform.move_direction)
        };

        let dx = move_end.0 - move_start.0;
        let dy = move_end.1 - move_start.1;
        let path_length = (dx * dx + dy * dy).sqrt();
        if path_length < 0.001 {
            return;
        }
        let dir_x = dx / path_length;
        let dir_y = dy / path_length;

        let velocity_dx = dir_x * move_speed * move_direction;
        let velocity_dy = dir_y * move_speed * move_direction;

        if let Some(transform) = world.get_component_mut::<Transform>(entity) {
            transform.x += velocity_dx * dt * 60.0;
            transform.y += velocity_dy * dt * 60.0;

            if move_direction > 0.0 {
                let progress_x = (transform.x - move_start.0) * dir_x;
                let progress_y = (transform.y - move_start.1) * dir_y;
                if progress_x + progress_y >= path_length {
                    if let Some(platform) = world.get_component_mut::<Platform>(entity) {
                        platform.move_direction = -1.0;
                    }
                }
            }
            else {
                let progress_x = (transform.x - move_start.0) * dir_x;
                let progress_y = (transform.y - move_start.1) * dir_y;
                if progress_x + progress_y <= 0.0 {
                    if let Some(platform) = world.get_component_mut::<Platform>(entity) {
                        platform.move_direction = 1.0;
                    }
                }
            }
        }
    }

    /// 处理易碎平台，返回是否应销毁
    fn handle_breakable_platform(&self, world: &mut World, entity: Entity, dt: f32) -> bool {
        let break_timer = world.get_component::<Platform>(entity).map(|p| p.break_timer).unwrap_or(0.0);

        if break_timer > 0.0 {
            if let Some(platform) = world.get_component_mut::<Platform>(entity) {
                platform.break_timer -= dt;
                if platform.break_timer <= 0.0 {
                    return true;
                }
            }
        }
        false
    }

    /// 处理消失平台
    fn handle_disappearing_platform(&self, world: &mut World, entity: Entity, dt: f32) {
        let (fade_timer, _fade_time, respawn_timer, respawn_time) = {
            let platform = match world.get_component::<Platform>(entity) {
                Some(p) => p,
                None => return,
            };
            (platform.fade_timer, platform.fade_time, platform.respawn_timer, platform.respawn_time)
        };

        if fade_timer > 0.0 {
            if let Some(platform) = world.get_component_mut::<Platform>(entity) {
                platform.fade_timer -= dt;
                if platform.fade_timer <= 0.0 {
                    platform.is_active = false;
                    platform.respawn_timer = respawn_time;
                    if let Some(sprite) = world.get_component_mut::<Sprite>(entity) {
                        sprite.visible = false;
                    }
                }
            }
        }
        else if respawn_timer > 0.0 {
            if let Some(platform) = world.get_component_mut::<Platform>(entity) {
                platform.respawn_timer -= dt;
                if platform.respawn_timer <= 0.0 {
                    platform.is_active = true;
                    platform.fade_timer = 0.0;
                    platform.respawn_timer = 0.0;
                    if let Some(sprite) = world.get_component_mut::<Sprite>(entity) {
                        sprite.visible = true;
                    }
                }
            }
        }
    }

    /// 处理弹簧平台
    fn handle_spring_platform(&self, world: &mut World, entity: Entity, dt: f32) {
        let spring_compressed = world.get_component::<Platform>(entity).map(|p| p.spring_compressed).unwrap_or(false);
        if spring_compressed {
            if let Some(platform) = world.get_component_mut::<Platform>(entity) {
                platform.spring_compressed = false;
            }
        }
        let _ = dt;
    }

    /// 处理传送带平台
    fn handle_conveyor_platform(&self, world: &mut World, entity: Entity) {
        let _ = world.get_component::<Platform>(entity).map(|p| (p.conveyor_direction, p.conveyor_speed));
    }

    /// 处理冰面平台
    fn handle_ice_platform(&self, world: &mut World, entity: Entity) {
        let _ = world.get_component::<Platform>(entity).map(|p| p.platform_type);
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
        Self { score: 0, scored_entities: HashSet::new() }
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
/// 处理敌人 AI 行为，包含状态机转换，支持巡逻/追踪/攻击/躲避/跳跃攻击/远程攻击
pub struct AISystem {
    #[allow(dead_code)]
    screen_width: f32,
    #[allow(dead_code)]
    screen_height: f32,
}

impl AISystem {
    /// 创建新的 AI 系统
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self { screen_width, screen_height }
    }
}

impl System for AISystem {
    fn name(&self) -> &str {
        "ai_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let dt = world.get_resource::<DeltaTime>().map(|d| d.seconds).unwrap_or(1.0 / 60.0);

        let player_pos: Option<(f32, f32)> = {
            let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
            let mut result = None;
            for pe in player_entities {
                if let Some(t) = world.get_component::<Transform>(pe) {
                    result = Some((t.x, t.y));
                    break;
                }
            }
            result
        };

        let player_pos = match player_pos {
            Some(pos) => pos,
            None => return Ok(()),
        };

        let entities: Vec<Entity> = world.query::<AI>().map(|(e, _)| e).collect();
        for entity in entities {
            self.update_ai_state(world, entity, player_pos);

            let behavior = world.get_component::<AI>(entity).map(|ai| ai.behavior);
            match behavior {
                Some(AIBehavior::Patrol) => self.handle_patrol(entity, world, dt),
                Some(AIBehavior::Chase) => self.handle_chase(entity, world, player_pos, dt),
                Some(AIBehavior::Attack) => self.handle_attack(entity, world, player_pos),
                Some(AIBehavior::Evade) => self.handle_evade(entity, world, player_pos, dt),
                Some(AIBehavior::JumpAttack) => self.handle_jump_attack(entity, world, player_pos, dt),
                Some(AIBehavior::RangedAttack) => self.handle_ranged_attack(entity, world, player_pos, dt),
                None => {}
            }
        }

        Ok(())
    }
}

impl AISystem {
    /// 更新 AI 状态机
    fn update_ai_state(&self, world: &mut World, entity: Entity, player_pos: (f32, f32)) {
        let (behavior, detection_range, attack_range, evade_health_threshold) = {
            let ai = match world.get_component::<AI>(entity) {
                Some(a) => a,
                None => return,
            };
            (ai.behavior, ai.detection_range, ai.attack_range, ai.evade_health_threshold)
        };

        let ai_pos = world.get_component::<Transform>(entity).map(|t| (t.x, t.y));
        let ai_pos = match ai_pos {
            Some(pos) => pos,
            None => return,
        };

        let dx = player_pos.0 - ai_pos.0;
        let dy = player_pos.1 - ai_pos.1;
        let distance = (dx * dx + dy * dy).sqrt();

        let health_ratio = world
            .get_component::<Health>(entity)
            .map(|h| if h.max > 0 { h.current as f32 / h.max as f32 } else { 0.0 })
            .unwrap_or(1.0);

        let new_behavior = if health_ratio <= evade_health_threshold && health_ratio > 0.0 {
            AIBehavior::Evade
        }
        else {
            match behavior {
                AIBehavior::Patrol => {
                    if distance <= detection_range {
                        AIBehavior::Chase
                    }
                    else {
                        AIBehavior::Patrol
                    }
                }
                AIBehavior::Chase => {
                    if distance > detection_range {
                        AIBehavior::Patrol
                    }
                    else if distance <= attack_range {
                        AIBehavior::Attack
                    }
                    else {
                        AIBehavior::Chase
                    }
                }
                AIBehavior::Attack => {
                    if distance > detection_range {
                        AIBehavior::Patrol
                    }
                    else if distance > attack_range {
                        AIBehavior::Chase
                    }
                    else {
                        let on_ground = world.get_component::<PhysicsBody>(entity).map(|pb| pb.on_ground).unwrap_or(false);
                        let has_nav = world.get_component::<AI>(entity).map(|ai| !ai.nav_waypoints.is_empty()).unwrap_or(false);
                        if on_ground && distance > attack_range * 0.5 {
                            AIBehavior::JumpAttack
                        }
                        else if has_nav && distance > attack_range * 0.3 {
                            AIBehavior::RangedAttack
                        }
                        else {
                            AIBehavior::Attack
                        }
                    }
                }
                AIBehavior::Evade => {
                    if health_ratio > evade_health_threshold {
                        if distance <= detection_range { AIBehavior::Chase } else { AIBehavior::Patrol }
                    }
                    else {
                        AIBehavior::Evade
                    }
                }
                AIBehavior::JumpAttack => {
                    if distance > detection_range {
                        AIBehavior::Patrol
                    }
                    else if distance > attack_range {
                        AIBehavior::Chase
                    }
                    else {
                        AIBehavior::JumpAttack
                    }
                }
                AIBehavior::RangedAttack => {
                    if distance > detection_range {
                        AIBehavior::Patrol
                    }
                    else if distance > attack_range {
                        AIBehavior::Chase
                    }
                    else {
                        AIBehavior::RangedAttack
                    }
                }
            }
        };

        if new_behavior != behavior {
            if let Some(ai) = world.get_component_mut::<AI>(entity) {
                ai.behavior = new_behavior;
            }
        }
    }

    /// 处理巡逻行为
    fn handle_patrol(&self, entity: Entity, world: &mut World, dt: f32) {
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
        else {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = dx / distance * move_speed;
                velocity.dy = dy / distance * move_speed;
            }
        }

        let _ = dt;
    }

    /// 处理追踪行为
    fn handle_chase(&self, entity: Entity, world: &mut World, player_pos: (f32, f32), dt: f32) {
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

        let dx = player_pos.0 - transform_x;
        let dy = player_pos.1 - transform_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 5.0 {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = dx / distance * move_speed;
                velocity.dy = dy / distance * move_speed;
            }
        }

        let _ = dt;
    }

    /// 处理攻击行为
    fn handle_attack(&self, entity: Entity, world: &mut World, player_pos: (f32, f32)) {
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

        let dx = player_pos.0 - transform_x;
        let dy = player_pos.1 - transform_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 200.0 && distance > 0.0 {
            let dash_speed = move_speed * 2.5;
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = dx / distance * dash_speed;
                velocity.dy = dy / distance * dash_speed;
            }
        }
    }

    /// 处理躲避行为
    fn handle_evade(&self, entity: Entity, world: &mut World, player_pos: (f32, f32), dt: f32) {
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

        let dx = transform_x - player_pos.0;
        let dy = transform_y - player_pos.1;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = dx / distance * move_speed;
                velocity.dy = dy / distance * move_speed;
            }
        }

        let _ = dt;
    }

    /// 处理跳跃攻击行为
    fn handle_jump_attack(&self, entity: Entity, world: &mut World, player_pos: (f32, f32), dt: f32) {
        let (transform_x, transform_y, move_speed, jump_attack_timer, jump_attack_cooldown, on_ground) = {
            let transform = match world.get_component::<Transform>(entity) {
                Some(t) => (t.x, t.y),
                None => return,
            };
            let ai = match world.get_component::<AI>(entity) {
                Some(a) => (a.move_speed, a.jump_attack_timer, a.jump_attack_cooldown),
                None => return,
            };
            let pb = world.get_component::<PhysicsBody>(entity).map(|pb| pb.on_ground).unwrap_or(false);
            (transform.0, transform.1, ai.0, ai.1, ai.2, pb)
        };

        let dx = player_pos.0 - transform_x;
        let distance = (dx * dx + (player_pos.1 - transform_y) * (player_pos.1 - transform_y)).sqrt();

        if on_ground && jump_attack_timer <= 0.0 && distance < 200.0 {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dy = -12.0;
                velocity.dx = if dx > 0.0 { move_speed * 2.0 } else { -move_speed * 2.0 };
            }
            if let Some(ai) = world.get_component_mut::<AI>(entity) {
                ai.jump_attack_timer = jump_attack_cooldown;
            }
        }

        if let Some(ai) = world.get_component_mut::<AI>(entity) {
            ai.jump_attack_timer -= dt;
        }
    }

    /// 处理远程攻击行为
    fn handle_ranged_attack(&self, entity: Entity, world: &mut World, player_pos: (f32, f32), dt: f32) {
        let (transform_x, transform_y, move_speed, ranged_attack_timer, ranged_attack_cooldown) = {
            let transform = match world.get_component::<Transform>(entity) {
                Some(t) => (t.x, t.y),
                None => return,
            };
            let ai = match world.get_component::<AI>(entity) {
                Some(a) => (a.move_speed, a.ranged_attack_timer, a.ranged_attack_cooldown),
                None => return,
            };
            (transform.0, transform.1, ai.0, ai.1, ai.2)
        };

        let dx = player_pos.0 - transform_x;
        let distance = dx.abs();

        if distance > 5.0 {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = if dx > 0.0 { move_speed * 0.5 } else { -move_speed * 0.5 };
            }
        }

        if ranged_attack_timer <= 0.0 {
            let projectile = world.spawn().id();
            let direction = if dx > 0.0 { 1.0 } else { -1.0 };
            let _ =
                world.add_component(projectile, Transform { x: transform_x, y: transform_y - 10.0, rotation: 0.0, scale: 1.0 });
            let _ = world.add_component(projectile, Velocity { dx: direction * 5.0, dy: 0.0, ax: 0.0, ay: 0.0 });
            let _ = world
                .add_component(projectile, Collider { collider_type: ColliderType::Circle { radius: 5.0 }, layer: 8, mask: 2 });

            if let Some(ai) = world.get_component_mut::<AI>(entity) {
                ai.ranged_attack_timer = ranged_attack_cooldown;
            }
        }

        if let Some(ai) = world.get_component_mut::<AI>(entity) {
            ai.ranged_attack_timer -= dt;
        }
    }
}

/// 渲染系统
/// 处理渲染逻辑，按实体类型分配不同颜色
pub struct RenderSystem {
    #[allow(dead_code)]
    screen_width: f32,
    #[allow(dead_code)]
    screen_height: f32,
}

impl RenderSystem {
    /// 创建新的渲染系统
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self { screen_width, screen_height }
    }
}

impl System for RenderSystem {
    fn name(&self) -> &str {
        "render_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
            buffer.clear();
        }
        else {
            world.insert_resource(DrawCommandBuffer::new());
        }

        let entities: Vec<Entity> = world.query::<Sprite>().filter(|(_, sprite)| sprite.visible).map(|(e, _)| e).collect();

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
                }
                else {
                    (0.0, 0.0, w, h)
                }
            };

            let color = if world.get_component::<Player>(entity).is_some() {
                Color::new(0.2, 0.6, 1.0, 1.0)
            }
            else if world.get_component::<Enemy>(entity).is_some() {
                Color::new(1.0, 0.3, 0.2, 1.0)
            }
            else if world.get_component::<Platform>(entity).is_some() {
                Color::new(0.5, 0.35, 0.2, 1.0)
            }
            else if world.get_component::<Collectible>(entity).is_some() {
                let collected = world.get_component::<Collectible>(entity).map(|c| c.collected).unwrap_or(false);
                if collected { Color::new(1.0, 1.0, 0.0, 0.3) } else { Color::new(1.0, 1.0, 0.0, 1.0) }
            }
            else {
                Color::WHITE
            };

            let command = DrawCommand::Rect { rect: Rect::new(x, y, width, height), color, corner_radius: 0.0 };

            if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
                buffer.push(command);
            }
        }

        Ok(())
    }
}
