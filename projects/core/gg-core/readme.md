# gg-core

**GG Game Engine 的核心模块，提供基础功能和平台抽象。**

## 📋 模块简介

gg-core 是 GG Game Engine 的核心模块，提供了引擎的基础功能和平台抽象层，为其他模块提供统一的接口和工具。

## ✨ 核心功能

- **平台抽象**：为不同平台（桌面、Web、移动）提供统一的接口
- **文件系统**：跨平台的文件操作接口
- **输入系统**：处理键盘、鼠标、触摸等输入事件
- **时间管理**：提供时间相关的功能和工具
- **服务管理**：管理引擎的各种服务

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-core = { path = "projects/core/gg-core" }
```

### 基础示例

```rust
use gg_core::prelude::*;

fn main() {
    // 初始化平台
    let platform = Platform::new();
    
    // 获取文件系统
    let fs = platform.fs();
    
    // 读取文件
    let content = fs.read_to_string("assets/config.toml").unwrap();
    println!("Config content: {}", content);
    
    // 获取输入系统
    let input = platform.input();
    
    // 检查按键状态
    if input.is_key_pressed(KeyCode::Space) {
        println!("Space key pressed!");
    }
}
```

## 📦 依赖关系

- **gg-error**：错误处理系统

## 📖 相关文档

- [平台抽象设计](../../../design/architecture/layers.md) - 了解平台抽象层的设计理念
- [核心模块文档](../../../design/modules/world.md) - 深入了解核心模块的功能