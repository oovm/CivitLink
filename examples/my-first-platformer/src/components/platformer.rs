use gg_ecs::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
}

#[derive(Component)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Component)]
pub struct Rigidbody {
    pub velocity: Vector2,
    pub gravity: f32,
    pub mass: f32,
    pub is_grounded: bool,
}

#[derive(Component)]
pub struct Collider {
    pub width: f32,
    pub height: f32,
    pub is_static: bool,
}

#[derive(Component)]
pub struct Sprite {
    pub texture: String,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Component)]
pub struct Platform {
    pub is_moving: bool,
    pub move_speed: f32,
    pub start_x: f32,
    pub end_x: f32,
    pub direction: f32,
}

#[derive(Component)]
pub struct Collectible {
    pub value: i32,
    pub collected: bool,
}

#[derive(Component)]
pub struct Goal {
    pub reached: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector2 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}