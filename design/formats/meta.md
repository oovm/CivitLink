# *.meta 文件格式规范

## 概述

*.meta 文件是 GG 游戏引擎用于存储资源元数据的文件格式，与资源文件一一对应，用于解决版本管理、资源引用和导入设置等问题。

## 设计原则

**原始文件只存内容**：
- 文本格式的原始文件（如 `.locale`、`.scene`、`.prefab`）专注于存储内容数据
- 元信息（版本、统计、作者、依赖关系等）存储在对应的 `.meta` 文件中
- 非文本文件（如 `.png`、`.mp3`）不可直接修改，所有元信息都存 `.meta` 文件

## 文件结构

一个完整的 *.meta 文件是一个 VON 格式的文本文件，与对应的资源文件同名，放在同一目录下，后缀为 `.meta`。

### 基本结构

```von
# 通用 MetaFile 基类结构
MetaFile {
    version: "1.0",
    guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
    type: "Texture",
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    dependencies: [ ... ],
    references: [ ... ],
    extra: { ... },
}

# 实际使用时，不同类型有对应的 MetaFile 子类：
# TextureMetaFile, AudioMetaFile, SceneMetaFile, LocaleMetaFile 等
```

## 通用字段说明

| 字段 | 类型 | 必填 | 说明 |
|-----|------|------|------|
| version | string | 是 | 元数据格式版本号，用于向后兼容 |
| guid | string | 是 | 资源的全局唯一标识符（UUID v7） |
| type | string | 是 | 资源类型，如 `Texture`、`Audio`、`Locale` 等 |
| timestamp | string | 是 | ISO 格式的时间戳，表示最后修改时间 |
| hash | string | 是 | 资源文件的 SHA-256 哈希值（前 64 位） |
| dependencies | array | 否 | 该资源依赖的其他资源 |
| references | array | 否 | 引用该资源的其他资源 |
| extra | object | 否 | 资源类型特定的扩展字段 |

### Dependency 字段

| 字段 | 类型 | 说明 |
|-----|------|------|
| path | string | 依赖资源的相对路径 |
| guid | string | 依赖资源的全局唯一标识符 |

### Reference 字段

| 字段 | 类型 | 说明 |
|-----|------|------|
| path | string | 引用资源的相对路径 |
| entity_id | number | 引用该资源的实体 ID（如果适用） |
| component | string | 引用该资源的组件类型（如果适用） |
| field | string | 引用该资源的字段名称（如果适用） |

## 资源类型与 Meta 结构

不同类型的资源有不同的 `extra` 字段结构。

### 纹理资源 (Texture)

```von
TextureMetaFile {
    version: "1.0",
    guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
    type: "Texture",
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    dependencies: [],
    references: [
        Reference {
            path: "scenes/main.scene",
            entity_id: 123,
            component: "SpriteRenderer",
            field: "texture",
        },
    ],
    extra: TextureMeta {
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
AudioMetaFile {
    version: "1.0",
    guid: "9a8b7c6d-5e4f-3g2h-1i0j-k9l8m7n6o5p4",
    type: "Audio",
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
    dependencies: [],
    references: [],
    extra: AudioMeta {
        compression: "vorbis",
        quality: 0.8,
        loop: false,
        streaming: false,
    },
}
```

### 场景资源 (Scene)

```von
SceneMetaFile {
    version: "1.0",
    guid: "a1b2c3d4-5e6f-7g8h-9i0j-k1l2m3n4o5p6",
    type: "Scene",
    timestamp: "2026-04-10T13:00:00Z",
    hash: "b5d4c3e2f1a0b9c8d7e6f5a4b3c2d1e0",
    dependencies: [
        Dependency {
            path: "textures/background.jpg",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
        },
    ],
    references: [],
    extra: SceneMeta {
        load_async: true,
        preload_resources: true,
    },
}
```

### Locale 资源

```von
LocaleMetaFile {
    version: "1.0",
    guid: "b2c3d4e5-f6a7-8b9c-0d1e-2f3a4b5c6d7e",
    type: "Locale",
    timestamp: "2026-04-13T15:30:00Z",
    hash: "c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1",
    dependencies: [
        Dependency {
            path: "stories/main.story",
            guid: "d3e4f5a6-b7c8-9d0e-1f2a-3b4c5d6e7f8a",
        },
    ],
    references: [],
    extra: LocaleMeta {
        source_file: "main.story",
        source_locale: "zh-CN",
        supported_locales: ["zh-CN", "en-US", "ja-JP"],
        author: "翻译团队",
        created_at: "2026-04-13T12:00:00Z",
        translation_stats: {
            en-US: TranslationStats {
                total: 100,
                translated: 85,
                fuzzy: 5,
                untranslated: 10,
                progress: 85.0,
            },
            ja-JP: TranslationStats {
                total: 100,
                translated: 70,
                fuzzy: 10,
                untranslated: 20,
                progress: 70.0,
            },
        },
    },
}
```

### LocaleMeta 字段

| 字段 | 类型 | 必填 | 说明 |
|-----|------|------|------|
| source_file | string | 是 | 源剧本文件名 |
| source_locale | string | 是 | 源语言代码 |
| supported_locales | array | 是 | 支持的语言列表 |
| author | string | 否 | 翻译作者 |
| created_at | string | 否 | 创建时间（ISO 8601） |
| translation_stats | object | 否 | 各语言翻译统计 |

### TranslationStats 字段

| 字段 | 类型 | 说明 |
|-----|------|------|
| total | number | 总条目数 |
| translated | number | 已翻译数 |
| fuzzy | number | 模糊匹配数 |
| untranslated | number | 未翻译数 |
| progress | number | 翻译进度（百分比） |

### 脚本资源 (Script)

```von
ScriptMetaFile {
    version: "1.0",
    guid: "e5f6a7b8-c9d0-1e2f-3a4b-5c6d7e8f9a0b",
    type: "Script",
    timestamp: "2026-04-10T14:00:00Z",
    hash: "d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2",
    dependencies: [],
    references: [],
    extra: ScriptMeta {
        compile: true,
        optimize: "size",
        target: "web",
    },
}
```

### Prefab 资源

```von
PrefabMetaFile {
    version: "1.0",
    guid: "f6a7b8c9-d0e1-2f3a-4b5c-6d7e8f9a0b1c",
    type: "Prefab",
    timestamp: "2026-04-10T14:30:00Z",
    hash: "e8f7a6b5c4d3e2f1a0b9c8d7e6f5a4b3",
    dependencies: [
        Dependency {
            path: "textures/sprite.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000",
        },
    ],
    references: [],
    extra: PrefabMeta {
        category: "Characters",
        tags: ["player", "main"],
    },
}
```

## 资源类型汇总

| 类型 | 文件扩展名 | Meta 类型 | 主要 Extra 字段 |
|-----|-----------|----------|----------------|
| Texture | `.png`, `.jpg` | TextureMetaFile | compression, max_size, format |
| Audio | `.mp3`, `.ogg`, `.wav` | AudioMetaFile | compression, quality, loop |
| Scene | `.scene` | SceneMetaFile | load_async, preload_resources |
| Locale | `.locale` | LocaleMetaFile | source_locale, supported_locales, translation_stats |
| Script | `.v` (Vlang) | ScriptMetaFile | compile, optimize, target |
| Prefab | `.prefab` | PrefabMetaFile | category, tags |
| Material | `.material` | MaterialMetaFile | shader, render_queue |
| Animation | `.anim` | AnimationMetaFile | duration, loop |
| Config | `.config` | ConfigMetaFile | schema_version |

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

## 文件配对示例

```
assets/
├── textures/
│   ├── background.png
│   ├── background.png.meta
│   └── sprite.png
│       └── sprite.png.meta
├── scenes/
│   ├── main.scene
│   ├── main.scene.meta
│   └── menu.scene
│       └── menu.scene.meta
├── locales/
│   ├── main.locale
│   ├── main.locale.meta
│   └── chapter1.locale
│       └── chapter1.locale.meta
└── stories/
    ├── main.story
    └── main.story.meta
```

## 最佳实践

1. **版本控制**：将 .meta 文件纳入版本控制系统，确保团队协作时的一致性。
2. **路径管理**：使用相对路径，避免硬编码绝对路径。
3. **GUID 生成**：使用 UUID v7 格式生成 `guid`，以提高数据库索引性能。
4. **哈希计算**：使用 SHA-256 算法计算资源文件的哈希值。
5. **元数据更新**：在资源文件发生变化时，自动更新 .meta 文件的 `hash` 和 `timestamp` 字段。
6. **依赖管理**：定期检查并清理无效的依赖关系。
7. **内容与元信息分离**：原始文件专注内容，元信息存 .meta 文件。

## 总结

*.meta 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理资源的元数据信息。通过存储资源的唯一标识、导入设置、依赖关系和引用关系，它解决了版本管理、资源引用和导入设置等问题。

**核心设计原则**：原始文件只存内容（或为非文本不可修改文件），元信息存 `.meta` 文件。这种设计使得：
- 内容变更与元信息变更分离，便于版本控制
- 大模型处理时专注内容，不受元信息干扰
- 引擎在没有 .meta 文件的情况下也能正常工作，但在有 .meta 文件时提供更好的管理体验
