//! Platformer 引擎系统定义
//! 定义游戏中使用的各种 ECS 系统

use crate::components::*;
use gg_core::GResult;
use gg_ecs::{Query, System, World};

/// 输入系统
/// 处理玩家输入
pub struct InputSystem {
    /// 移动速度
    move_speed: f32,
    /// 跳跃力度
    jump_force: f32,
    /// 左方向键是否按下
    left_pressed: bool,
    /// 右方向键是否按下
    right_pressed: bool,
    /// 跳跃键是否按下
    jump_pressed: bool,
}

impl InputSystem {
    /// 创建新的输入系统
    pub fn new(move_speed: f32, jump_force: f32) -> Self {
        Self {
            move_speed,
            jump_force,
            left_pressed: false,
            right_pressed: false,
            jump_pressed: false,
        }
    }

    /// 设置按键状态
    pub fn set_key_state(&mut self, key: &str, pressed: bool) {
        match key {
            "Left" => self.left_pressed = pressed,
            "Right" => self.right_pressed = pressed,
            "Space" => self.jump_pressed = pressed,
            _ => {},
        }
    }
}

impl System for InputSystem {
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Velocity, &mut Player, &PhysicsBody)>();

        for (velocity, player, physics_body) in query.iter_mut() {
            // 处理水平移动
            if self.left_pressed {
                velocity.dx = -self.move_speed;
            } else if self.right_pressed {
                velocity.dx = self.move_speed;
            } else {
                velocity.dx = 0.0;
            }

            // 处理跳跃
            if self.jump_pressed && player.can_jump && physics_body.on_ground {
                velocity.dy = -self.jump_force;
                player.jump_count = 1;
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
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Velocity, &mut PhysicsBody)>();

        for (velocity, physics_body) in query.iter_mut() {
            if physics_body.affected_by_gravity && !physics_body.is_static {
                // 应用重力
                velocity.dy += self.gravity * physics_body.gravity_scale;

                // 限制最大下落速度
                if velocity.dy > self.max_fall_speed {
                    velocity.dy = self.max_fall_speed;
                }
            }

            // 应用摩擦力
            if physics_body.on_ground {
                velocity.dx *= self.ground_friction;
            } else {
                velocity.dx *= self.air_friction;
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
        let mut query = world.query::<(&mut Transform, &Velocity, &PhysicsBody)>();

        for (transform, velocity, physics_body) in query.iter_mut() {
            if !physics_body.is_static {
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

        // 重置地面状态
        let mut physics_query = world.query::<&mut PhysicsBody>();
        for physics_body in physics_query.iter_mut() {
            physics_body.on_ground = false;
        }

        // 使用空间分区优化碰撞检测
        // 这里使用简单的网格分区，实际项目中可以使用更复杂的空间分区算法
        let mut grid: std::collections::HashMap<(i32, i32), Vec<u32>> = std::collections::HashMap::new();
        let cell_size = 64.0; // 网格单元格大小

        // 第一步：将实体分配到网格中
        for &entity in &entities {
            if let Some(transform) = world.get_component::<Transform>(entity) {
                if world.has_component::<Collider>(entity) {
                    let cell_x = (transform.x / cell_size) as i32;
                    let cell_y = (transform.y / cell_size) as i32;
                    grid.entry((cell_x, cell_y)).or_default().push(entity);
                }
            }
        }

        // 第二步：只检查同一网格和相邻网格中的实体
        for ((cell_x, cell_y), cell_entities) in &grid {
            // 检查当前网格中的实体对
            for i in 0..cell_entities.len() {
                for j in i + 1..cell_entities.len() {
                    let entity1 = cell_entities[i];
                    let entity2 = cell_entities[j];
                    self.check_and_handle_collision(world, entity1, entity2);
                }
            }

            // 检查与相邻网格的实体
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue; // 跳过当前网格，已经检查过了
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
    fn check_and_handle_collision(&self, world: &mut World, entity1: u32, entity2: u32) {
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

    /// 处理碰撞响应
    fn handle_collision(&self, world: &mut World, entity1: u32, entity2: u32) {
        // 检查是否是玩家与平台的碰撞
        if world.has_component::<Player>(entity1) && world.has_component::<Platform>(entity2) {
            // 处理玩家与平台的碰撞
            if let (Some(mut transform1), Some(mut velocity1), Some(mut physics_body1), Some(transform2), Some(collider2)) = (
                world.get_component_mut::<Transform>(entity1),
                world.get_component_mut::<Velocity>(entity1),
                world.get_component_mut::<PhysicsBody>(entity1),
                world.get_component::<Transform>(entity2),
                world.get_component::<Collider>(entity2),
            ) {
                // 检查玩家是否在平台上方
                if transform1.y < transform2.y {
                    // 标记玩家在地面上
                    physics_body1.on_ground = true;
                    physics_body1.ground_normal = (0.0, 1.0);

                    // 调整玩家位置
                    if let ColliderType::Rectangle { height: h2, .. } = collider2.collider_type {
                        transform1.y = transform2.y - h2 / 2.0 - 16.0; // 16.0 是玩家高度的一半
                    }

                    // 重置垂直速度
                    velocity1.dy = 0.0;
                }
            }
        }

        // 检查是否是玩家与可收集物品的碰撞
        if world.has_component::<Player>(entity1) && world.has_component::<Collectible>(entity2) {
            // 处理玩家收集物品
            if let Some(mut collectible) = world.get_component_mut::<Collectible>(entity2) {
                if !collectible.collected {
                    collectible.collected = true;
                    // 这里可以添加收集物品的逻辑，如增加分数等
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
}

impl CollectibleSystem {
    /// 创建新的收集系统
    pub fn new() -> Self {
        Self {
            score: 0,
        }
    }
}

impl System for CollectibleSystem {
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Collectible, &Transform)>();

        for (collectible, transform) in query.iter_mut() {
            if collectible.collected {
                // 增加分数
                self.score += collectible.value;
                // 这里可以添加其他收集物品的逻辑
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
    fn run(&mut self, world: &mut World) -> GResult<()> {
        let mut query = world.query::<(&mut Transform, &mut Velocity, &mut AI)>();

        for (transform, velocity, ai) in query.iter_mut() {
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
        // 这里可以实现攻击逻辑
        // 例如创建攻击实体或发射子弹
    }

    /// 处理躲避行为
    fn handle_evade(&self, transform: &mut Transform, velocity: &mut Velocity, ai: &mut AI, world: &World) {
        // 这里可以实现躲避逻辑
        // 例如检测玩家位置并远离
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
