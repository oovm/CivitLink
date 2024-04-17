# *.scene 文件格式规范

## 概述

*.scene 文件是 GG 游戏引擎用于存储游戏场景的文件格式，包含场景的结构、实体布局、环境设置等信息。场景文件是游戏世界的基础构建块，用于组织和管理游戏中的各种元素。

## 文件结构

一个完整的 *.scene 文件是一个 TOML 格式的文本文件，用于描述场景的结构和属性。

### 基本结构

```ron
// 场景文件
SceneFile({
    version: "1.0",
    scene: Scene({
        name: "MainScene",
        description: "Main game scene",
        author: "Game Developer",
        created_at: "2026-04-10T12:00:00Z",
        last_modified: "2026-04-10T12:30:00Z",
    }),
    environment: Environment({
        ambient_light: [0.5, 0.5, 0.5, 1.0],
        fog: Fog({
            enabled: true,
            color: [0.1, 0.1, 0.1, 1.0],
            near: 10.0,
            far: 100.0,
            density: 0.01,
        }),
        gravity: [0, -9.81, 0],
        time_of_day: 12.0,
        weather: "clear",
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
                    type: "PlayerController",
                    properties: PlayerControllerProperties({
                        speed: 5.0,
                        jump_force: 10.0,
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
                    }),
                }),
            ],
        }),
    ],
    prefabs: [
        PrefabInstance({
            path: "assets/prefabs/enemy.prefab",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
            instances: [
                PrefabInstanceData({
                    id: 3,
                    name: "Enemy1",
                    position: [5, 0, 0],
                    rotation: [0, 0, 0],
                    scale: [1, 1, 1],
                }),
                PrefabInstanceData({
                    id: 4,
                    name: "Enemy2",
                    position: [-5, 0, 0],
                    rotation: [0, 0, 0],
                    scale: [1, 1, 1],
                }),
            ],
        }),
    ],
    scripts: [
        Script({
            path: "assets/scripts/scene_manager.gscript",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0002",
            enabled: true,
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/prefabs/enemy.prefab",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0001",
        }),
        Dependency({
            path: "assets/scripts/scene_manager.gscript",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0002",
        }),
    ],
    references: [
        Reference({
            path: "assets/scripts/game_manager.gscript",
            field: Some("current_scene"),
        }),
    ],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
})
```

## 字段详细说明

### 1. 版本信息

- **`version`**：字符串，场景格式版本号，用于向后兼容。

### 2. 场景信息

- **`scene`**：对象，包含场景的基本信息。
  - **`name`**：字符串，场景名称。
  - **`description`**：字符串，场景描述。
  - **`author`**：字符串，场景作者。
  - **`created_at`**：字符串，ISO 格式的时间戳，表示场景的创建时间。
  - **`last_modified`**：字符串，ISO 格式的时间戳，表示场景的最后修改时间。

### 3. 环境设置

- **`environment`**：对象，包含场景的环境设置。
  - **`ambient_light`**：数组，环境光颜色 [r, g, b, a]。
  - **`fog`**：对象，雾效设置。
    - **`enabled`**：布尔值，是否启用雾效。
    - **`color`**：数组，雾效颜色 [r, g, b, a]。
    - **`near`**：数字，雾效近裁剪面。
    - **`far`**：数字，雾效远裁剪面。
    - **`density`**：数字，雾效密度。
  - **`gravity`**：数组，重力向量 [x, y, z]。
  - **`time_of_day`**：数字，一天中的时间（0-24）。
  - **`weather`**：字符串，天气类型。

### 4. 实体列表

- **`entities`**：数组，包含场景中的所有实体。
  - 每个实体包含：
    - **`id`**：数字，实体 ID，在场景内唯一。
    - **`name`**：字符串，实体名称。
    - **`parent_id`**：数字或 null，父实体 ID，如果是根实体则为 null。
    - **`components`**：数组，实体的组件列表。
      - 每个组件包含：
        - **`type`**：字符串，组件类型。
        - **`properties`**：对象，组件属性，根据组件类型不同而不同。

### 5. 预制体实例

- **`prefabs`**：数组，包含场景中使用的预制体及其实例。
  - 每个预制体包含：
    - **`path`**：字符串，预制体文件的相对路径。
    - **`guid`**：字符串，预制体的全局唯一标识符。
    - **`instances`**：数组，预制体的实例列表。
      - 每个实例包含：
        - **`id`**：数字，实例 ID，在场景内唯一。
        - **`name`**：字符串，实例名称。
        - **`position`**：数组，位置 [x, y, z]。
        - **`rotation`**：数组，旋转 [x, y, z]。
        - **`scale`**：数组，缩放 [x, y, z]。

### 6. 脚本列表

- **`scripts`**：数组，包含场景中使用的脚本。
  - 每个脚本包含：
    - **`path`**：字符串，脚本文件的相对路径。
    - **`guid`**：字符串，脚本的全局唯一标识符。
    - **`enabled`**：布尔值，是否启用脚本。

### 7. 依赖关系

- **`dependencies`**：数组，包含该场景依赖的其他资源。
  - 每个依赖项包含：
    - **`path`**：字符串，依赖资源的相对路径。
    - **`guid`**：字符串，依赖资源的全局唯一标识符。

### 8. 引用关系

- **`references`**：数组，包含引用该场景的其他资源。
  - 每个引用项包含：
    - **`path`**：字符串，引用资源的相对路径。
    - **`field`**：字符串，引用该场景的字段名称。

### 9. 元数据

- **`timestamp`**：字符串，ISO 格式的时间戳，表示该场景文件的最后修改时间。
- **`hash`**：字符串，场景文件的 SHA1 哈希值，用于检测场景文件是否发生变化。

## 使用场景

### 1. 游戏世界构建

- **主场景**：游戏的主要场景，包含玩家、环境和游戏逻辑。
- **关卡场景**：游戏中的各个关卡，包含不同的环境和挑战。
- **菜单场景**：游戏的菜单界面，包含开始、设置、退出等功能。
- **加载场景**：游戏的加载界面，显示加载进度。

### 2. 场景管理

- **场景切换**：在不同场景之间切换，如从菜单场景切换到游戏场景。
- **场景叠加**：在现有场景上叠加新的场景，如弹出对话框。
- **场景流**：定义场景之间的切换流程，如剧情发展。

### 3. 编辑器集成

- **可视化编辑**：在编辑器中可视化编辑场景的布局和属性。
- **实时预览**：在编辑器中实时预览场景效果。
- **场景模板**：基于现有场景创建场景模板，快速生成新场景。

## 最佳实践

1. **场景组织**：将游戏划分为合理的场景，每个场景负责特定的功能。
2. **实体管理**：合理组织场景中的实体，避免过多的实体导致性能问题。
3. **资源管理**：合理使用预制体和脚本，避免重复创建相同的游戏对象。
4. **依赖管理**：定期检查并清理无效的依赖关系。
5. **版本控制**：将 .scene 文件纳入版本控制系统，确保团队协作时的一致性。
6. **性能优化**：根据场景的复杂度，合理设置环境参数和实体数量。
7. **哈希计算**：使用 SHA1 算法计算场景文件的哈希值。

## 示例完整文件

```ron
// 场景示例文件
SceneFile({
    version: "1.0",
    scene: Scene({
        name: "Level1",
        description: "First level of the game",
        author: "Game Developer",
        created_at: "2026-04-10T12:00:00Z",
        last_modified: "2026-04-10T12:30:00Z",
    }),
    environment: Environment({
        ambient_light: [0.5, 0.5, 0.5, 1.0],
        fog: Fog({
            enabled: true,
            color: [0.1, 0.1, 0.1, 1.0],
            near: 10.0,
            far: 100.0,
            density: 0.01,
        }),
        gravity: [0, -9.81, 0],
        time_of_day: 10.0,
        weather: "sunny",
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
                    type: "PlayerController",
                    properties: PlayerControllerProperties({
                        speed: 5.0,
                        jump_force: 10.0,
                    }),
                }),
                Component({
                    type: "Health",
                    properties: HealthProperties({
                        max_health: 100,
                        current_health: 100,
                    }),
                }),
            ],
        }),
        Entity({
            id: 2,
            name: "MainCamera",
            parent_id: Some(1),
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [0, 2, -5],
                        rotation: [15, 0, 0],
                        scale: [1, 1, 1],
                    }),
                }),
                Component({
                    type: "Camera",
                    properties: CameraProperties({
                        field_of_view: 60,
                        near_plane: 0.1,
                        far_plane: 1000,
                        clear_color: [0.5, 0.7, 1.0, 1.0],
                    }),
                }),
            ],
        }),
        Entity({
            id: 3,
            name: "Ground",
            parent_id: None,
            components: [
                Component({
                    type: "Transform",
                    properties: TransformProperties({
                        position: [0, -1, 0],
                        rotation: [0, 0, 0],
                        scale: [20, 1, 20],
                    }),
                }),
                Component({
                    type: "MeshRenderer",
                    properties: MeshRendererProperties({
                        mesh: "assets/meshes/ground.obj",
                        material: "assets/materials/ground.mat",
                    }),
                }),
                Component({
                    type: "Collider",
                    properties: ColliderProperties({
                        shape: "box",
                        size: [20, 1, 20],
                        is_trigger: false,
                    }),
                }),
            ],
        }),
    ],
    prefabs: [
        PrefabInstance({
            path: "assets/prefabs/enemy.prefab",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0004",
            instances: [
                PrefabInstanceData({
                    id: 4,
                    name: "Enemy1",
                    position: [5, 0, 0],
                    rotation: [0, 0, 0],
                    scale: [1, 1, 1],
                }),
                PrefabInstanceData({
                    id: 5,
                    name: "Enemy2",
                    position: [-5, 0, 0],
                    rotation: [0, 0, 0],
                    scale: [1, 1, 1],
                }),
                PrefabInstanceData({
                    id: 6,
                    name: "Enemy3",
                    position: [0, 0, 5],
                    rotation: [0, 90, 0],
                    scale: [1, 1, 1],
                }),
            ],
        }),
        PrefabInstance({
            path: "assets/prefabs/health_pack.prefab",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0005",
            instances: [
                PrefabInstanceData({
                    id: 7,
                    name: "HealthPack1",
                    position: [3, 0, 3],
                    rotation: [0, 0, 0],
                    scale: [1, 1, 1],
                }),
            ],
        }),
    ],
    scripts: [
        Script({
            path: "assets/scripts/level_manager.gscript",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0006",
            enabled: true,
        }),
        Script({
            path: "assets/scripts/enemy_spawner.gscript",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0007",
            enabled: true,
        }),
    ],
    dependencies: [
        Dependency({
            path: "assets/prefabs/enemy.prefab",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0004",
        }),
        Dependency({
            path: "assets/prefabs/health_pack.prefab",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0005",
        }),
        Dependency({
            path: "assets/scripts/level_manager.gscript",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0006",
        }),
        Dependency({
            path: "assets/scripts/enemy_spawner.gscript",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0007",
        }),
        Dependency({
            path: "assets/meshes/ground.obj",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0008",
        }),
        Dependency({
            path: "assets/materials/ground.mat",
            guid: "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0009",
        }),
    ],
    references: [],
    timestamp: "2026-04-10T12:30:00Z",
    hash: "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3",
})
```

## 总结

*.scene 文件格式为 GG 游戏引擎提供了一种统一、有效的方式来管理游戏场景。通过存储场景的环境设置、实体布局、预制体实例和脚本，它解决了游戏世界构建和管理的问题。

这种设计使得 GG 游戏引擎能够：
- 快速创建和管理游戏场景
- 支持复杂的场景结构和层次关系
- 与预制体和脚本系统无缝集成
- 提供环境设置和物理参数的配置
- 支持场景的版本控制和依赖管理

场景系统是 GG 游戏引擎中重要的组成部分，为游戏开发者提供了一种高效、灵活的方式来构建和管理游戏世界。