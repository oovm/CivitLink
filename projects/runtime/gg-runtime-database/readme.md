# GG Runtime Database

GG Runtime Database provides database functionality for the GG engine runtime.

## Features
- SQLite integration
- Database schema management
- Query building
- Transaction support

## Dependencies
- gg-core
- rusqlite

## Usage

```rust
use gg_runtime_database::*;

// Create database connection
let db = Database::open("game.db");

// Execute query
let result = db.execute("SELECT * FROM players");

// Use transactions
let tx = db.transaction();
tx.execute("INSERT INTO players (name) VALUES (?)", &["Player1"]);
tx.commit();
```
