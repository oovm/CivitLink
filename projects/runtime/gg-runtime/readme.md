# gg-runtime

**GG Game Engine 的运行时核心，负责游戏的初始化、运行和管理。**

## 📋 模块简介

gg-runtime 是 GG Game Engine 的运行时核心，负责游戏的初始化、运行和管理，提供应用程序的生命周期管理和插件系统。

## ✨ 核心功能

- **应用程序管理**：管理游戏应用程序的生命周期
- **插件系统**：支持通过插件扩展游戏功能
- **热重载**：支持代码和资源的热重载
- **调度器**：管理游戏系统的执行顺序
- **注册表**：管理游戏中的各种资源和服务
- **阶段管理**：将游戏逻辑划分为不同的执行阶段

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-runtime = { path = "projects/runtime/gg-runtime" }
```

### 基础示例

```rust
use gg_runtime_core::prelude::*;

fn main() -> Result<()> {
    // 创建应用程序
    let app = App::builder()
        .with_plugins([
            gg_plugin_dialogue::DialoguePlugin::default(),
            gg_plugin_portrait::PortraitPlugin::default(),
        ])
        .build();
    
    // 运行应用程序
    app.run()
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-world**：游戏世界管理
- **gg-schedule**：任务调度系统
- **gg-error**：错误处理系统

## 📖 相关文档

- [运行时设计](../../../design/architecture/overview.md) - 了解运行时系统的设计理念