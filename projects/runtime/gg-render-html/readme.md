# gg-render-native

**GG Game Engine 的原生组件渲染实现，负责编辑器 UI 的渲染。**

## 📋 模块简介

gg-render-native 是 GG Game Engine 的原生组件渲染实现，负责编辑器 UI 的渲染，提供基于各平台原生 GUI 系统的渲染功能。

## ✨ 核心功能

- **原生 GUI 集成**：基于各平台原生 GUI 系统的渲染实现
- **跨平台支持**：支持 Windows、macOS、iOS、Android、H5、微信小游戏等平台
- **平台抽象**：处理不同平台原生 GUI 系统的差异
- **与编辑器集成**：与 GG Editor 无缝集成
- **性能优化**：优化原生 GUI 渲染性能

## 🎮 UI 渲染区分

- **Game UI**：使用 WGPU 自渲，由 gg-render-wgpu 模块负责
- **Editor UI**：使用原生组件渲染（本模块负责），获得更好的性能和原生体验

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-render-native = { path = "projects/runtime/gg-render-native" }
```

### 基础示例

```rust
use gg_render_native::prelude::*;

fn main() {
    // 创建原生渲染器
    let mut renderer = NativeRenderer::new();
    
    // 初始化渲染器
    renderer.initialize().unwrap();
    
    // 创建编辑器窗口
    let window = renderer.create_window(800, 600, "GG Editor");
    
    // 开始渲染
    renderer.begin_frame(&window);
    
    // 绘制原生 UI 组件
    renderer.draw_button(100.0, 100.0, 200.0, 50.0, "Click Me");
    
    // 结束渲染
    renderer.end_frame();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-render**：渲染系统抽象
- **gg-error**：错误处理系统
- **原生 GUI 库**：各平台原生 GUI 系统

## 📖 相关文档

- [渲染系统设计](../../../design/architecture/overview.md) - 了解渲染系统的设计理念

