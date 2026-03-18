//! GWG Engine 调度器扩展
//!
//! 提供扩展的系统调度功能。

pub use bevy_ecs::schedule::{ScheduleLabel, SystemSet};
pub use gwg_ecs::*;

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

pub mod prelude {
    //! 调度器扩展的预导入模块

    pub use super::{
        CoreSet, Exit, FixedUpdate, PostUpdate, Render, Startup, Update,
    };
    pub use gwg_ecs::*;
}
