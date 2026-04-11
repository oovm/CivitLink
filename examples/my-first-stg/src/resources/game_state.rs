use gg_ecs::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct GameState {
    pub score: u32,
    pub lives: u32,
    pub level: u32,
    pub game_over: bool,
    pub wave: u32,
    pub enemies_spawned: u32,
    pub enemies_destroyed: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: 3,
            level: 1,
            game_over: false,
            wave: 1,
            enemies_spawned: 0,
            enemies_destroyed: 0,
        }
    }
}
