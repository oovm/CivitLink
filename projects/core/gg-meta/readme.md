# gg-meta

## Overview

`gg-meta` is a metadata management module for the GG Game Engine. It provides functionality to create, read, and write metadata files for game assets using Oak-von format.

## Features

- Create metadata files with unique GUIDs (UUID v7)
- Read metadata from Oak-von files
- Write metadata to Oak-von files
- Manage dependencies and references between assets
- Track asset modifications with timestamps
- Generate unique GUIDs for assets

## Usage

### Creating a new metadata file

```rust
use gg_meta::{MetaFile, generate_guid};
use std::path::Path;

// Create a new metadata file for a texture asset
let mut meta = MetaFile::new(
    "texture",
    "assets/textures/player.png",
    "Player Texture",
    1024 * 1024 * 4, // 4MB
);

// Add dependencies
meta.add_dependency("assets/textures/player_normal.png", &generate_guid());

// Add references
meta.add_reference("assets/prefabs/player.prefab", Some("texture"));

// Update timestamp
meta.update_timestamp();

// Write to file
meta.to_file(Path::new("assets/textures/player.png.meta")).unwrap();
```

### Reading a metadata file

```rust
use gg_meta::MetaFile;
use std::path::Path;

// Read metadata from file
let meta = MetaFile::from_file(Path::new("assets/textures/player.png.meta")).unwrap();

// Access metadata fields
println!("Asset GUID: {}", meta.asset.guid);
println!("Asset type: {}", meta.asset.r#type);
println!("Asset path: {}", meta.asset.path);
```

## File Structure

The metadata file is stored in Oak-von format and contains the following fields:

- `version`: Metadata format version
- `asset`: Asset information (type, path, GUID, name, size, modified timestamp)
- `import_settings`: Optional import settings for the asset
- `dependencies`: List of dependencies for the asset
- `references`: List of references to the asset
- `timestamp`: Metadata file timestamp
- `hash`: Asset file hash (not yet implemented)

## Dependencies

- `serde`: For serialization and deserialization
- `oak-von`: For Oak-von format support
- `oak-core`: For Oak core functionality
- `uuid`: For generating UUID v7 GUIDs
- `chrono`: For timestamp generation
- `serde_json`: For import settings

## License

MIT
