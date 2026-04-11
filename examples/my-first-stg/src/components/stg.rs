use gg_ecs::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Player {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub health: u32,
    pub speed: f32,
    pub fire_rate: f32,
    pub last_fire_time: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Enemy {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub health: u32,
    pub damage: u32,
    pub speed: f32,
    pub pattern: EnemyPattern,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Projectile {
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub damage: u32,
    pub owner: ProjectileOwner,
    pub lifetime: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Collider {
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyPattern {
    Straight,
    ZigZag,
    Circular,
    Boss,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectileOwner {
    Player,
    Enemy,
}
