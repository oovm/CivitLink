# GG Engine

GG Engine is the core engine implementation for the GG game engine framework.

## Features
- Game state management
- Configuration handling
- Character system integration
- Engine lifecycle management

## Dependencies
- gg-core
- gg-ecs
- gg-world
- gg-asset

## Usage

```rust
use gg_engine::*;

// Create engine instance
let mut engine = Engine::new();

// Load configuration
engine.load_config("game.toml");

// Run engine
engine.run();
```
