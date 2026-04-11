# gg-platform-desktop

**GG Game Engine 的桌面平台实现，支持 Windows、macOS 和 Linux。**

## 📋 模块简介

gg-platform-desktop 是 GG Game Engine 的桌面平台实现，支持 Windows、macOS 和 Linux 平台，提供文件系统、输入、时间和服务管理等功能。

## ✨ 核心功能

- **文件系统**：提供桌面平台的文件操作功能
- **输入系统**：处理键盘、鼠标和游戏手柄输入
- **时间管理**：提供高精度的时间测量和管理
- **服务管理**：管理桌面平台的各种服务
- **窗口管理**：创建和管理应用窗口
- **系统集成**：与桌面操作系统集成

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-platform-desktop = { path = "projects/platforms/gg-platform-desktop" }
```

### 基础示例

```rust
use gg_platform_desktop::prelude::*;

fn main() {
    // 初始化桌面平台
    let platform = DesktopPlatform::new();
    
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
    
    // 获取时间系统
    let time = platform.time();
    
    // 获取当前时间
    let current_time = time.now();
    println!("Current time: {:?}", current_time);
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [平台抽象设计](../../../design/architecture/layers.md) - 了解平台抽象层的设计理念