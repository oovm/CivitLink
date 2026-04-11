# *.meta 文件格式规范

## 概述

*.meta 文件是 GG 游戏引擎用于存储资源元数据的文件格式，与资源文件一一对应，用于解决版本管理、资源引用和导入设置等问题。

## 文件结构

一个完整的 *.meta 文件是一个 TOML 格式的文本文件，与对应的资源文件同名，放在同一目录下，后缀为 `.meta`。

### 基本结构

```von
# 资源元数据文件
MetaFile {
    version: "1.0",
    asset: Asset {
        guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
        type: "Texture",
        import_settings: ImportSettings {
            compression: "high",
            max_size: 2048,
            format: "RGBA32",
        },
    },
    dependencies: [
        Dependency {
            path: "textures/common/background.jpg",
            guid: "a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6",
        },
    ],
    references: [
        Reference {
            path: "scenes/main.scene",
            entity_id: 123,
            component: "SpriteRenderer",
            field: "texture",
        },
    ],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，元数据格式版本号，用于向后兼容。

### 2. 资源信息

- **`asset`**：对象，包含资源的基本信息。
  - **`guid`**：字符串，资源的全局唯一标识符，用于在不同路径下识别同一资源。
  - **`type`**：字符串，资源类型，如 `Texture`、`Audio`、`Script` 等。
  - **`import_settings`**：对象，资源导入设置，根据资源类型不同而不同。

### 3. 依赖关系

- **`dependencies`**：数组，包含该资源依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 4. 引用关系

- **`references`**：数组，包含引用该资源的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`entity_id`**：数字，引用该资源的实体 ID（如果适用）。
    - **`component`**：字符串，引用该资源的组件类型（如果适用）。
    - **`field`**：字符串，引用该资源的字段名称（如果适用）。

### 5. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该元数据文件的最后修改时间。
- **`hash`**：字符串，资源文件的 SHA1 哈希值，用于检测资源文件是否发生变化。

## 资源类型与导入设置

### 纹理资源 (Texture)

```von
Asset {
    type: "Texture",
    import_settings: ImportSettings {
        compression: "high",
        max_size: 2048,
        format: "RGBA32",
        generate_mipmaps: true,
        wrap_mode: "clamp",
        filter_mode: "bilinear",
    },
}
```

### 音频资源 (Audio)

```von
Asset {
    type: "Audio",
    import_settings: ImportSettings {
        compression: "vorbis",
        quality: 0.8,
        loop: false,
        streaming: false,
    },
}
```

### 脚本资源 (Script)

```von
Asset {
    type: "Script",
    import_settings: ImportSettings {
        compile: true,
        optimize: "size",
        target: "web",
    },
}
```

### 场景资源 (Scene)

```von
Asset {
    type: "Scene",
    import_settings: ImportSettings {
        load_async: true,
        preload_resources: true,
    },
}
```

## 使用场景

### 1. 版本管理

- **资源唯一标识**：通过 `guid` 字段，即使资源路径发生变化，也能保持资源的唯一标识。
- **依赖关系跟踪**：通过 `dependencies` 字段，跟踪资源之间的依赖关系。
- **引用关系跟踪**：通过 `references` 字段，跟踪哪些资源引用了当前资源。
- **资源变更检测**：通过 `hash` 字段，检测资源文件是否发生变化。

### 2. 资源管理

- **导入设置**：存储资源的导入设置，确保资源在不同环境中保持一致的处理方式。
- **依赖解析**：在资源加载时，自动解析并加载依赖资源。
- **循环依赖检测**：通过依赖关系，检测并防止循环依赖。

### 3. 编辑器集成

- **资源浏览器**：在资源浏览器中显示资源的元数据信息。
- **引用查找**：快速查找哪些资源引用了当前资源。
- **依赖可视化**：可视化显示资源之间的依赖关系。

## 最佳实践

1. **版本控制**：将 .meta 文件纳入版本控制系统，确保团队协作时的一致性。
2. **路径管理**：使用相对路径，避免硬编码绝对路径。
3. **GUID 生成**：使用 UUID v7 格式生成 `guid`，以提高数据库索引性能。
4. **哈希计算**：使用 SHA1 算法计算资源文件的哈希值。
5. **元数据更新**：在资源文件发生变化时，自动更新 .meta 文件的 `hash` 和 `timestamp` 字段。
6. **依赖管理**：定期检查并清理无效的依赖关系。

## 示例完整文件

### 纹理资源的 .meta 文件

```von
# 纹理资源元数据
MetaFile {
    version: "1.0",
    asset: Asset {
        guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
        type: "Texture",
        import_settings: ImportSettings {
            compression: "high",
            max_size: 2048,
            format: "RGBA32",
            generate_mipmaps: true,
            wrap_mode: "clamp",
            filter_mode: "bilinear",
        },
    },
    dependencies: [],
    references: [
        Reference {
            path: "scenes/main.scene",
            entity_id: 123,
            component: "SpriteRenderer",
            field: "texture",
        },
        Reference {
            path: "scenes/menu.scene",
            entity_id: 456,
            component: "BackgroundRenderer",
            field: "texture",
        },
    ],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

### 场景资源的 .meta 文件

```von
# 场景资源元数据
MetaFile {
    version: "1.0",
    asset: Asset {
        guid: "a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6",
        type: "Scene",
        import_settings: ImportSettings {
            load_async: true,
            preload_resources: true,
        },
    },
    dependencies: [
        Dependency {
            path: "textures/background.jpg",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
        },
        Dependency {
            path: "audio/bgm.mp3",
            guid: "9a8b7c6d-5e4f-3g2h-1i0j-k9l8m7n6o5p4",
        },
    ],
    references: [
        Reference {
            path: "scripts/game_manager.v",
            entity_id: null,
            component: null,
            field: "start_scene",
        },
    ],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
}
```

## 总结

*.meta 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理资源的元数据信息。通过存储资源的唯一标识、导入设置、依赖关系和引用关系，它解决了版本管理、资源引用和导入设置等问题，同时为编辑器集成提供了必要的信息。

这种设计使得 GG 游戏引擎在没有 .meta 文件的情况下也能正常工作，但在有 .meta 文件的情况下能够提供更好的版本管理和资源管理体验。