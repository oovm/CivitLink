# gg-plugin-scene-transition

**GG Game Engine 的场景过渡插件，负责管理游戏场景之间的过渡效果。**

## 📋 模块简介

gg-plugin-scene-transition 是 GG Game Engine 的场景过渡插件，负责管理游戏场景之间的过渡效果，提供各种场景切换动画和特效。

## ✨ 核心功能

- **场景过渡**：管理场景之间的过渡效果
- **过渡动画**：提供各种过渡动画效果（淡入淡出、滑动、溶解等）
- **场景管理**：管理场景的加载和卸载
- **过滤器支持**：支持自定义过渡过滤器
- **进度管理**：显示场景加载进度
- **异步加载**：支持场景的异步加载

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-plugin-scene-transition = { path = "projects/plugins/gg-plugin-scene-transition" }
```

### 基础示例

```rust
use gg_plugin_scene_transition::prelude::*;

fn main() {
    // 创建场景过渡系统
    let mut transition_system = SceneTransitionSystem::new();
    
    // 切换场景
    transition_system.switch_scene(
        "scenes/level2.scene",
        TransitionType::Fade(
            Color::black(),
            Duration::from_secs(1)
        )
    );
    
    // 更新过渡系统
    transition_system.update();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
-