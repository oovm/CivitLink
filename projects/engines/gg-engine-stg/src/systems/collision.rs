//! 碰撞系统
//! 使用空间哈希网格优化碰撞检测，子弹使用专用轻量检测路径

use crate::components::*;
use crate::item::{Item, ItemType};
use crate::boss::Boss;
use gg_core::GResult;
use gg_ecs::{Entity, System, World};
use std::collections::HashMap;

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
