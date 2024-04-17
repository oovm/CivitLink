# *.material 文件格式规范

## 概述

*.material 文件是 GG 游戏引擎用于存储材质信息的文件格式，包含材质的属性、着色器、纹理等信息。材质文件用于定义游戏对象的视觉外观，是渲染系统的重要组成部分。

## 文件结构

一个完整的 *.material 文件是一个 RON 格式的文本文件，用于描述材质的属性和配置。

### 基本结构

```von
# 材质文件
MaterialFile {
    version: "1.0",
    material: Material {
        name: "StandardMaterial",
        description: "Standard PBR material",
        shader: "assets/shaders/standard.shader",
        type: "PBR",
        is_variant: false,
        material_base: null,
    },
    properties: Properties {
        albedo: TextureProperty {
            type: "texture",
            value: "assets/textures/albedo.png",
            tilling: [1, 1],
            offset: [0, 0],
        },
        normal: TextureProperty {
            type: "texture",
            value: "assets/textures/normal.png",
            tilling: [1, 1],
            offset: [0, 0],
        },
        metallic: FloatProperty {
            type: "float",
            value: 0.5,
        },
        roughness: FloatProperty {
            type: "float",
            value: 0.5,
        },
        specular: FloatProperty {
            type: "float",
            value: 0.5,
        },
        emissive: ColorProperty {
            type: "color",
            value: [0, 0, 0, 1],
        },
        opacity: FloatProperty {
            type: "float",
            value: 1.0,
        },
    },
    render_states: RenderStates {
        cull_mode: "back",
        blend_mode: "opaque",
        depth_test: true,
        depth_write: true,
        wireframe: false,
    },
    tags: Tags {
        render_queue: "opaque",
        light_mode: "standard",
    },
    dependencies: [
        Dependency {
            path: "assets/shaders/standard.shader",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        },
        Dependency {
            path: "assets/textures/albedo.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0002",
        },
        Dependency {
            path: "assets/textures/normal.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0003",
        },
    ],
    references: [
        Reference {
            path: "assets/meshes/ground.obj",
            field: "material",
        },
    ],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，材质格式版本号，用于向后兼容。

### 2. 材质信息

- **`material`**：对象，包含材质的基本信息。
  - **`name`**：字符串，材质名称。
  - **`description`**：字符串，材质描述。
  - **`shader`**：字符串，着色器文件的相对路径。
  - **`type`**：字符串，材质类型，如 `PBR`、`Unlit`、`Phong` 等。
  - **`is_variant`**：布尔值，是否为材质变体。
  - **`material_base`**：字符串或 null，如果是变体，则为基础材质的 guid；否则为 null。

### 3. 材质属性

- **`properties`**：对象，包含材质的属性。
  - 每个属性包含：
    - **`type`**：字符串，属性类型，如 `texture`、`float`、`color`、`vector` 等。
    - **`value`**：属性值，根据类型不同而不同。
    - **`tilling`**：数组，纹理平铺值 [x, y]（仅适用于纹理类型）。
    - **`offset`**：数组，纹理偏移值 [x, y]（仅适用于纹理类型）。

### 4. 渲染状态

- **`render_states`**：对象，包含材质的渲染状态。
  - **`cull_mode`**：字符串，剔除模式，如 `back`、`front`、`none`。
  - **`blend_mode`**：字符串，混合模式，如 `opaque`、`alpha`、`additive` 等。
  - **`depth_test`**：布尔值，是否启用深度测试。
  - **`depth_write`**：布尔值，是否启用深度写入。
  - **`wireframe`**：布尔值，是否以线框模式渲染。

### 5. 标签

- **`tags`**：对象，包含材质的标签。
  - **`render_queue`**：字符串，渲染队列，如 `opaque`、`transparent`、`overlay` 等。
  - **`light_mode`**：字符串，光照模式，如 `standard`、`unlit` 等。

### 6. 依赖关系

- **`dependencies`**：数组，包含该材质依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 7. 引用关系

- **`references`**：数组，包含引用该材质的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`field`**：字符串，引用该材质的字段名称。

### 8. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该材质文件的最后修改时间。
- **`hash`**：字符串，材质文件的 SHA1 哈希值，用于检测材质文件是否发生变化。

## 材质类型

### 1. PBR 材质

PBR (Physically Based Rendering) 材质是一种基于物理原理的材质类型，提供更真实的渲染效果。

```von
Material {
    type: "PBR",
    is_variant: false,
    material_base: null,
}
Properties {
    albedo: TextureProperty {
        type: "texture",
        value: "assets/textures/albedo.png",
    },
    normal: TextureProperty {
        type: "texture",
        value: "assets/textures/normal.png",
    },
    metallic: FloatProperty {
        type: "float",
        value: 0.5,
    },
    roughness: FloatProperty {
        type: "float",
        value: 0.5,
    },
    specular: FloatProperty {
        type: "float",
        value: 0.5,
    },
    emissive: ColorProperty {
        type: "color",
        value: [0, 0, 0, 1],
    },
}
```

### 2. Unlit 材质

Unlit 材质是一种不接受光照的材质类型，适用于UI元素、特效等。

```von
Material {
    type: "Unlit",
    is_variant: false,
    material_base: null,
}
Properties {
    color: ColorProperty {
        type: "color",
        value: [1, 1, 1, 1],
    },
    texture: TextureProperty {
        type: "texture",
        value: "assets/textures/ui.png",
    },
}
```

### 3. Phong 材质

Phong 材质是一种传统的光照模型材质，适用于一些风格化的游戏。

```von
Material {
    type: "Phong",
    is_variant: false,
    material_base: null,
}
Properties {
    diffuse: ColorProperty {
        type: "color",
        value: [0.8, 0.8, 0.8, 1],
    },
    specular: ColorProperty {
        type: "color",
        value: [1, 1, 1, 1],
    },
    shininess: FloatProperty {
        type: "float",
        value: 32,
    },
    emissive: ColorProperty {
        type: "color",
        value: [0, 0, 0, 1],
    },
}
```

## 使用场景

### 1. 游戏对象外观

- **角色材质**：定义游戏角色的外观，包括皮肤、服装等。
- **环境材质**：定义游戏环境的外观，包括地面、墙壁、天空等。
- **道具材质**：定义游戏道具的外观，包括武器、装备、物品等。

### 2. UI 元素

- **界面材质**：定义游戏界面的外观，包括按钮、面板、图标等。
- **特效材质**：定义游戏特效的外观，包括粒子、光效等。

### 3. 编辑器集成

- **材质编辑器**：在编辑器中可视化编辑材质的属性。
- **材质预览**：在编辑器中实时预览材质效果。
- **材质模板**：基于现有材质创建材质模板，快速生成新材质。

## 最佳实践

1. **材质组织**：将材质按照用途和类型组织到不同的文件夹中。
2. **纹理管理**：合理使用纹理，避免过大的纹理导致性能问题。
3. **材质复用**：尽量复用材质，减少材质数量。
4. **依赖管理**：定期检查并清理无效的依赖关系。
5. **版本控制**：将 .material 文件纳入版本控制系统，确保团队协作时的一致性。
6. **性能优化**：根据目标平台，合理设置材质的复杂度。
7. **哈希计算**：使用 SHA1 算法计算材质文件的哈希值。

## 示例完整文件

### PBR 材质示例

```von
# PBR 材质示例
MaterialFile {
    version: "1.0",
    material: Material {
        name: "CharacterMaterial",
        description: "PBR material for characters",
        shader: "assets/shaders/standard.shader",
        type: "PBR",
        is_variant: false,
        material_base: null,
    },
    properties: Properties {
        albedo: TextureProperty {
            type: "texture",
            value: "assets/textures/character_albedo.png",
            tilling: [1, 1],
            offset: [0, 0],
        },
        normal: TextureProperty {
            type: "texture",
            value: "assets/textures/character_normal.png",
            tilling: [1, 1],
            offset: [0, 0],
        },
        metallic: FloatProperty {
            type: "float",
            value: 0.2,
        },
        roughness: FloatProperty {
            type: "float",
            value: 0.8,
        },
        specular: FloatProperty {
            type: "float",
            value: 0.5,
        },
        emissive: ColorProperty {
            type: "color",
            value: [0, 0, 0, 1],
        },
        opacity: FloatProperty {
            type: "float",
            value: 1.0,
        },
    },
    render_states: RenderStates {
        cull_mode: "back",
        blend_mode: "opaque",
        depth_test: true,
        depth_write: true,
        wireframe: false,
    },
    tags: Tags {
        render_queue: "opaque",
        light_mode: "standard",
    },
    dependencies: [
        Dependency {
            path: "assets/shaders/standard.shader",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0005",
        },
        Dependency {
            path: "assets/textures/character_albedo.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0006",
        },
        Dependency {
            path: "assets/textures/character_normal.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0007",
        },
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

### UI 材质示例

```von
# UI 材质示例
MaterialFile {
    version: "1.0",
    material: Material {
        name: "UIMaterial",
        description: "Material for UI elements",
        shader: "assets/shaders/ui.shader",
        type: "Unlit",
        is_variant: false,
        material_base: null,
    },
    properties: Properties {
        color: ColorProperty {
            type: "color",
            value: [1, 1, 1, 1],
        },
        texture: TextureProperty {
            type: "texture",
            value: "assets/textures/ui_atlas.png",
            tilling: [1, 1],
            offset: [0, 0],
        },
    },
    render_states: RenderStates {
        cull_mode: "none",
        blend_mode: "alpha",
        depth_test: false,
        depth_write: false,
        wireframe: false,
    },
    tags: Tags {
        render_queue: "transparent",
        light_mode: "unlit",
    },
    dependencies: [
        Dependency {
            path: "assets/shaders/ui.shader",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0009",
        },
        Dependency {
            path: "assets/textures/ui_atlas.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0010",
        },
    ],
    references: [],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
}
```

### 材质变体示例

```von
# 材质变体示例
MaterialFile {
    version: "1.0",
    material: Material {
        name: "CharacterMaterial_Gold",
        description: "Gold variant of character material",
        shader: "assets/shaders/standard.shader",
        type: "PBR",
        is_variant: true,
        material_base: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0004",
    },
    properties: Properties {
        albedo: TextureProperty {
            type: "texture",
            value: "assets/textures/character_albedo_gold.png",
            tilling: [1, 1],
            offset: [0, 0],
        },
        metallic: FloatProperty {
            type: "float",
            value: 0.9,
        },
        roughness: FloatProperty {
            type: "float",
            value: 0.2,
        },
    },
    dependencies: [
        Dependency {
            path: "assets/textures/character_albedo_gold.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0012",
        },
    ],
    references: [],
    timestamp: "2026-04-10T13:00:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
}
```

## 总结

*.material 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理材质。通过存储材质的属性、着色器、纹理和渲染状态，它解决了游戏对象视觉外观的问题。

这种设计使得 GG 游戏引擎能够：
- 快速创建和管理材质
- 支持多种材质类型（PBR、Unlit、Phong 等）
- 通过变体机制实现材质的差异化
- 与纹理和着色器系统无缝集成
- 提供灵活的渲染状态配置
- 支持材质的版本控制和依赖管理

材质系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来定义和管理游戏对象的视觉外观。