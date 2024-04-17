# gg-render

**GG Game Engine 的渲染系统抽象，提供统一的渲染接口和基础渲染功能。**

## 📋 模块简介

gg-render 是 GG Game Engine 的渲染系统抽象，提供统一的渲染接口和基础渲染功能，为不同的渲染后端（如 WGPU）提供统一的抽象层。

## ✨ 核心功能

- **渲染抽象**：为不同渲染后端提供统一的接口
- **图形渲染**：支持基本的 2D 图形渲染
- **文本渲染**：支持文本和字体渲染
- **颜色管理**：提供颜色处理和转换功能
- **渲染命令**：支持批处理和优化的渲染命令
- **变换系统**：提供 2D 变换和坐标系统

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-render = { path = "projects/core/gg-render" }
```

### 基础示例

```rust
use gg_render::prelude::*;

fn main() {
    // 创建渲染器（实际使用时会由具体后端实现）
    let mut renderer = Renderer::new();
    
    // 创建表面
    let surface = Surface::new(800, 600);
    
    // 开始渲染
    renderer.begin_frame(&surface);
    
    // 绘制矩形
    let rect = Rect::new(100.0, 100.0, 200.0, 150.0);
    let color = Color::rgb(1.0, 0.0, 0.0);
    renderer.draw_rect(rect, color);
    
    // 绘制文本
    let font = Font::default();
    renderer.draw_text("Hello, GG Game Engine!", (50.0, 50.0), font, Color::white());
    
    // 结束渲染
    renderer.end_frame();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [渲染系统设计](../../../design/architecture/overview.md) - 了解渲染系统的设计理念