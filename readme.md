# GG Game Engine

**现代化的 GAL 游戏引擎，基于 Rust 构建，提供跨平台支持和丰富的插件系统。**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## ✨ 核心特性

| 特性               | 说明                                   |
|------------------|--------------------------------------|
| 🎮 **GAL 游戏专用**    | 专为视觉小说和 GAL 游戏设计，提供丰富的对话、角色和场景管理功能    |
| 🦀 **纯血 Rust**   | 利用 Rust 的内存安全和性能优势，确保游戏运行稳定高效 |
| 📱 **跨平台支持**    | 支持桌面（Windows、macOS、Linux）、Web 和移动平台        |
| 🔌 **插件系统**     | 模块化设计，支持通过插件扩展功能，如对话系统、角色立绘、场景过渡等    |
| 🎨 **渲染系统**     | 基于 WGPU 的现代化渲染系统，支持 2D 图形和文本渲染       |
| 📜 **脚本系统**     | 内置 Valkyrie 脚本引擎，支持游戏逻辑和剧情编写         |
| 🔧 **编辑器集成**    | 提供可视化编辑器，简化游戏开发流程          |

---

## 📦 模块概览

GG Game Engine 采用模块化的 Crate 设计，每个模块可独立使用：

### 核心模块 (core)
- **[gg-core](projects/core/gg-core/)**: 核心功能和平台抽象
- **[gg-ecs](projects/core/gg-ecs/)**: 实体组件系统
- **[gg-asset](projects/core/gg-asset/)**: 资源管理系统
- **[gg-reflection](projects/core/gg-reflection/)**: 运行时反射系统
- **[gg-render](projects/core/gg-render/)**: 渲染系统抽象
- **[gg-schedule](projects/core/gg-schedule/)**: 任务调度系统
- **[gg-world](projects/core/gg-world/)**: 游戏世界管理
- **[gg-error](projects/core/gg-error/)**: 错误处理系统

### 编译器模块 (compiler)
- **[gg-compiler-core](projects/compiler/gg-compiler-core/)**: 编译器核心功能
- **[gg-compiler-script](projects/compiler/gg-compiler-script/)**: 脚本编译器
- **[gg-compiler-aot](projects/compiler/gg-compiler-aot/)**: AOT 编译器
- **[gg-script](projects/compiler/gg-script/)**: 脚本解析和执行

### 编辑器模块 (editor)
- **[gg-editor-shell](projects/editor/gg-editor-shell/)**: 编辑器外壳和基础框架
- **[gg-editor-asset-browser](projects/editor/gg-editor-asset-browser/)**: 资源浏览器
- **[gg-editor-inspector](projects/editor/gg-editor-inspector/)**: 属性检查器
- **[gg-editor-scene](projects/editor/gg-editor-scene/)**: 场景编辑器
- **[gg-editor-script](projects/editor/gg-editor-script/)**: 脚本编辑器
- **[gg-editor-preview](projects/editor/gg-editor-preview/)**: 游戏预览
- **[gg-editor-character](projects/editor/gg-editor-character/)**: 角色编辑器
- **[gg-editor-lsp](projects/editor/gg-editor-lsp/)**: 语言服务器协议支持

### 平台模块 (platforms)
- **[gg-platform-desktop](projects/platforms/gg-platform-desktop/)**: 桌面平台实现
- **[gg-platform-web](projects/platforms/gg-platform-web/)**: Web 平台实现
- **[gg-platform-mobile](projects/platforms/gg-platform-mobile/)**: 移动平台实现

### 插件模块 (plugins)
- **[gg-plugin-dialogue](projects/plugins/gg-plugin-dialogue/)**: 对话系统插件
- **[gg-plugin-portrait](projects/plugins/gg-plugin-portrait/)**: 角色立绘插件
- **[gg-plugin-save](projects/plugins/gg-plugin-save/)**: 存档系统插件
- **[gg-plugin-scene-transition](projects/plugins/gg-plugin-scene-transition/)**: 场景过渡插件
- **[gg-plugin-spine](projects/plugins/gg-plugin-spine/)**: Spine 动画插件
- **[gg-plugin-tilemap](projects/plugins/gg-plugin-tilemap/)**: 瓦片地图插件
- **[gg-ui](projects/plugins/gg-ui/)**: UI 系统
- **[gg-galgame-schema](projects/plugins/gg-galgame-schema/)**: GAL 游戏数据 schema

### 运行时模块 (runtime)
- **[gg-runtime-core](projects/runtime/gg-runtime-core/)**: 运行时核心
- **[gg-runtime-audio](projects/runtime/gg-runtime-audio/)**: 音频系统
- **[gg-render-wgpu](projects/runtime/gg-render-wgpu/)**: WGPU 渲染实现
- **[gg-bytecode](projects/runtime/gg-bytecode/)**: 字节码执行
- **[gg-ir](projects/runtime/gg-ir/)**: 中间表示
- **[gg-vm](projects/runtime/gg-vm/)**: 虚拟机

### 工具链模块 (toolchain)
- **[gg-cli](projects/toolchain/gg-cli/)**: 命令行工具
- **[gg-factory](projects/toolchain/gg-factory/)**: 项目生成器
- **[gg-manifest](projects/toolchain/gg-manifest/)**: 项目配置管理

---

## 🚀 快速开始

### 安装

```toml
[dependencies]
gg-runtime-core = { path = "projects/runtime/gg-runtime-core" }
```

### 基础示例

```rust
use gg_runtime_core::prelude::*;

fn main() -> Result<()> {
    let app = App::builder()
        .with_plugins([
            gg_plugin_dialogue::DialoguePlugin::default(),
            gg_plugin_portrait::PortraitPlugin::default(),
        ])
        .build();
    
    app.run()
}
```

更多详细示例请查看 [examples](examples/) 目录。

---

## 📖 文档

- [设计文档](design/index.md) - 了解引擎的设计理念和架构
- [快速开始](design/guide/getting-started.md) - 5 分钟上手 GG Game Engine
- [核心优势](design/guide/advantages.md) - 深入了解各模块特性
- [架构设计](design/architecture/overview.md) - 模块化设计原则

---

## 🎯 项目结构

```
gg-game-engine/
├── design/          # 设计文档
├── examples/        # 示例项目
├── projects/        # 核心模块
│   ├── core/        # 核心功能
│   ├── compiler/    # 编译器
│   ├── editor/      # 编辑器
│   ├── galgame/     # GAL 游戏模板
│   ├── platforms/   # 平台实现
│   ├── plugins/     # 插件系统
│   ├── runtime/     # 运行时
│   └── toolchain/   # 工具链
└── scripts/         # 辅助脚本
```

---

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何参与。

---

## 📄 License

MIT License