//! STG 引擎系统定义
//! 定义游戏中使用的各种 ECS 系统

use crate::{
    boss::{Boss, BossMovePattern, PhaseTransition},
    bullet_pattern::{BulletEmitter, BulletLifetime, BulletPattern, BulletPool, PatternType},
    components::*,
    item::{Item, ItemSpawner, ItemType},
};
use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use gg_render::{Color, DrawCommand, Rect};
use std::collections::HashMap;

/// 行为树节点执行结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// 执行成功
    Success,
    /// 执行失败
    Failure,
    /// 仍在执行中
    Running,
}

/// 行为树节点执行结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    /// 执行成功
    Success,
    /// 执行失败
    Failure,
    /// 仍在执行中
    Running,
}

/// 输入系统
/// 处理玩家输入，从 World 的 InputState 资源读取按键状态
pub struct InputSystem {
    /// 移动速度
    move_speed: f32,
}

impl InputSystem {
    /// 创建新的输入系统
    pub fn new(move_speed: f32) -> Self {
        Self { move_speed }
    }
}

impl System for InputSystem {
    fn name(&self) -> &str {
        "input_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (up, down, left, right, shoot, special) = world
            .get_resource::<InputState>()
            .map(|s| (s.up_pressed, s.down_pressed, s.left_pressed, s.right_pressed, s.shoot_pressed, s.special_pressed))
            .unwrap_or((false, false, false, false, false, false));

        let move_speed = self.move_speed;
        let entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for entity in entities {
            if let Some(velocity) = world.get_component_mut::<Velocity>(entity) {
                velocity.dx = 0.0;
                velocity.dy = 0.0;
                if left {
                    velocity.dx -= move_speed;
                }
                if right {
                    velocity.dx += move_speed;
                }
                if up {
                    velocity.dy -= move_speed;
                }
                if down {
                    velocity.dy += move_speed;
                }
            }
        }

        let _ = (shoot, special);
        Ok(())
    }
}

/// 移动系统
/// 处理实体移动和边界约束
pub struct MovementSystem;

impl System for MovementSystem {
    fn name(&self) -> &str {
        "movement_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let (screen_w, screen_h) = world.get_resource::<ScreenBounds>().map(|b| (b.width, b.height)).unwrap_or((800.0, 600.0));

        let entities: Vec<Entity> = world.query::<Velocity>().map(|(e, _)| e).collect();
        for entity in entities {
            let (dx, dy) = world.get_component::<Velocity>(entity).map(|v| (v.dx, v.dy)).unwrap_or((0.0, 0.0));
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                transform.x += dx;
                transform.y += dy;
                if transform.x < 0.0 {
                    transform.x = 0.0;
                }
                if transform.x > screen_w {
                    transform.x = screen_w;
                }
                if transform.y < 0.0 {
                    transform.y = 0.0;
                }
                if transform.y > screen_h {
                    transform.y = screen_h;
                }
            }
        }
        Ok(())
    }
}

/// 碰撞系统
/// 使用空间哈希网格优化碰撞检测，子弹使用专用轻量检测路径
pub struct CollisionSystem {
    /// 空间哈希单元格大小
    cell_size: f32,
}

/// 子弹碰撞轻量数据
struct BulletCollisionData {
    /// 子弹实体
    entity: Entity,
    /// X 坐标
    x: f32,
    /// Y 坐标
    y: f32,
    /// 碰撞半径
    radius: f32,
    /// 伤害值
    damage: u32,
    /// 发射者类型
    shooter_type: ShooterType,
}

/// 非子弹实体碰撞轻量数据
struct SolidCollisionData {
    /// 实体
    entity: Entity,
    /// X 坐标
    x: f32,
    /// Y 坐标
    y: f32,
    /// 碰撞体
    collider: Collider,
    /// 是否为玩家
    is_player: bool,
    /// 是否为敌人
    is_enemy: bool,
    /// 是否为道具
    is_item: bool,
}

impl CollisionSystem {
    /// 创建新的碰撞系统
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size }
    }

    /// 计算空间哈希键
    fn hash_key(&self, x: f32, y: f32) -> (i32, i32) {
        ((x / self.cell_size).floor() as i32, (y / self.cell_size).floor() as i32)
    }

    /// 检查圆形与碰撞体的碰撞
    fn circle_vs_collider(x1: f32, y1: f32, r1: f32, c2: &Collider, t2_x: f32, t2_y: f32) -> bool {
        match &c2.collider_type {
            ColliderType::Circle { radius: r2 } => {
                let dx = x1 - t2_x;
                let dy = y1 - t2_y;
                (dx * dx + dy * dy).sqrt() < (r1 + r2)
            }
            ColliderType::Rectangle { width, height } => {
                let closest_x = x1.max(t2_x - width / 2.0).min(t2_x + width / 2.0);
                let closest_y = y1.max(t2_y - height / 2.0).min(t2_y + height / 2.0);
                let dx = x1 - closest_x;
                let dy = y1 - closest_y;
                (dx * dx + dy * dy).sqrt() < r1
            }
        }
    }

    /// 检查两个碰撞体是否碰撞
    fn check_collision(c1: &Collider, t1: &Transform, c2: &Collider, t2: &Transform) -> bool {
        if (c1.layer & c2.mask) == 0 || (c2.layer & c1.mask) == 0 {
            return false;
        }
        match (&c1.collider_type, &c2.collider_type) {
            (ColliderType::Circle { radius: r1 }, ColliderType::Circle { radius: r2 }) => {
                let dx = t1.x - t2.x;
                let dy = t1.y - t2.y;
                (dx * dx + dy * dy).sqrt() < (r1 + r2)
            }
            (ColliderType::Rectangle { width: w1, height: h1 }, ColliderType::Rectangle { width: w2, height: h2 }) => {
                let l1 = t1.x - w1 / 2.0;
                let r1b = t1.x + w1 / 2.0;
                let t1b = t1.y - h1 / 2.0;
                let b1 = t1.y + h1 / 2.0;
                let l2 = t2.x - w2 / 2.0;
                let r2b = t2.x + w2 / 2.0;
                let t2b = t2.y - h2 / 2.0;
                let b2 = t2.y + h2 / 2.0;
                r1b > l2 && l1 < r2b && b1 > t2b && t1b < b2
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
        let entities = world.entities();

        let mut bullets: Vec<BulletCollisionData> = Vec::new();
        let mut solids: Vec<SolidCollisionData> = Vec::new();
        let mut solid_grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();

        for &entity in &entities {
            let is_bullet = world.get_component::<Bullet>(entity).is_some();
            if is_bullet {
                let transform = match world.get_component::<Transform>(entity) {
                    Some(t) => (t.x, t.y),
                    None => continue,
                };
                let collider = world.get_component::<Collider>(entity);
                let bullet_comp = world.get_component::<Bullet>(entity);
                let radius = collider
                    .as_ref()
                    .and_then(|c| match &c.collider_type {
                        ColliderType::Circle { radius } => Some(*radius),
                        _ => None,
                    })
                    .unwrap_or(4.0);
                let damage = bullet_comp.as_ref().map(|b| b.damage).unwrap_or(1);
                let shooter_type = bullet_comp.as_ref().map(|b| b.shooter_type).unwrap_or(ShooterType::Player);
                bullets.push(BulletCollisionData { entity, x: transform.0, y: transform.1, radius, damage, shooter_type });
            }
            else if world.get_component::<Collider>(entity).is_some() {
                let transform = match world.get_component::<Transform>(entity) {
                    Some(t) => (t.x, t.y),
                    None => continue,
                };
                let collider = match world.get_component::<Collider>(entity) {
                    Some(c) => c.clone(),
                    None => continue,
                };
                let is_player = world.get_component::<Player>(entity).is_some();
                let is_enemy = world.get_component::<Enemy>(entity).is_some();
                let is_item = world.get_component::<Item>(entity).is_some();
                let idx = solids.len();
                let key = self.hash_key(transform.0, transform.1);
                solid_grid.entry(key).or_default().push(idx);
                solids.push(SolidCollisionData {
                    entity,
                    x: transform.0,
                    y: transform.1,
                    collider,
                    is_player,
                    is_enemy,
                    is_item,
                });
            }
        }

        let mut player_hits: Vec<Entity> = Vec::new();
        let mut enemy_hits: Vec<(Entity, u32)> = Vec::new();
        let mut bullets_to_despawn: Vec<Entity> = Vec::new();
        let mut items_to_collect: Vec<Entity> = Vec::new();

        for bullet in &bullets {
            let key = self.hash_key(bullet.x, bullet.y);
            for dx in -1..=1i32 {
                for dy in -1..=1i32 {
                    if let Some(cell) = solid_grid.get(&(key.0 + dx, key.1 + dy)) {
                        for &solid_idx in cell {
                            let solid = &solids[solid_idx];
                            if (bullet.shooter_type == ShooterType::Player && solid.is_enemy)
                                || (bullet.shooter_type == ShooterType::Enemy && solid.is_player)
                            {
                                if Self::circle_vs_collider(
                                    bullet.x,
                                    bullet.y,
                                    bullet.radius,
                                    &solid.collider,
                                    solid.x,
                                    solid.y,
                                ) {
                                    if solid.is_player {
                                        player_hits.push(solid.entity);
                                    }
                                    else if solid.is_enemy {
                                        enemy_hits.push((solid.entity, bullet.damage));
                                    }
                                    bullets_to_despawn.push(bullet.entity);
                                }
                            }
                            if solid.is_item && bullet.shooter_type == ShooterType::Player {
                                if Self::circle_vs_collider(
                                    bullet.x,
                                    bullet.y,
                                    bullet.radius,
                                    &solid.collider,
                                    solid.x,
                                    solid.y,
                                ) {
                                    items_to_collect.push(solid.entity);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut player_solid_indices: Vec<usize> = Vec::new();
        let mut enemy_solid_indices: Vec<usize> = Vec::new();
        for (idx, solid) in solids.iter().enumerate() {
            if solid.is_player {
                player_solid_indices.push(idx);
            }
            if solid.is_enemy {
                enemy_solid_indices.push(idx);
            }
        }

        for &p_idx in &player_solid_indices {
            let player = &solids[p_idx];
            let player_key = self.hash_key(player.x, player.y);
            for dx in -1..=1i32 {
                for dy in -1..=1i32 {
                    if let Some(cell) = solid_grid.get(&(player_key.0 + dx, player_key.1 + dy)) {
                        for &e_idx in cell {
                            if e_idx == p_idx {
                                continue;
                            }
                            let enemy = &solids[e_idx];
                            if enemy.is_enemy {
                                if Self::check_collision(
                                    &player.collider,
                                    &Transform { x: player.x, y: player.y, rotation: 0.0, scale: 1.0 },
                                    &enemy.collider,
                                    &Transform { x: enemy.x, y: enemy.y, rotation: 0.0, scale: 1.0 },
                                ) {
                                    player_hits.push(player.entity);
                                }
                            }
                            if enemy.is_item {
                                if Self::circle_vs_collider(player.x, player.y, 16.0, &enemy.collider, enemy.x, enemy.y) {
                                    items_to_collect.push(enemy.entity);
                                }
                            }
                        }
                    }
                }
            }
        }

        for entity in player_hits {
            let invulnerable = world.get_component::<Player>(entity).map(|p| p.invulnerable).unwrap_or(false);
            if !invulnerable {
                if let Some(health) = world.get_component_mut::<Health>(entity) {
                    if health.current > 0 {
                        health.current -= 1;
                    }
                }
                if let Some(player) = world.get_component_mut::<Player>(entity) {
                    player.invulnerable = true;
                    player.invulnerable_frames = 90;
                }
            }
        }

        for (entity, damage) in enemy_hits {
            let boss_invulnerable =
                world.get_component::<Boss>(entity).map(|b| b.remaining_invulnerable_frames > 0).unwrap_or(false);
            if !boss_invulnerable {
                if let Some(health) = world.get_component_mut::<Health>(entity) {
                    health.current = health.current.saturating_sub(damage);
                }
            }
        }

        for entity in &bullets_to_despawn {
            let _ = world.despawn(*entity);
        }

        for entity in items_to_collect {
            let item_type = world.get_component::<Item>(entity).map(|i| i.item_type);
            let item_value = world.get_component::<Item>(entity).map(|i| i.value).unwrap_or(1);
            if let Some(item_type) = item_type {
                let player_entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
                for player_entity in player_entities {
                    match item_type {
                        ItemType::PowerUp => {
                            if let Some(player) = world.get_component_mut::<Player>(player_entity) {
                                player.power_level = player.power_level.saturating_add(1);
                            }
                        }
                        ItemType::ScoreBonus => {
                            if let Some(state) = world.get_resource_mut::<GameState>() {
                                state.score += item_value * 100;
                            }
                        }
                        ItemType::Bomb => {
                            if let Some(player) = world.get_component_mut::<Player>(player_entity) {
                                player.bomb_count = player.bomb_count.saturating_add(1);
                            }
                        }
                        ItemType::Life => {
                            if let Some(state) = world.get_resource_mut::<GameState>() {
                                state.lives += 1;
                            }
                        }
                        ItemType::Shield => {
                            if let Some(player) = world.get_component_mut::<Player>(player_entity) {
                                player.invulnerable = true;
                                player.invulnerable_frames = 180;
                            }
                        }
                    }
                }
            }
            let _ = world.despawn(entity);
        }

        Ok(())
    }
}

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

/// 弹幕发射器系统
/// 根据弹幕模式生成敌人子弹
pub struct BulletEmitterSystem;

impl BulletEmitterSystem {
    /// 创建新的弹幕发射器系统
    pub fn new() -> Self {
        Self
    }

    /// 根据弹幕模式生成子弹
    fn emit_pattern(world: &mut World, emitter_entity: Entity, pattern: &BulletPattern, spiral_angle: &mut f32) {
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
fn spawn_enemy_bullet(world: &mut World, x: f32, y: f32, dx: f32, dy: f32, damage: u32) {
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

/// 无敌帧系统
/// 处理玩家无敌帧倒计时
pub struct InvulnerabilitySystem;

impl System for InvulnerabilitySystem {
    fn name(&self) -> &str {
        "invulnerability_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let entities: Vec<Entity> = world.query::<Player>().map(|(e, _)| e).collect();
        for entity in entities {
            let frames = world.get_component::<Player>(entity).map(|p| p.invulnerable_frames).unwrap_or(0);
            if frames > 0 {
                if let Some(player) = world.get_component_mut::<Player>(entity) {
                    player.invulnerable_frames = player.invulnerable_frames.saturating_sub(1);
                    if player.invulnerable_frames == 0 {
                        player.invulnerable = false;
                    }
                }
            }
        }
        Ok(())
    }
}

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

/// 渲染系统
/// 处理渲染逻辑
pub struct RenderSystem;

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

            let is_player = world.get_component::<Player>(entity).is_some();
            let is_enemy = world.get_component::<Enemy>(entity).is_some();
            let is_bullet = world.get_component::<Bullet>(entity).is_some();
            let is_item = world.get_component::<Item>(entity).is_some();
            let is_boss = world.get_component::<Boss>(entity).is_some();

            let color = if is_player {
                Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 }
            }
            else if is_boss {
                Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 }
            }
            else if is_enemy {
                Color::RED
            }
            else if is_item {
                Color::GREEN
            }
            else if is_bullet {
                let is_player_bullet = world
                    .get_component::<Bullet>(entity)
                    .map(|b| matches!(b.shooter_type, ShooterType::Player))
                    .unwrap_or(true);
                if is_player_bullet { Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 } } else { Color::RED }
            }
            else {
                Color::WHITE
            };

            let command = DrawCommand::Rect {
                rect: Rect::new(x - width / 2.0, y - height / 2.0, width, height),
                color,
                corner_radius: 0.0,
            };

            if let Some(buffer) = world.get_resource_mut::<DrawCommandBuffer>() {
                buffer.push(command);
            }
        }
        Ok(())
    }
}

/// 音频系统
/// 读取 AudioBus 中的音频事件，映射为音频命令，委托给运行时音频后端执行
pub struct AudioSystem {
    /// 音频事件到片段路径的映射
    event_clip_map: HashMap<AudioEvent, String>,
    /// 主音量
    master_volume: f32,
    /// 音效音量
    sfx_volume: f32,
}

impl AudioSystem {
    /// 创建新的音频系统
    pub fn new(config: &AudioConfig) -> Self {
        Self {
            event_clip_map: config.event_clip_map.clone(),
            master_volume: config.master_volume,
            sfx_volume: config.sfx_volume,
        }
    }
}

impl System for AudioSystem {
    fn name(&self) -> &str {
        "audio_system"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        let events: Vec<AudioEvent> = world.get_resource::<AudioBus>().map(|bus| bus.events.clone()).unwrap_or_default();

        for event in &events {
            if let Some(clip_path) = self.event_clip_map.get(event) {
                let final_volume = self.master_volume * self.sfx_volume;
                if let Some(bus) = world.get_resource_mut::<AudioBus>() {
                    bus.send(AudioCommand::Play(clip_path.clone()));
                    bus.send(AudioCommand::SetVolume(final_volume));
                }
                log::info!("[AudioSystem] 事件 {:?} -> 播放 {:?}", event, clip_path);
            }
            else {
                log::debug!("[AudioSystem] 事件 {:?} 无对应片段映射，已跳过", event);
            }
        }

        let entities: Vec<Entity> = world.query::<AudioSource>().map(|(e, _)| e).collect();
        for entity in entities {
            let play_on_start = world.get_component::<AudioSource>(entity).map(|s| s.play_on_start).unwrap_or(false);
            if !play_on_start {
                continue;
            }

            let (clip_path, volume) =
                world.get_component::<AudioSource>(entity).map(|s| (s.clip_path.clone(), s.volume)).unwrap_or_default();

            if clip_path.is_empty() {
                continue;
            }

            let final_volume = self.master_volume * self.sfx_volume * volume;
            if let Some(bus) = world.get_resource_mut::<AudioBus>() {
                bus.send(AudioCommand::Play(clip_path.clone()));
                bus.send(AudioCommand::SetVolume(final_volume));
            }
            log::info!("[AudioSystem] AudioSource 自动播放: {:?}", clip_path);

            if let Some(source) = world.get_component_mut::<AudioSource>(entity) {
                source.play_on_start = false;
            }
        }

        if let Some(bus) = world.get_resource_mut::<AudioBus>() {
            bus.events.clear();
        }

        Ok(())
    }
}
