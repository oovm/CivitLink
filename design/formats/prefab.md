# *.prefab 文件格式规范

## 概述

*.prefab 文件是 GG 游戏引擎用于存储可重用游戏对象的文件格式，包含游戏对象的层次结构、组件和属性信息。Prefab 可以作为基础模板创建游戏对象，也可以通过变体（Variant）机制创建基于基础 Prefab 的差异化版本。

## 文件结构

一个完整的 *.prefab 文件是一个 TOML 格式的文本文件，用于描述游戏对象的结构和属性。

### 基本结构

```ron
// 预制体文件
PrefabFile({
    version: "1.0",
    prefab: Prefab({
        name: "PlayerPrefab",
        description: "Player character prefab",
        is_variant: false,
        base_prefab: None,
    }),
    entities: [
        Entity({
            id: 1,
            name: "Player",
            parent_id: None,
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                }),
                Component({
                    type: "SpriteRenderer",
                    properties: SpriteRendererProperties({
                        sprite: "assets/textures/player.png",
                        sort_order: 0,
                        flip_x: false,
                        flip_y: false,
                    }),
                }),
                Component({
                    type: "PlayerController",
                    properties: PlayerControllerProperties({
                        speed: 5.0,
                        jump_force: 10.0,
                        health: 100,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "Camera",
            parent_id: Some(1),
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [0, 0, -10],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                }),
                Component({
                    type: "Camera",
                    properties: CameraProperties({
                        field_of_view: 60,
                        near_plane: 0.1,
                        far_plane: 1000,
                        clear_color: [0, 0, 0, 1],
                    }),
                }),
            ],
        }),
    ],
    variants: [],
    dependencies: [
        Dependency({
            path: "assets/textures/player.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        }),
    ],
    references: [
        Reference({
            path: "scenes/main.scene",
            entity_id: Some(123),
            component: Some("Spawner"),
            field: Some("prefab"),
        }),
    ],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
})
```

### Prefab Variant 结构

```ron
// 预制体变体文件
PrefabFile({
    version: "1.0",
    prefab: Prefab({
        name: "PlayerPrefab_Armed",
        description: "Player character with weapon",
        is_variant: true,
        base_prefab: Some("018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000"),
    }),
    entities: [
        Entity({
            id: 1,
            name: "Player",
            parent_id: None,
            components: [
                Component({
                    type: "PlayerController",
                    properties: PlayerControllerProperties({
                        speed: 6.0,
                        health: 120,
                    }),
                }),
            ],
        }),
        Entity({
            id: 3,
            name: "Weapon",
            parent_id: Some(1),
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [1, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                }),
                Component({
                    type: "SpriteRenderer",
                    properties: SpriteRendererProperties({
                        sprite: "assets/textures/weapon.png",
                    }),
                }),
                Component({
                    type: "Weapon",
                    properties: WeaponProperties({
                        damage: 20,
                        range: 5,
                    }),
                }),
            ],
        }),
    ],
    variants: [],
    dependencies: [
        Dependency({
            path: "assets/textures/weapon.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0003",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
})
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，预制体格式版本号，用于向后兼容。

### 2. 预制体信息

- **`prefab`**：对象，包含预制体的基本信息。
  - **`name`**：字符串，预制体名称。
  - **`description`**：字符串，预制体描述。
  - **`is_variant`**：布尔值，是否为预制体变体。
  - **`base_prefab`**：字符串或 null，如果是变体，则为基础预制体的 guid；否则为 null。

### 3. 实体列表

- **`entities`**：数组，包含预制体中的所有实体。
  - 每个实体包含：
    - **`id`**：数字，实体 ID，在预制体内唯一。
    - **`name`**：字符串，实体名称。
    - **`parent_id`**：数字或 null，父实体 ID，如果是根实体则为 null。
    - **`components`**：数组，实体的组件列表。
      - 每个组件包含：
        - **`type`**：字符串，组件类型。
        - **`properties`**：对象，组件属性，根据组件类型不同而不同。

### 4. 变体列表

- **`variants`**：数组，包含基于此预制体创建的变体。
  - 每个变体包含：
    - **`guid`**：字符串，变体的全局唯一标识符。
    - **`name`**：字符串，变体名称。

### 5. 依赖关系

- **`dependencies`**：数组，包含该预制体依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 6. 引用关系

- **`references`**：数组，包含引用该预制体的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`entity_id`**：数字，引用该预制体的实体 ID（如果适用）。
    - **`component`**：字符串，引用该预制体的组件类型（如果适用）。
    - **`field`**：字符串，引用该预制体的字段名称（如果适用）。

### 7. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该预制体文件的最后修改时间。
- **`hash`**：字符串，预制体文件的 SHA1 哈希值，用于检测预制体文件是否发生变化。

## Prefab Variant 机制

### 1. 基础预制体与变体的关系

- **基础预制体**：原始的预制体模板，包含完整的实体和组件信息。
- **变体**：基于基础预制体创建的差异化版本，可以覆盖基础预制体的部分属性或添加新的实体/组件。

### 2. 变体的继承规则

1. **属性继承**：变体继承基础预制体的所有属性，除非明确覆盖。
2. **组件继承**：变体继承基础预制体的所有组件，除非明确覆盖或移除。
3. **实体继承**：变体继承基础预制体的所有实体，除非明确添加新实体或修改现有实体。
4. **结构继承**：变体继承基础预制体的实体层次结构。

### 3. 变体的修改规则

1. **添加**：变体可以添加新的实体或组件。
2. **修改**：变体可以修改继承自基础预制体的组件属性。
3. **移除**：变体可以移除继承自基础预制体的组件（但不能移除实体）。

### 4. 变体的解析过程

1. 加载基础预制体。
2. 加载变体文件。
3. 合并基础预制体和变体的实体和组件。
4. 应用变体的修改。
5. 生成最终的预制体实例。

## 组件类型与属性示例

### Transform 组件

```ron
Component({
    type: "Transform",
    properties: TransformProperties({
        position: [0, 0, 0],
        rotation: [0, 0, 0],
        scale: [1, 1, 1],
    }),
})
```

### SpriteRenderer 组件

```ron
Component({
    type: "SpriteRenderer",
    properties: SpriteRendererProperties({
        sprite: "assets/textures/player.png",
        sort_order: 0,
        flip_x: false,
        flip_y: false,
        color: [1, 1, 1, 1],
    }),
})
```

### Camera 组件

```ron
Component({
    type: "Camera",
    properties: CameraProperties({
        field_of_view: 60,
        near_plane: 0.1,
        far_plane: 1000,
        clear_color: [0, 0, 0, 1],
        orthographic: false,
        size: 5,
    }),
})
```

### Collider 组件

```ron
Component({
    type: "Collider",
    properties: ColliderProperties({
        shape: "box",
        size: [1, 1, 1],
        offset: [0, 0, 0],
        is_trigger: false,
        layer: "default",
    }),
})
```

### Rigidbody 组件

```ron
Component({
    type: "Rigidbody",
    properties: RigidbodyProperties({
        mass: 1.0,
        is_kinematic: false,
        use_gravity: true,
        linear_velocity: [0, 0, 0],
        angular_velocity: [0, 0, 0],
    }),
})
```

## 使用场景

### 1. 游戏对象模板

- **角色模板**：创建玩家、敌人、NPC 等角色的基础模板。
- **道具模板**：创建武器、装备、道具等物品的基础模板。
- **环境模板**：创建建筑、地形、装饰等环境元素的基础模板。

### 2. 变体系统

- **角色变体**：基于基础角色模板创建不同职业、等级的角色变体。
- **敌人变体**：基于基础敌人模板创建不同难度、类型的敌人变体。
- **道具变体**：基于基础道具模板创建不同品质、等级的道具变体。

### 3. 场景管理

- **场景对象**：在场景中实例化预制体，快速构建游戏世界。
- **动态生成**：通过代码动态生成预制体实例，实现 procedural content generation。

## 最佳实践

1. **层次结构**：保持预制体的层次结构清晰，避免过深的嵌套。
2. **组件组织**：将相关的组件组织到同一实体中，提高代码可读性。
3. **变体使用**：合理使用变体机制，避免创建过多的基础预制体。
4. **依赖管理**：定期检查并清理无效的依赖关系。
5. **版本控制**：将 .prefab 文件纳入版本控制系统，确保团队协作时的一致性。
6. **哈希计算**：使用 SHA1 算法计算预制体文件的哈希值。
7. **元数据更新**：在预制体文件发生变化时，自动更新 `hash` 和 `timestamp` 字段。

## 示例完整文件

### 基础预制体示例

```ron
// 基础预制体示例
PrefabFile({
    version: "1.0",
    prefab: Prefab({
        name: "EnemyPrefab",
        description: "Base enemy prefab",
        is_variant: false,
        base_prefab: None,
    }),
    entities: [
        Entity({
            id: 1,
            name: "Enemy",
            parent_id: None,
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    }),
                }),
                Component({
                    type: "SpriteRenderer",
                    properties: SpriteRendererProperties({
                        sprite: "assets/textures/enemy.png",
                        sort_order: 0,
                    }),
                }),
                Component({
                    type: "EnemyAI",
                    properties: EnemyAIProperties({
                        health: 50,
                        speed: 2.0,
                        damage: 10,
                    }),
                }),
                Component({
                    type: "Collider",
                    properties: ColliderProperties({
                        shape: "box",
                        size: [1, 1, 1],
                    }),
                }),
                Component({
                    type: "Rigidbody",
                    properties: RigidbodyProperties({
                        mass: 1.0,
                        use_gravity: true,
                    }),
                }),
            ],
        }),
    ],
    variants: [],
    dependencies: [
        Dependency({
            path: "assets/textures/enemy.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0005",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
})
```

### 预制体变体示例

```ron
// 预制体变体示例
PrefabFile({
    version: "1.0",
    prefab: Prefab({
        name: "EnemyPrefab_Boss",
        description: "Boss enemy variant",
        is_variant: true,
        base_prefab: Some("018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0004"),
    }),
    entities: [
        Entity({
            id: 1,
            name: "Enemy",
            parent_id: None,
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        scale: [2, 2, 1],
                    }),
                }),
                Component({
                    type: "SpriteRenderer",
                    properties: SpriteRendererProperties({
                        sprite: "assets/textures/boss.png",
                    }),
                }),
                Component({
                    type: "EnemyAI",
                    properties: EnemyAIProperties({
                        health: 200,
                        speed: 1.5,
                        damage: 25,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "BossAura",
            parent_id: Some(1),
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [0, 0, -0.1],
                        scale: [2.5, 2.5, 1],
                    }),
                }),
                Component({
                    type: "SpriteRenderer",
                    properties: SpriteRendererProperties({
                        sprite: "assets/textures/boss_aura.png",
                        sort_order: -1,
                    }),
                }),
            ],
        }),
    ],
    variants: [],
    dependencies: [
        Dependency({
            path: "assets/textures/boss.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0007",
        }),
        Dependency({
            path: "assets/textures/boss_aura.png",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0008",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
})
```

## 总结

*.prefab 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理可重用游戏对象。通过存储游戏对象的层次结构、组件和属性，以及支持变体机制，它解决了游戏对象模板管理和差异化版本创建的问题。

这种设计使得 GG 游戏引擎能够：
- 快速创建和管理游戏对象模板
- 通过变体机制实现游戏对象的差异化
- 保持游戏对象的一致性和可维护性
- 支持复杂的游戏对象层次结构
- 与资源管理系统无缝集成

Prefab 系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来构建和管理游戏世界中的对象。