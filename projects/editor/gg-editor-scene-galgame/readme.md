# GG Editor Scene Galgame

GG Editor Scene Galgame is the galgame scene editor module for the GG Game Engine, providing tools for creating and editing galgame scenes.

## Features
- Galgame scene editing
- Character and background management
- Dialogue and script editing
- Integration with GG Galgame Engine

## Usage
```rust
use gg_editor_scene_galgame::GalgameSceneEditor;

let editor = GalgameSceneEditor::new();
editor.create_scene("my_scene");
editor.add_character("character1", "path/to/sprite.png");
editor.add_dialogue("character1", "Hello, world!");
```

## Requirements
- Rust 1.60+
- GG Galgame Engine
- GG Editor Render
