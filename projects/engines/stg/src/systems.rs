//! STG 引擎系统定义
//! 定义游戏中使用的各种 ECS 系统

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Query, System, World};
use std::time::{Duration, Instant};

/// 输入系统
/// 处理玩家输入
pub struct InputSystem {
    /// 移动速度
    move_speed: f32,
    /// 上方向键是否按下
    up_pressed: bool,
    /// 下方向键是否按下
    down_pressed: bool,
    /// 左方向键是否按下
    left_pressed: bool,
    /// 右方向键是否按下
    right_pressed: bool,
    /// 射击键是否按下
    shoot_pressed: bool,
    /// 特殊武器键是否按下
    special_pressed: bool,
}

impl InputSystem {
    /// 创建新的输入系统
    pub fn new(move_speed: f32) -> Self {
        Self {
            move_speed,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            shoot_pressed: false,
            special_pressed: false,
        }
    }

    /// 设置按键状态
    pub fn set_key_state(&mut self, key: &str, pressed: bool) {
        match key {
            "Up" => self.up_pressed = pressed,
            "Down" => self.down_pressed = pressed,
            "Left" => self.left_pressed = pressed,
            "Right" => self.right_pressed = pressed,
            "Space" => self.shoot_pressed = pressed,
            "Shift" => self.special_pressed = pressed,
            _ => {},
        }
    }
}

impl System for InputSystem {
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Velocity, &Player)>();

        for (velocity, _) in query.iter_mut() {
            // 重置速度
            velocity.dx = 0.0;
            velocity.dy = 0.0;

            // 处理水平移动
            if self.left_pressed {
                velocity.dx -= self.move_speed;
            }
            if self.right_pressed {
                velocity.dx += self.move_speed;
            }

            // 处理垂直移动
            if self.up_pressed {
                velocity.dy -= self.move_speed;
            }
            if self.down_pressed {
                velocity.dy += self.move_speed;
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
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Transform, &Velocity)>();

        for (transform, velocity) in query.iter_mut() {
            // 更新位置
            transform.x += velocity.dx;
            transform.y += velocity.dy;

            // 边界检查
            if transform.x < 0.0 {
                transform.x = 0.0;
            }
            if transform.x > self.screen_width {
                transform.x = self.screen_width;
            }
            if transform.y < 0.0 {
                transform.y = 0.0;
            }
            if transform.y > self.screen_height {
                transform.y = self.screen_height;
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
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let entities = world.entities().to_vec();

        // 检查所有实体对之间的碰撞
        for i in 0..entities.len() {
            for j in i + 1..entities.len() {
                let entity1 = entities[i];
                let entity2 = entities[j];

                // 检查两个实体是否都有碰撞体和变换组件
                if let (Some(collider1), Some(transform1)) = (
                    world.get_component::<Collider>(entity1),
                    world.get_component::<Transform>(entity1),
                ) {
                    if let (Some(collider2), Some(transform2)) = (
                        world.get_component::<Collider>(entity2),
                        world.get_component::<Transform>(entity2),
                    ) {
                        // 检查碰撞
                        if self.check_collision(collider1, transform1, collider2, transform2) {
                            // 处理碰撞响应
                            self.handle_collision(world, entity1, entity2);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl CollisionSystem {
    /// 处理碰撞响应
    fn handle_collision(&self, world: &mut World, entity1: u32, entity2: u32) {
        // 检查是否是玩家与敌人的碰撞
        if world.has_component::<Player>(entity1) && world.has_component::<Enemy>(entity2) {
            // 处理玩家受伤
            if let Some(mut health) = world.get_component_mut::<Health>(entity1) {
                if health.current > 0 {
                    health.current -= 1;
                }
            }
        }

        // 检查是否是玩家子弹与敌人的碰撞
        if world.has_component::<Bullet>(entity1) && world.has_component::<Enemy>(entity2) {
            if let Some(bullet) = world.get_component::<Bullet>(entity1) {
                if matches!(bullet.shooter_type, ShooterType::Player) {
                    // 处理敌人受伤
                    if let Some(mut health) = world.get_component_mut::<Health>(entity2) {
                        health.current -= bullet.damage;
                    }
                    // 销毁子弹
                    world.despawn(entity1);
                }
            }
        }

        // 检查是否是敌人子弹与玩家的碰撞
        if world.has_component::<Bullet>(entity2) && world.has_component::<Player>(entity1) {
            if let Some(bullet) = world.get_component::<Bullet>(entity2) {
                if matches!(bullet.shooter_type, ShooterType::Enemy) {
                    // 处理玩家受伤
                    if let Some(mut health) = world.get_component_mut::<Health>(entity1) {
                        if health.current > 0 {
                            health.current -= bullet.damage;
                        }
                    }
                    // 销毁子弹
                    world.despawn(entity2);
                }
            }
        }
    }
}

/// 武器系统
/// 处理武器和射击逻辑
pub struct WeaponSystem {
    /// 子弹速度
    bullet_speed: f32,
    /// 当前时间
    current_time: Instant,
}

impl WeaponSystem {
    /// 创建新的武器系统
    pub fn new(bullet_speed: f32) -> Self {
        Self {
            bullet_speed,
            current_time: Instant::now(),
        }
    }
}

impl System for WeaponSystem {
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&Transform, &Weapon, &Player)>();
        let current_time = Instant::now();

        for (transform, weapon, _) in query.iter() {
            // 检查是否可以射击
            let elapsed = current_time.duration_since(self.current_time).as_millis() as u64;
            if elapsed - weapon.last_fire_time > (1000.0 / weapon.fire_rate) as u64 {
                // 创建子弹
                self.create_bullet(world, transform, weapon);
            }
        }

        self.current_time = current_time;
        Ok(())
    }
}

impl WeaponSystem {
    /// 创建子弹
    fn create_bullet(&self, world: &mut World, transform: &Transform, weapon: &Weapon) {
        let bullet_entity = world.spawn();

        // 添加变换组件
        world.add_component(bullet_entity, Transform {
            x: transform.x,
            y: transform.y,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();

        // 添加速度组件
        world.add_component(bullet_entity, Velocity {
            dx: 0.0,
            dy: -self.bullet_speed,
            ax: 0.0,
            ay: 0.0,
        }).unwrap();

        // 添加精灵组件
        world.add_component(bullet_entity, Sprite {
            path: "bullet.png".to_string(),
            width: 8.0,
            height: 16.0,
            visible: true,
        }).unwrap();

        // 添加碰撞体组件
        world.add_component(bullet_entity, Collider {
            collider_type: ColliderType::Circle { radius: 4.0 },
            layer: 2,
            mask: 4,
        }).unwrap();

        // 添加子弹组件
        world.add_component(bullet_entity, Bullet {
            bullet_type: weapon.weapon_type.clone(),
            damage: 1,
            shooter_type: ShooterType::Player,
        }).unwrap();
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
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Transform, &mut Velocity, &mut AI, &Enemy)>();

        for (transform, velocity, ai, _) in query.iter_mut() {
            match ai.behavior {
                AIBehavior::Patrol => {
                    self.handle_patrol(transform, velocity, ai);
                }
                AIBehavior::Chase => {
                    self.handle_chase(transform, velocity, ai, world);
                }
                AIBehavior::Attack => {
                    self.handle_attack(transform, world);
                }
                AIBehavior::Evade => {
                    self.handle_evade(transform, velocity, ai, world);
                }
            }
        }

        Ok(())
    }
}

impl AISystem {
    /// 处理巡逻行为
    fn handle_patrol(&self, transform: &mut Transform, velocity: &mut Velocity, ai: &mut AI) {
        if ai.patrol_path.is_empty() {
            return;
        }

        let target = ai.patrol_path[ai.current_path_index];
        let dx = target.0 - transform.x;
        let dy = target.1 - transform.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 5.0 {
            // 到达路径点，移动到下一个
            ai.current_path_index = (ai.current_path_index + 1) % ai.patrol_path.len();
        } else {
            // 向目标移动
            velocity.dx = dx / distance * ai.move_speed;
            velocity.dy = dy / distance * ai.move_speed;
        }
    }

    /// 处理追踪行为
    fn handle_chase(&self, transform: &mut Transform, velocity: &mut Velocity, ai: &mut AI, world: &World) {
        // 寻找玩家
        let mut player_pos = None;
        let mut player_query = world.query::<&Transform, &Player>();
        for (player_transform, _) in player_query.iter() {
            player_pos = Some((player_transform.x, player_transform.y));
            break;
        }

        if let Some((player_x, player_y)) = player_pos {
            let dx = player_x - transform.x;
            let dy = player_y - transform.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 5.0 {
                // 向玩家移动
                velocity.dx = dx / distance * ai.move_speed;
                velocity.dy = dy / distance * ai.move_speed;
            }
        }
    }

    /// 处理攻击行为
    fn handle_attack(&self, transform: &mut Transform, world: &mut World) {
        // 创建敌人子弹
        let bullet_entity = world.spawn();

        // 添加变换组件
        world.add_component(bullet_entity, Transform {
            x: transform.x,
            y: transform.y,
            rotation: 0.0,
            scale: 1.0,
        }).unwrap();

        // 添加速度组件
        world.add_component(bullet_entity, Velocity {
            dx: 0.0,
            dy: 2.0,
            ax: 0.0,
            ay: 0.0,
        }).unwrap();

        // 添加精灵组件
        world.add_component(bullet_entity, Sprite {
            path: "enemy_bullet.png".to_string(),
            width: 8.0,
            height: 16.0,
            visible: true,
        }).unwrap();

        // 添加碰撞体组件
        world.add_component(bullet_entity, Collider {
            collider_type: ColliderType::Circle { radius: 4.0 },
            layer: 4,
            mask: 2,
        }).unwrap();

        // 添加子弹组件
        world.add_component(bullet_entity, Bullet {
            bullet_type: "enemy".to_string(),
            damage: 1,
            shooter_type: ShooterType::Enemy,
        }).unwrap();
    }

    /// 处理躲避行为
    fn handle_evade(&self, transform: &mut Transform, velocity: &mut Velocity, ai: &mut AI, world: &World) {
        // 寻找玩家子弹
        let mut bullet_positions = vec![];
        let mut bullet_query = world.query::<&Transform, &Bullet>();
        for (bullet_transform, bullet) in bullet_query.iter() {
            if matches!(bullet.shooter_type, ShooterType::Player) {
                bullet_positions.push((bullet_transform.x, bullet_transform.y));
            }
        }

        // 避开子弹
        for (bullet_x, bullet_y) in bullet_positions {
            let dx = bullet_x - transform.x;
            let dy = bullet_y - transform.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 50.0 {
                // 远离子弹
                velocity.dx -= dx / distance * ai.move_speed;
                velocity.dy -= dy / distance * ai.move_speed;
            }
        }
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
    fn run(&mut self, world: &mut World) -> GResult<()> {
        // 这里应该实现渲染逻辑
        // 由于我们没有实际的渲染实现，这里只是一个占位符
        Ok(())
    }
}
