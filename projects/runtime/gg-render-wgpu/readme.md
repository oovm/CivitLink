# gg-render-wgpu

**GG Game Engine 的 WGPU 渲染实现，负责游戏内 UI 和图形的渲染。**

## 📋 模块简介

gg-render-wgpu 是 GG Game Engine 的 WGPU 渲染实现，负责游戏内 UI（如游戏菜单、HUD 等）和图形的渲染，提供基于 WGPU 的现代化渲染功能。

## ✨ 核心功能

- **WGPU 集成**：基于 WGPU 的渲染实现
- **渲染管道**：管理渲染管道和着色器
- **纹理管理**：管理纹理和纹理缓存
- **glyph 缓存**：优化文本渲染的字形缓存
- **着色器系统**：支持自定义着色器
- **性能优化**：优化渲染性能和资源使用

## 🎮 UI 渲染区分

- **Game UI**：使用 WGPU 自渲（本模块负责），确保与游戏渲染的一致性和性能
- **Editor UI**：使用原生组件渲染，由 gg-render-native 模块负责

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-render-wgpu = { path = "projects/runtime/gg-render-wgpu" }
```

### 基础示例

```rust
use gg_render_wgpu::prelude::*;

fn main() {
    // 创建 WGPU 渲染器
    let mut renderer = WgpuRenderer::new();
    
    // 初始化渲染器
    renderer.initialize().unwrap();
    
    // 创建表面
    let surface = renderer.create_surface(800, 600);
    
    // 开始渲染
    renderer.begin_frame(&surface);
    
    // 绘制矩形
    let rect = Rect::new(100.0, 100.0, 200.0, 150.0);
    let color = Color::rgb(1.0, 0.0, 0.0);
    renderer.draw_rect(rect, color);
    
    // 结束渲染
    renderer.end_frame();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-render**：渲染系统抽象
- **gg-error**：错误处理系统
- **wgpu**：WGPU 库

## 📖 相关文档

- [渲染系统设计](../../../design/architecture/overview.md) - 了解渲染系统的设计理念

