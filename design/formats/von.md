# VON 格式规范

## 概述

VON（Valkyrie Object Notation）是 GG 游戏引擎使用的一种数据序列化格式，类似于 RON（Rusty Object Notation），但有一些简化和调整。VON 格式用于存储游戏中的各种数据，如预制体、场景、材质等。

## 基本语法

### 1. 基本结构

VON 格式使用大括号 `{}` 来表示对象，使用逗号 `,` 分隔字段。

```von
# 基本对象结构
Object {
    field1: value1,
    field2: value2,
    field3: value3,
}
```

### 2. Class 名称

Class 名称是可选的，如果不指定，则默认为匿名对象。

```von
# 带 Class 名称的对象
Player {
    name: "Alice",
    level: 10,
    health: 100,
}

# 匿名对象
{
    name: "Bob",
    level: 5,
    health: 50,
}
```

### 3. 字段

字段名称一般不需要引号，直接使用标识符。

```von
# 字段名称不需要引号
Person {
    first_name: "John",
    last_name: "Doe",
    age: 30,
}
```

### 4. 值类型

VON 支持多种值类型：

- **字符串**：使用双引号 `""` 包围
- **数字**：直接写入，支持整数和浮点数
- **布尔值**：`true` 或 `false`
- **数组**：使用方括号 `[]` 包围，元素用逗号分隔
- **对象**：使用大括号 `{}` 包围
- **空值**：`null`

```von
# 各种值类型
Example {
    string_value: "Hello, World!",
    int_value: 42,
    float_value: 3.14,
    bool_value: true,
    array_value: [1, 2, 3, 4, 5],
    object_value: {
        nested_field: "Nested value",
    },
    null_value: null,
}
```

### 5. 注释

VON 使用 `#` 符号进行单行注释。

```von
# 注释示例
Person {
    name: "Alice", # 人物名称
    age: 25,       # 人物年龄
}
```

### 6. 变体（Variant）

直接使用对象作为变体，不需要额外的 Variant 包装。

```von
# 变体示例
Player {
    name: "Alice",
    level: 10,
}

# 另一个变体示例
Enemy {
    name: "Goblin",
    level: 5,
}
```

## 格式示例

### 预制体文件示例

```von
PrefabFile {
    version: "1.0",
    prefab: Prefab {
        name: "PlayerPrefab",
        description: "Player character prefab",
        is_variant: false,
        base_prefab: null,
    },
    entities: [
        Entity {
            id: 1,
            name: "Player",
            parent_id: null,
            components: [
                Component {
                    type: "Transform",
                    properties: TransformProperties {
                        position: [0, 0, 0],
                        rotation: [0, 0, 0],
                        scale: [1, 1, 1],
                    },
                },
            ],
        },
    ],
    variants: [],
    dependencies: [],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

### 材质文件示例

```von
MaterialFile {
    version: "1.0",
    material: Material {
        name: "DefaultMaterial",
        shader: "assets/shaders/default.shader",
        properties: [
            MaterialProperty {
                name: "albedo",
                type: "texture",
                value: "assets/textures/albedo.png",
            },
            MaterialProperty {
                name: "metallic",
                type: "float",
                value: "0.5",
            },
        ],
    },
    dependencies: [],
    references: [],
    timestamp: "2026-04-10T12:00:00Z",
    hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
}
```

## 与 RON 的区别

VON 格式基于 RON 格式，但有以下区别：

1. **Class 名称可选**：VON 中 Class 名称是可选的，而 RON 中通常是必需的
2. **字段名称不需要引号**：VON 中字段名称可以直接使用标识符，不需要引号
3. **注释使用 #**：VON 使用 `#` 进行注释，而 RON 使用 `//`
4. **变体名称必选**：VON 中变体的名称是必选的
5. **缩进建议**：VON 建议使用 4 空格缩进

## 最佳实践

1. **使用缩进**：建议使用 4 空格缩进，提高可读性
2. **添加注释**：使用 `#` 添加注释，说明复杂结构的含义
3. **命名规范**：字段名称使用 snake\_case，Class 名称使用 PascalCase
4. **保持一致性**：在整个项目中保持 VON 格式的一致性
5. **使用工具**：使用 GG 游戏引擎提供的 VON 解析工具进行验证和格式化

## 总结

VON 格式是 GG 游戏引擎使用的一种简洁、易读的数据序列化格式，类似于 RON 但有一些简化和调整。它适用于存储游戏中的各种数据，如预制体、场景、材质等。通过遵循 VON 格式规范，可以确保数据的一致性和可读性，提高开发效率。
