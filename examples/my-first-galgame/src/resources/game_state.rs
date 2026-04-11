use gg_ecs::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub current_node: String,
    pub variables: std::collections::HashMap<String, i32>,
    pub is_dialogue_active: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_node: "start".to_string(),
            variables: std::collections::HashMap::new(),
            is_dialogue_active: true,
        }
    }
}

#[derive(Resource)]
pub struct DialogueData {
    pub nodes: std::collections::HashMap<String, DialogueNodeData>,
}

pub struct DialogueNodeData {
    pub text: String,
    pub speaker: String,
    pub portrait: Option<String>,
    pub expression: Option<String>,
    pub position: Option<String>,
    pub choices: Vec<(String, String)>,
    pub next: Option<String>,
}