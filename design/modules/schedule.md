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
use gwg_schedule::prelude::*;
use gwg_ecs::prelude::*;

fn startup_system() {
    println!("Game starting...");
}

fn update_system() {
    println!("Updating...");
}

fn render_system() {
    println!("Rendering...");
}

fn main() {
    let mut world = GwgWorld::new();
    
    // 创建不同阶段的调度器
    let mut startup_schedule = Schedule::new(Startup);
    let mut update_schedule = Schedule::new(Update);
    let mut render_schedule = Schedule::new(Render);
    
    startup_schedule.add_systems(startup_system);
    update_schedule.add_systems(update_system);
    render_schedule.add_systems(render_system);
    
    // 游戏循环
    startup_schedule.run(&mut world);
    
    loop {
        update_schedule.run(&mut world);
        render_schedule.run(&mut world);
    }
}
```

### 系统集使用

```rust
use gwg_schedule::prelude::*;
use gwg_ecs::prelude::*;

fn input_system() {}
fn physics_system() {}
fn movement_system() {}
fn collision_system() {}

fn main() {
    let mut schedule = Schedule::new(Update);
    
    schedule
        .add_systems((input_system,).in_set(CoreSet::PreUpdate))
        .add_systems((physics_system, movement_system).in_set(CoreSet::Update))
        .add_systems((collision_system,).in_set(CoreSet::PostUpdate));
}
```

### 系统排序

```rust
use gwg_schedule::prelude::*;
use gwg_ecs::prelude::*;

fn system_a() {}
fn system_b() {}
fn system_c() {}

fn main() {
    let mut schedule = Schedule::new(Update);
    
    schedule
        .add_systems(system_a)
        .add_systems(system_b.after(system_a))
        .add_systems(system_c.before(system_a));
}
```

## 相关模块

- [ecs](./ecs.md) - ECS 核心
- [world](./world.md) - 世界管理
- [reflection](./reflection.md) - 反射系统
