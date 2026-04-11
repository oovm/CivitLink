use gg_ecs::prelude::*;
use super::super::components::platformer::*;

pub fn player_system(
    mut query: Query<(&mut Player, &mut Rigidbody, &mut Transform, &mut Sprite)>,
    input: Res<InputState>,
) {
    for (mut player, mut rigidbody, mut transform, mut sprite) in query.iter_mut() {
        // 处理水平移动
        let mut move_x = 0.0;
        if input.left_pressed {
            move_x = -player.speed;
            sprite.flip_x = true;
        } else if input.right_pressed {
            move_x = player.speed;
            sprite.flip_x = false;
        }
        
        rigidbody.velocity.x = move_x;
        
        // 处理跳跃
        if input.space_pressed && player.is_grounded {
            rigidbody.velocity.y = -player.jump_force;
            player.is_grounded = false;
        }
        
        // 更新玩家状态
        player.is_grounded = rigidbody.is_grounded;
    }
}

pub fn physics_system(
    mut query: Query<(&mut Rigidbody, &mut Transform, &Collider)>,
    delta_time: Res<DeltaTime>,
) {
    for (mut rigidbody, mut transform, collider) in query.iter_mut() {
        if collider.is_static {
            continue;
        }
        
        // 应用重力
        rigidbody.velocity.y += rigidbody.gravity * delta_time.0;
        
        // 更新位置
        transform.x += rigidbody.velocity.x * delta_time.0;
        transform.y += rigidbody.velocity.y * delta_time.0;
    }
}

pub fn collision_system(
    mut player_query: Query<(&mut Player, &mut Rigidbody, &mut Transform, &Collider), With<Player>>,
    mut collider_query: Query<(&Transform, &Collider, Option<&Platform>, Option<&Collectible>, Option<&Goal>)>,
) {
    for (mut player, mut rigidbody, mut transform, player_collider) in player_query.iter_mut() {
        rigidbody.is_grounded = false;
        
        for (other_transform, other_collider, platform, collectible, goal) in collider_query.iter() {
            // 检测碰撞
            if check_collision(
                transform.x, transform.y, player_collider.width, player_collider.height,
                other_transform.x, other_transform.y, other_collider.width, other_collider.height
            ) {
                // 处理与平台的碰撞
                if platform.is_some() && other_collider.is_static {
                    // 从上方碰撞平台
                    if rigidbody.velocity.y > 0.0 && transform.y + player_collider.height <= other_transform.y + 10.0 {
                        transform.y = other_transform.y - player_collider.height;
                        rigidbody.velocity.y = 0.0;
                        rigidbody.is_grounded = true;
                        player.is_grounded = true;
                    }
                }
                
                // 处理与收集品的碰撞
                if collectible.is_some() {
                    // 这里应该处理收集逻辑
                }
                
                // 处理与目标的碰撞
                if goal.is_some() {
                    // 这里应该处理到达目标的逻辑
                }
            }
        }
    }
}

pub fn platform_system(
    mut query: Query<(&mut Platform, &mut Transform)>,
    delta_time: Res<DeltaTime>,
) {
    for (mut platform, mut transform) in query.iter_mut() {
        if platform.is_moving {
            transform.x += platform.move_speed * platform.direction * delta_time.0;
            
            // 到达边界时改变方向
            if transform.x <= platform.start_x {
                transform.x = platform.start_x;
                platform.direction = 1.0;
            } else if transform.x >= platform.end_x {
                transform.x = platform.end_x;
                platform.direction = -1.0;
            }
        }
    }
}

fn check_collision(
    x1: f32, y1: f32, w1: f32, h1: f32,
    x2: f32, y2: f32, w2: f32, h2: f32,
) -> bool {
    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

#[derive(Resource)]
pub struct InputState {
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub space_pressed: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            left_pressed: false,
            right_pressed: false,
            space_pressed: false,
        }
    }
}

#[derive(Resource)]
pub struct DeltaTime(pub f32);

impl Default for DeltaTime {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}