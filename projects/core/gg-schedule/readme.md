# gg-schedule

**GG Game Engine 的任务调度系统，负责管理和执行游戏中的各种任务和系统。**

## 📋 模块简介

gg-schedule 是 GG Game Engine 的任务调度系统，负责管理和执行游戏中的各种任务和系统，提供灵活的调度机制和执行顺序控制。

## ✨ 核心功能

- **任务调度**：支持基于优先级的任务调度
- **系统执行**：管理游戏系统的执行顺序和依赖关系
- **阶段管理**：将游戏逻辑划分为不同的执行阶段
- **并行执行**：支持任务的并行执行和同步
- **时间管理**：基于时间的任务调度和延迟执行

## 🚀 使用方法

### 依赖添加

```toml
[dependencies]
gg-schedule = { path = "projects/core/gg-schedule" }
```

### 基础示例

```rust
use gg_schedule::prelude::*;

// 定义系统
struct PhysicsSystem;
impl System for PhysicsSystem {
    fn run(&mut self) {
        println!("Running physics system");
    }
}

struct RenderSystem;
impl System for RenderSystem {
    fn run(&mut self) {
        println!("Running render system");
    }
}

fn main() {
    // 创建调度器
    let mut scheduler = Scheduler::new();
    
    // 添加系统到不同阶段
    scheduler.add_system_to_stage(Stage::Update, PhysicsSystem);
    scheduler.add_system_to_stage(Stage::Render, RenderSystem);
    
    // 运行调度器
    scheduler.run();
}
```

## 📦 依赖关系

- **gg-core**：核心功能和平台抽象
- **gg-error**：错误处理系统

## 📖 相关文档

- [调度系统设计](../../../design/modules/schedule.md) - 了解调度系统的设计理念