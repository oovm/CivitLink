# GG Editor Render

GG Editor Render is the rendering module for the GG Game Engine's editor, providing rendering capabilities for the editor interface and scene preview.

## Features
- Editor-specific rendering
- Scene preview rendering
- UI rendering for editor
- Integration with GG Render

## Usage
```rust
use gg_editor_render::EditorRenderer;

let renderer = EditorRenderer::new();
renderer.render_scene(&scene);
renderer.render_ui(&ui_elements);
```

## Requirements
- Rust 1.60+
- GG Render
- winit
