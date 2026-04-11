use gg_ecs::prelude::*;

#[derive(Component)]
pub struct Dialogue {
    pub text: String,
    pub speaker: String,
    pub portrait: Option<String>,
    pub expression: Option<String>,
    pub position: Option<String>,
}

#[derive(Component)]
pub struct Choice {
    pub options: Vec<(String, String)>,
    pub current_index: usize,
}

#[derive(Component)]
pub struct Character {
    pub name: String,
    pub portraits: Vec<String>,
    pub current_expression: String,
}

#[derive(Component)]
pub struct Scene {
    pub name: String,
    pub background: String,
}

#[derive(Component)]
pub struct DialogueNode {
    pub id: String,
    pub next: Option<String>,
}