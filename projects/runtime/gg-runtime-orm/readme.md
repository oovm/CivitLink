# GG Runtime ORM

GG Runtime ORM provides object-relational mapping functionality for the GG engine runtime.

## Features
- Entity mapping
- Query building
- Relationship management
- Migration support

## Dependencies
- gg-core
- gg-runtime-database

## Usage

```rust
use gg_runtime_orm::*;

// Define entity
#[derive(Entity)]
struct Player {
    id: i32,
    name: String,
    level: i32,
}

// Create repository
let repo = Repository::new(db);

// Create entity
let player = Player {
    id: 1,
    name: "Player1".to_string(),
    level: 1,
};

// Save entity
repo.save(&player);

// Find entity
let found = repo.find_by_id(1);

// Query entities
let players = repo.query().filter("level > ?", &[5]).all();
```
