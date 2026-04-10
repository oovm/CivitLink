# gg-platform-web

**GG Game Engine 的 Web 平台实现，支持浏览器环境。**

## 📋 模块简介

gg-platform-web 是 GG Game Engine 的 Web 平台实现，支持浏览器环境，提供文件系统、输入、时间和服务管理等功能。

## ✨ 核心功能

- **文件系统**：提供 Web 平台的文件操作功能
- **输入系统**：处理键盘、鼠标和触摸输入
- **时间管理**：提供高精度的时间测量和管理
- **服务管理**：管理 Web 平台的各种服务
- **Canvas 集成**：与浏览器 Canvas 元素集成
- **Web 特性**：支持 Web 特有功能，如 WebAssembly

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-platform-web = { path = "projects/platforms/gg-platform-web" }
```

### 基础示例

```rust
use gg_platform_web::prelude::*;

fn main() {
    // 初始化 Web 平台
    let platform = WebPlatform::new();
    
    // 获取文件系统
    let fs = platform.fs();
    
    // 读取文件
    let content = fs.read_to_string("assets/config.toml").unwrap();
    console::log_1(&format!("Config content: {}", content).into());
    
    // 获取输入系统
    let input = platform.input();
    
    // 检查按键状态
    if input.is_key_pressed(KeyCode::Space) {
        console::log_1(&"Space key pressed!".into());
    }
    
    // 获取时间系统
    let time = platform.time();
    
    // 获取当前时间
    let current_time = time.now();
    console::log_1(&format!("Current time: {:?}", current_time).into());
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统
- **wasm-bindgen**：WebAssembly 绑定

## 📖 相关文档

- [平台抽象设计](../../../design/architecture/layers.md) - 了解平台抽象层的设计理念