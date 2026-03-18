//! GWG Engine 调度器扩展
//!
//! 提供扩展的系统调度功能。

use gwg_ecs::prelude::*;
use gwg_ecs::schedule::{ScheduleLabel, SystemSet};

pub use gwg_ecs::schedule::*;

/// 标准的游戏执行阶段
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

/// 标准的更新阶段
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

/// 标准的固定更新阶段
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixedUpdate;

/// 标准的后置更新阶段
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostUpdate;

/// 标准的渲染阶段
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Render;

/// 标准的退出阶段
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Exit;

/// 系统集配置，用于组织相关系统
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoreSet {
    /// 初始化系统
    Startup,
    /// 第一阶段更新
    First,
    /// 预处理系统
    PreUpdate,
    /// 主要更新系统
    Update,
    /// 后处理系统
    PostUpdate,
    /// 最后阶段更新
    Last,
}

/// 调度器构建器，用于方便地构建多阶段调度器
pub struct ScheduleBuilder {
    schedules: Vec<(Box<dyn ScheduleLabel>, Schedule)>,
}

impl ScheduleBuilder {
    /// 创建一个新的调度器构建器
    pub fn new() -> Self {
        Self {
            schedules: Vec::new(),
        }
    }

    /// 添加一个调度阶段
    pub fn add_schedule<L: ScheduleLabel + 'static>(mut self, label: L, schedule: Schedule) -> Self {
        self.schedules.push((Box::new(label), schedule));
        self
    }

    /// 添加一个系统到指定调度阶段
    pub fn add_system_to_schedule<L: ScheduleLabel + 'static, M>(
        mut self,
        label: L,
        system: impl IntoSystemConfigs<M>,
    ) -> Self {
        if let Some((_, schedule)) = self
            .schedules
            .iter_mut()
            .find(|(l, _)| l.dyn_eq(&label))
        {
            schedule.add_systems(system);
        }
        self
    }

    /// 构建调度器集合
    pub fn build(self) -> ScheduleCollection {
        ScheduleCollection {
            schedules: self.schedules,
        }
    }
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 调度器集合，管理多个调度阶段
pub struct ScheduleCollection {
    schedules: Vec<(Box<dyn ScheduleLabel>, Schedule)>,
}

impl ScheduleCollection {
    /// 创建一个新的空调度器集合
    pub fn new() -> Self {
        Self {
            schedules: Vec::new(),
        }
    }

    /// 获取指定标签的调度器
    pub fn get<L: ScheduleLabel + 'static>(&self, label: L) -> Option<&Schedule> {
        self.schedules
            .iter()
            .find(|(l, _)| l.dyn_eq(&label))
            .map(|(_, s)| s)
    }

    /// 获取指定标签的调度器的可变引用
    pub fn get_mut<L: ScheduleLabel + 'static>(&mut self, label: L) -> Option<&mut Schedule> {
        self.schedules
            .iter_mut()
            .find(|(l, _)| l.dyn_eq(&label))
            .map(|(_, s)| s)
    }

    /// 运行指定标签的调度器
    pub fn run<L: ScheduleLabel + 'static>(&mut self, label: L, world: &mut gwg_ecs::World) {
        if let Some((_, schedule)) = self
            .schedules
            .iter_mut()
            .find(|(l, _)| l.dyn_eq(&label))
        {
            schedule.run(world.inner_mut());
        }
    }

    /// 按顺序运行所有调度器
    pub fn run_all(&mut self, world: &mut gwg_ecs::World) {
        for (_, schedule) in &mut self.schedules {
            schedule.run(world.inner_mut());
        }
    }

    /// 添加一个调度阶段
    pub fn add_schedule<L: ScheduleLabel + 'static>(&mut self, label: L, schedule: Schedule) {
        self.schedules.push((Box::new(label), schedule));
    }

    /// 添加一个系统到指定调度阶段
    pub fn add_system_to_schedule<L: ScheduleLabel + 'static, M>(
        &mut self,
        label: L,
        system: impl IntoSystemConfigs<M>,
    ) {
        if let Some((_, schedule)) = self
            .schedules
            .iter_mut()
            .find(|(l, _)| l.dyn_eq(&label))
        {
            schedule.add_systems(system);
        }
    }
}

impl Default for ScheduleCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建标准的游戏调度器集合
pub fn create_standard_schedules() -> ScheduleCollection {
    let mut builder = ScheduleBuilder::new();
    
    builder = builder.add_schedule(Startup, Schedule::new(Startup));
    builder = builder.add_schedule(Update, Schedule::new(Update));
    builder = builder.add_schedule(FixedUpdate, Schedule::new(FixedUpdate));
    builder = builder.add_schedule(PostUpdate, Schedule::new(PostUpdate));
    builder = builder.add_schedule(Render, Schedule::new(Render));
    builder = builder.add_schedule(Exit, Schedule::new(Exit));
    
    builder.build()
}

pub mod prelude {
    //! 调度器扩展的预导入模块

    pub use super::{
        CoreSet, Exit, FixedUpdate, PostUpdate, Render, ScheduleBuilder, ScheduleCollection,
        Startup, Update, create_standard_schedules,
    };
    pub use gwg_ecs::schedule::*;
}
