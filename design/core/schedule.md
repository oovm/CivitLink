# Schedule 模块

## 概述

Schedule 模块提供扩展的系统调度功能，定义了标准的游戏执行阶段和系统集。

## 核心概念

### 调度标签（ScheduleLabel）
调度标签用于标识不同的执行阶段。

### 系统集（SystemSet）
系统集用于组织相关的系统。

## 核心类型

### 标准调度标签

```rust
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixedUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Render;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Exit;
```

**说明：**

- `Startup` - 游戏启动时执行一次
- `Update` - 每帧执行
- `FixedUpdate` - 固定时间间隔执行（用于物理等）
- `PostUpdate` - Update 之后执行
- `Render` - 渲染阶段
- `Exit` - 游戏退出时执行

### CoreSet

核心系统集，用于组织标准的执行阶段。

```rust
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoreSet {
    Startup,
    First,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
}
```

**说明：**

- `Startup` - 初始化系统
- `First` - 第一阶段更新
- `PreUpdate` - 预处理系统
- `Update` - 主要更新系统
- `PostUpdate` - 后处理系统
- `Last` - 最后阶段更新

## 使用示例

### 基本调度

```rust
use gg_schedule::prelude::*;
use gg_ecs::prelude::*;

struct StartupSystem;

impl System for StartupSystem {
    fn name(&self) -> &str {
        "StartupSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        println!("Game starting...");
        Ok(())
    }
}

struct UpdateSystem;

impl System for UpdateSystem {
    fn name(&self) -> &str {
        "UpdateSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        println!("Updating...");
        Ok(())
    }
}

struct RenderSystem;

impl System for RenderSystem {
    fn name(&self) -> &str {
        "RenderSystem"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        println!("Rendering...");
        Ok(())
    }
}

fn main() {
    let mut world = World::new();
    
    // 注册系统
    world.register_system(Box::new(StartupSystem));
    world.register_system(Box::new(UpdateSystem));
    world.register_system(Box::new(RenderSystem));
    
    // 游戏循环
    world.run_systems().unwrap(); // 执行所有系统
    
    loop {
        world.run_systems().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(16)); // 约 60 FPS
    }
}
```

### 系统排序

```rust
use gg_schedule::prelude::*;
use gg_ecs::prelude::*;

struct SystemA;

impl System for SystemA {
    fn name(&self) -> &str {
        "SystemA"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        println!("System A executed");
        Ok(())
    }
}

struct SystemB;

impl System for SystemB {
    fn name(&self) -> &str {
        "SystemB"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        println!("System B executed");
        Ok(())
    }
}

struct SystemC;

impl System for SystemC {
    fn name(&self) -> &str {
        "SystemC"
    }

    fn execute(&mut self, world: &mut World) -> GResult<()> {
        println!("System C executed");
        Ok(())
    }
}

fn main() {
    let mut world = World::new();
    
    // 注册系统，注意注册顺序决定执行顺序
    world.register_system(Box::new(SystemA));
    world.register_system(Box::new(SystemB));
    world.register_system(Box::new(SystemC));
    
    // 执行系统，顺序为 A -> B -> C
    world.run_systems().unwrap();
}
```

## 相关模块

- [ecs](../core/ecs.md) - ECS 核心
- [world](../core/world.md) - 世界管理