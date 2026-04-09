# gg-plugin-dialogue

GG Game Engine 的对话系统插件，提供 Galgame 风格的对话功能。

## 功能

- 对话系统实现
- 角色对话管理
- 场景切换
- 与 ECS 系统集成

## 依赖

- gg-core
- gg-ecs
- gg-asset
- gg-galgame-schema

## 使用

```rust
use gg_plugin_dialogue::DialogueSystem;

// 初始化对话系统
let dialogue_system = DialogueSystem::new();

// 开始对话
dialogue_system.start_dialogue("intro");

// 更新对话系统
dialogue_system.update(&mut world);
```
