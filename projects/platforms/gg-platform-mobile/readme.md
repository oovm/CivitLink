# gg-platform-mobile

**GG Game Engine 的移动平台实现，支持 iOS 和 Android。**

## 📋 模块简介

gg-platform-mobile 是 GG Game Engine 的移动平台实现，支持 iOS 和 Android 平台，提供文件系统、输入、时间和服务管理等功能。

## ✨ 核心功能

- **文件系统**：提供移动平台的文件操作功能
- **输入系统**：处理触摸、手势和设备按钮输入
- **时间管理**：提供高精度的时间测量和管理
- **服务管理**：管理移动平台的各种服务
- **生命周期管理**：处理应用的生命周期事件
- **移动特性**：支持移动平台特有功能，如传感器

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-platform-mobile = { path = "projects/platforms/gg-platform-mobile" }
```

### 基础示例

```rust
use gg_platform_mobile::prelude::*;

fn main() {
    // 初始化移动平台
    let platform = MobilePlatform::new();
    
    // 获取文件系统
    let fs = platform.fs();
    
    // 读取文件
    let content = fs.read_to_string("assets/config.toml").unwrap();
    println!("Config content: {}", content);
    
    // 获取输入系统
    let input = platform.input();
    
    // 检查触摸状态
    if input.is_touch_pressed() {
        let touch_pos = input.get_touch_position();
        println!("Touch pressed at: {:?}", touch_pos);
    }
    
    // 获取时间系统
    let time = platform.time();
    
    // 获取当前时间
    let current_time = time.now();
    println!("Current time: {:?}", current_time);
    
    // 注册生命周期事件
    platform.on_resume(|| {
        println!("App resumed!");
    });
    
    platform.on_pause(|| {
        println!("App paused!");
    });
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [平台抽象设计](../../../design/architecture/layers.md) - 了解平台抽象层的设计理念