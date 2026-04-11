# gg-ui-perf

**GG Game Engine 的 UI 性能优化模块，负责提升 UI 渲染和交互性能。**

## 📋 模块简介

gg-ui-perf 是 GG Game Engine 的 UI 性能优化模块，专注于提升 UI 渲染和交互性能，提供高效的 UI 渲染策略和性能分析工具。

## ✨ 核心功能

- **UI 渲染优化**：优化 UI 元素的渲染性能
- **批量渲染**：支持 UI 元素的批量渲染
- **性能分析**：提供 UI 性能分析工具
- **线程优化**：利用多线程提升 UI 处理能力
- **内存管理**：优化 UI 相关的内存使用

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-ui-perf = { path = "projects/core/gg-ui-perf" }
```

### 基础示例

```rust
use gg_ui_perf::prelude::*;

fn main() {
    // 创建 UI 性能管理器
    let ui_perf_manager = UiPerfManager::new();
    
    // 启用性能分析
    ui_perf_manager.enable_profiling();
    
    // 执行 UI 渲染
    ui_perf_manager.render_ui(|| {
        // UI 渲染代码
    });
    
    // 获取性能报告
    let report = ui_perf_manager.get_performance_report();
    println!("UI 渲染时间: {}ms", report.render_time);
}
```

## 📦 依赖关系

- **gg-error**：错误处理系统
- **num_cpus**：CPU 核心数检测

## 📖 相关文档

- [UI 系统架构](../../../doc/ui-system-architecture.md) - 了解 UI 系统的架构设计