# gg-plugin-scene-transition

GG Game Engine 的场景过渡插件，提供 Galgame 风格的场景切换效果。

## 功能

- 场景过渡效果
- 淡入淡出
- 滑动效果
- 与 ECS 系统集成

## 依赖

- gg-core
- gg-ecs
- gg-asset
- gg-galgame-schema

## 使用

```rust
use gg_plugin_scene_transition::SceneTransitionSystem;

// 初始化场景过渡系统
let transition_system = SceneTransitionSystem::new();

// 执行场景过渡
transition_system.fade_to_black(1.0); // 1秒淡入黑屏

// 更新过渡系统
transition_system.update(&mut world);
```
