# GG Galgame Schema

GG Galgame Schema defines the data structures and schema for the GG Game Engine's galgame functionality, providing a standardized way to represent galgame data.

## Features
- Galgame data structures
- Schema validation
- Serialization and deserialization
- Integration with GG Galgame Engine

## Usage
```rust
use gg_galgame_schema::{Character, Scene, Dialogue};

let character = Character {
    id: "character1".to_string(),
    name: "Alice".to_string(),
    sprites: vec!["path/to/sprite.png".to_string()],
};

let dialogue = Dialogue {
    character_id: "character1".to_string(),
    text: "Hello, world!".to_string(),
    emotions: vec!["happy".to_string()],
};

let scene = Scene {
    id: "scene1".to_string(),
    background: "path/to/background.png".to_string(),
    characters: vec![character],
    dialogues: vec![dialogue],
};
```

## Requirements
- Rust 1.60+
- serde
- serde_json
