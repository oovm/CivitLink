# GG Runtime Cache

GG Runtime Cache provides caching functionality for the GG engine runtime.

## Features
- Memory caching
- Disk caching
- Cache invalidation
- Cache management utilities

## Dependencies
- gg-core

## Usage

```rust
use gg_runtime_cache::*;

// Create cache instance
let mut cache = Cache::new();

// Store data in cache
cache.set("key", data);

// Retrieve data from cache
let data = cache.get("key");

// Invalidate cache
cache.invalidate("key");
```
