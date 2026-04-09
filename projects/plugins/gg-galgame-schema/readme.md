# gg-galgame-schema

GG Game Engine 的 Galgame 插件数据结构定义，提供 Galgame 相关的组件和数据模型。

## 功能

- Galgame 数据结构定义
- 支持对话、角色、场景等核心概念
- 与 ECS 系统集成

## 依赖

- gg-core
- gg-ecs
- gg-asset
- serde

## 使用

```rust
use gg_galgame_schema::components::Dialogue;

// 创建对话组件
let dialogue = Dialogue {
    text: "Hello, world!".to_string(),
    speaker: "Character1".to_string(),
    background: Some("bg1".to_string()),
};

// 添加到实体
entity.add_component(dialogue);
```
