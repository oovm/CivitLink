# GG Platform

GG Platform is the base library for GG engine platform support.

## Features
- Cross-platform abstractions
- File system operations
- Input handling
- Window management
- Time and threading utilities

## Dependencies
- gg-core

## Usage

```rust
use gg_platform::*;

// Initialize platform
let platform = Platform::new();

// Access file system
let fs = platform.fs();

// Handle input
let input = platform.input();

// Manage window
let window = platform.window();
```
