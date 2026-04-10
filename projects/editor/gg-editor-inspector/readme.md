# gg-editor-inspector

**GG Game Engine 的属性检查器，负责编辑游戏对象的属性。**

## 📋 模块简介

gg-editor-inspector 是 GG Game Engine 的属性检查器，负责编辑游戏对象的属性，提供可视化的属性编辑界面。

## ✨ 核心功能

- **属性编辑**：编辑游戏对象的各种属性
- **类型支持**：支持各种类型的属性编辑（数值、字符串、布尔值、枚举等）
- **自定义编辑器**：支持为特定类型创建自定义编辑器
- **实时预览**：实时预览属性修改的效果
- **属性绑定**：支持属性与其他对象的绑定
- **分组和折叠**：支持属性的分组和折叠

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-editor-inspector = { path = "projects/editor/gg-editor-inspector" }
```

### 基础示例

```rust
use gg_editor_inspector::prelude::*;

// 定义可编辑类型
#[derive(Reflect)]
struct Player {
    name: String,
    health: f32,
    position: (f32, f32),
    is_alive: bool,
}

fn main() {
    // 创建属性检查器面板
    let mut inspector = InspectorPanel::new();
    
    // 创建对象实例
    let mut player = Player {
        name: "Alice".to_string(),
        health: 100.0,
        position: (0.0, 0.0),
        is_alive: true,
    };
    
    // 设置检查对象
    inspector.set_target(&mut player);
    
    // 显示面板
    inspector.show();
}
```

## 📦 依赖关系

- **gg-editor-shell**：编辑器外壳和基础框架
- **gg-core**：核心功能和平台抽象
- **gg-reflection**：反射系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [编辑器设计](../../../design/architecture/overview.md) - 了解编辑器系统的设计理念