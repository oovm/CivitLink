//! GWG Engine 调度器扩展
//!
//! 提供系统调度功能。

use std::collections::VecDeque;

use gwg_ecs::prelude::*;

/// 系统函数 trait
pub trait System: Send + Sync {
    /// 运行系统
    fn run(&mut self, world: &mut World);
}

/// 函数系统包装器
pub struct FunctionSystem<F> {
    func: F,
}

impl<F> FunctionSystem<F>
where
    F: FnMut(&mut World) + Send + Sync,
{
    /// 创建新的函数系统
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F> System for FunctionSystem<F>
where
    F: FnMut(&mut World) + Send + Sync,
{
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }
}

/// 系统容器
pub struct SystemContainer {
    system: Box<dyn System>,
    name: String,
}

impl SystemContainer {
    /// 创建新的系统容器
    pub fn new(name: impl Into<String>, system: impl System + 'static) -> Self {
        Self {
            system: Box::new(system),
            name: name.into(),
        }
    }

    /// 运行系统
    pub fn run(&mut self, world: &mut World) {
        self.system.run(world);
    }

    /// 获取系统名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 调度器，用于组织和执行系统
pub struct Schedule {
    /// 系统列表
    systems: Vec<SystemContainer>,
    /// 名称
    name: String,
}

impl Schedule {
    /// 创建新的调度器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            systems: Vec::new(),
            name: name.into(),
        }
    }

    /// 添加系统
    pub fn add_system(&mut self, name: impl Into<String>, system: impl System + 'static) {
        self.systems.push(SystemContainer::new(name, system));
    }

    /// 运行所有系统
    pub fn run(&mut self, world: &mut World) {
        for container in &mut self.systems {
            container.run(world);
        }
    }

    /// 获取调度器名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取系统数量
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }
}

/// 主调度器，管理多个调度阶段
pub struct MainSchedule {
    /// 调度阶段列表
    stages: VecDeque<Schedule>,
}

impl MainSchedule {
    /// 创建新的主调度器
    pub fn new() -> Self {
        Self {
            stages: VecDeque::new(),
        }
    }

    /// 添加调度阶段
    pub fn add_stage(&mut self, stage: Schedule) {
        self.stages.push_back(stage);
    }

    /// 在指定阶段之前插入调度阶段
    pub fn insert_stage_before(&mut self, name: &str, stage: Schedule) {
        let pos = self
            .stages
            .iter()
            .position(|s| s.name() == name);
        if let Some(pos) = pos {
            self.stages.insert(pos, stage);
        } else {
            self.stages.push_front(stage);
        }
    }

    /// 在指定阶段之后插入调度阶段
    pub fn insert_stage_after(&mut self, name: &str, stage: Schedule) {
        let pos = self
            .stages
            .iter()
            .position(|s| s.name() == name);
        if let Some(pos) = pos {
            self.stages.insert(pos + 1, stage);
        } else {
            self.stages.push_back(stage);
        }
    }

    /// 运行所有调度阶段
    pub fn run(&mut self, world: &mut World) {
        for stage in &mut self.stages {
            stage.run(world);
        }
    }

    /// 获取调度阶段
    pub fn get_stage(&mut self, name: &str) -> Option<&mut Schedule> {
        self.stages.iter_mut().find(|s| s.name() == name)
    }

    /// 移除调度阶段
    pub fn remove_stage(&mut self, name: &str) -> Option<Schedule> {
        let pos = self
            .stages
            .iter()
            .position(|s| s.name() == name);
        pos.map(|p| self.stages.remove(p).unwrap())
    }
}

impl Default for MainSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建默认的主调度器
pub fn create_default_schedule() -> MainSchedule {
    let mut schedule = MainSchedule::new();
    
    schedule.add_stage(Schedule::new("Startup"));
    schedule.add_stage(Schedule::new("PreUpdate"));
    schedule.add_stage(Schedule::new("Update"));
    schedule.add_stage(Schedule::new("PostUpdate"));
    
    schedule
}

pub mod prelude {
    //! 调度器扩展的预导入模块

    pub use super::{
        create_default_schedule, FunctionSystem, MainSchedule, Schedule, System,
        SystemContainer,
    };
    pub use gwg_ecs::prelude::*;
}
