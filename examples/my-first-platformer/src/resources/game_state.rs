use gg_ecs::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub score: i32,
    pub lives: i32,
    pub game_running: bool,
    pub level: i32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: 3,
            game_running: true,
            level: 1,
        }
    }
}

#[derive(Resource)]
pub struct LevelData {
    pub platforms: Vec<PlatformData>,
    pub collectibles: Vec<CollectibleData>,
    pub goal: GoalData,
}

pub struct PlatformData {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_moving: bool,
    pub move_speed: f32,
    pub start_x: f32,
    pub end_x: f32,
}

pub struct CollectibleData {
    pub x: f32,
    pub y: f32,
    pub value: i32,
}

pub struct GoalData {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}