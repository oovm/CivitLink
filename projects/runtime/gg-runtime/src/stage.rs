//! 阶段驱动的系统调度类型定义
//!
//! 定义系统执行阶段、系统集合、系统描述符等核心类型，
//! 为阶段调度器提供基础数据结构。

use gg_core::GResult;
use gg_ecs::World;

/// 系统执行阶段
///
/// 定义系统在游戏循环中的执行时机，按固定顺序依次执行。
/// Startup 阶段仅在首次 tick 时执行一次，Exit 阶段在停止时执行。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stage {
    /// 启动阶段，仅在首次 tick 时执行一次
    Startup,
    /// 预更新阶段，在 Update 之前执行
    PreUpdate,
    /// 固定更新阶段，以固定时间步长执行，适用于物理和游戏逻辑
    FixedUpdate,
    /// 更新阶段，主要游戏逻辑执行
    Update,
    /// 后更新阶段，在 Update 之后执行
    PostUpdate,
    /// 渲染阶段，执行渲染逻辑
    Render,
    /// 退出阶段，在 stop 时执行
    Exit,
}

impl Stage {
    /// 获取阶段的显示名称
    pub fn name(&self) -> &'static str {
        match self {
            Stage::Startup => "Startup",
            Stage::PreUpdate => "PreUpdate",
            Stage::FixedUpdate => "FixedUpdate",
            Stage::Update => "Update",
            Stage::PostUpdate => "PostUpdate",
            Stage::Render => "Render",
            Stage::Exit => "Exit",
        }
    }
}

/// 系统集合标识符
///
/// 用于将多个系统归组，以便统一配置排序约束。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemSetId {
    /// 集合标识字符串
    id: String,
}

impl SystemSetId {
    /// 创建新的系统集合标识符
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// 获取标识符字符串引用
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl From<String> for SystemSetId {
    fn from(value: String) -> Self {
        Self { id: value }
    }
}

impl From<&str> for SystemSetId {
    fn from(value: &str) -> Self {
        Self { id: value.to_string() }
    }
}

impl std::fmt::Display for SystemSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// 系统集合 trait
///
/// 实现此 trait 以定义可注册到调度器的系统集合，
/// 系统集合用于对多个系统进行分组和统一排序配置。
pub trait SystemSet: Send + Sync {
    /// 获取系统集合标识符
    fn id(&self) -> SystemSetId;
}

/// 系统函数类型
///
/// 系统函数接收 ECS 世界的可变引用，返回执行结果。
pub type SystemFn = Box<dyn FnMut(&mut World) -> GResult<()> + Send + Sync>;

/// 系统描述符
///
/// 包含系统函数及其调度元信息，如所属阶段、排序约束等。
/// 调度器根据描述符中的信息决定系统的执行顺序。
pub struct SystemDescriptor {
    /// 系统名称，用于排序约束引用
    pub name: String,
    /// 系统函数
    pub system: SystemFn,
    /// 所属阶段
    pub stage: Stage,
    /// 所属系统集合
    pub system_set: Option<SystemSetId>,
    /// 在指定系统之前执行的约束列表
    pub before: Vec<String>,
    /// 在指定系统之后执行的约束列表
    pub after: Vec<String>,
    /// 所属并行组
    pub parallel_group: Option<String>,
}

impl SystemDescriptor {
    /// 创建新的系统描述符
    ///
    /// 使用给定的名称、系统函数和阶段创建描述符，
    /// 排序约束默认为空，系统集合默认为 None。
    pub fn new(name: impl Into<String>, system: SystemFn, stage: Stage) -> Self {
        Self { name: name.into(), system, stage, system_set: None, before: Vec::new(), after: Vec::new(), parallel_group: None }
    }

    /// 设置系统所属的集合
    pub fn in_set(mut self, set: SystemSetId) -> Self {
        self.system_set = Some(set);
        self
    }

    /// 添加前置约束：当前系统必须在指定系统之前执行
    pub fn before(mut self, name: impl Into<String>) -> Self {
        self.before.push(name.into());
        self
    }

    /// 添加后置约束：当前系统必须在指定系统之后执行
    pub fn after(mut self, name: impl Into<String>) -> Self {
        self.after.push(name.into());
        self
    }
}
