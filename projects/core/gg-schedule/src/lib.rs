#![warn(missing_docs)]

//! GG 引擎调度模块
//!
//! 提供系统调度、标签和执行顺序管理功能。
//!
//! # 核心概念
//!
//! - [`ScheduleLabel`] - 调度标签，标识不同的调度阶段
//! - [`SystemSet`] - 系统集合，对系统进行分组
//! - [`SystemWrapper`] - 系统包装器，将函数系统包装为可调度单元
//! - [`Schedule`] - 调度器，管理一组系统的执行顺序

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
};

/// 调度标签 trait，用于标识不同的调度阶段
///
/// 所有调度标签必须实现此 trait。由于需要支持 trait 对象（`Box<dyn ScheduleLabel>`），
/// 提供了对象安全的克隆、比较和哈希方法。
pub trait ScheduleLabel: Debug + Send + Sync + 'static {
    /// 获取标签名称
    fn label_name(&self) -> &'static str;

    /// 克隆为装箱的 trait 对象
    fn clone_box(&self) -> Box<dyn ScheduleLabel>;

    /// 判断是否与另一个标签相等
    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool;

    /// 将标签哈希值写入哈希器
    fn hash_box(&self, hasher: &mut dyn Hasher);
}

impl Clone for Box<dyn ScheduleLabel> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn ScheduleLabel> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_box(other.as_ref())
    }
}

impl Eq for Box<dyn ScheduleLabel> {}

impl Hash for Box<dyn ScheduleLabel> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_box(state);
    }
}

/// 游戏启动标签，仅在游戏启动时运行一次
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Startup;

impl ScheduleLabel for Startup {
    fn label_name(&self) -> &'static str {
        "Startup"
    }

    fn clone_box(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool {
        other.label_name() == self.label_name()
    }

    fn hash_box(&self, hasher: &mut dyn Hasher) {
        hasher.write(self.label_name().as_bytes());
    }
}

/// 每帧更新标签
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Update;

impl ScheduleLabel for Update {
    fn label_name(&self) -> &'static str {
        "Update"
    }

    fn clone_box(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool {
        other.label_name() == self.label_name()
    }

    fn hash_box(&self, hasher: &mut dyn Hasher) {
        hasher.write(self.label_name().as_bytes());
    }
}

/// 固定间隔更新标签，用于物理模拟
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixedUpdate;

impl ScheduleLabel for FixedUpdate {
    fn label_name(&self) -> &'static str {
        "FixedUpdate"
    }

    fn clone_box(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool {
        other.label_name() == self.label_name()
    }

    fn hash_box(&self, hasher: &mut dyn Hasher) {
        hasher.write(self.label_name().as_bytes());
    }
}

/// 后更新标签，在 Update 之后运行
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostUpdate;

impl ScheduleLabel for PostUpdate {
    fn label_name(&self) -> &'static str {
        "PostUpdate"
    }

    fn clone_box(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool {
        other.label_name() == self.label_name()
    }

    fn hash_box(&self, hasher: &mut dyn Hasher) {
        hasher.write(self.label_name().as_bytes());
    }
}

/// 渲染阶段标签
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Render;

impl ScheduleLabel for Render {
    fn label_name(&self) -> &'static str {
        "Render"
    }

    fn clone_box(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool {
        other.label_name() == self.label_name()
    }

    fn hash_box(&self, hasher: &mut dyn Hasher) {
        hasher.write(self.label_name().as_bytes());
    }
}

/// 游戏退出阶段标签
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Exit;

impl ScheduleLabel for Exit {
    fn label_name(&self) -> &'static str {
        "Exit"
    }

    fn clone_box(&self) -> Box<dyn ScheduleLabel> {
        Box::new(self.clone())
    }

    fn eq_box(&self, other: &dyn ScheduleLabel) -> bool {
        other.label_name() == self.label_name()
    }

    fn hash_box(&self, hasher: &mut dyn Hasher) {
        hasher.write(self.label_name().as_bytes());
    }
}

/// 系统集合 trait，用于对系统进行分组
///
/// 所有系统集合必须实现此 trait，提供集合名称。
pub trait SystemSet: Clone + Debug + PartialEq + Eq + Hash + Send + Sync + 'static {
    /// 获取集合名称
    fn set_name(&self) -> &'static str;
}

/// 核心系统集合枚举，定义引擎内置的系统执行阶段
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoreSet {
    /// 启动阶段
    Startup,
    /// 首个执行阶段
    First,
    /// 更新前阶段
    PreUpdate,
    /// 更新阶段
    Update,
    /// 更新后阶段
    PostUpdate,
    /// 末尾阶段
    Last,
}

impl SystemSet for CoreSet {
    fn set_name(&self) -> &'static str {
        match self {
            CoreSet::Startup => "Startup",
            CoreSet::First => "First",
            CoreSet::PreUpdate => "PreUpdate",
            CoreSet::Update => "Update",
            CoreSet::PostUpdate => "PostUpdate",
            CoreSet::Last => "Last",
        }
    }
}

/// 系统包装器，将函数系统包装为可调度单元
///
/// 支持设置所属集合和执行顺序约束（before/after）。
pub struct SystemWrapper {
    /// 系统名称，用于标识和排序
    pub name: String,
    /// 系统执行函数
    pub func: Box<dyn FnMut(&mut gg_ecs::World) -> gg_error::GResult<()> + Send + Sync>,
    /// 所属核心集合
    pub set: Option<CoreSet>,
    /// 在此列表中的系统之前执行
    pub before: Vec<String>,
    /// 在此列表中的系统之后执行
    pub after: Vec<String>,
}

impl SystemWrapper {
    /// 创建新的系统包装器
    ///
    /// # 参数
    ///
    /// - `name` - 系统名称，用于标识和排序
    /// - `func` - 系统执行函数
    pub fn new(
        name: impl Into<String>,
        func: impl FnMut(&mut gg_ecs::World) -> gg_error::GResult<()> + Send + Sync + 'static,
    ) -> Self {
        Self { name: name.into(), func: Box::new(func), set: None, before: Vec::new(), after: Vec::new() }
    }

    /// 设置系统所属的核心集合
    pub fn in_set(mut self, set: CoreSet) -> Self {
        self.set = Some(set);
        self
    }

    /// 声明此系统在指定名称的系统之前执行
    pub fn before_system(mut self, name: &str) -> Self {
        self.before.push(name.to_string());
        self
    }

    /// 声明此系统在指定名称的系统之后执行
    pub fn after_system(mut self, name: &str) -> Self {
        self.after.push(name.to_string());
        self
    }
}

/// 系统名称自动编号计数器
static SYSTEM_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 系统输出转换 trait
///
/// 将不同返回类型的系统函数统一转换为 [`GResult`](gg_error::GResult)`<()>`。
/// 支持 `GResult<()>` 和 `()` 两种返回类型。
pub trait SystemOutput {
    /// 将输出转换为 GResult<()>
    fn into_result(self) -> gg_error::GResult<()>;
}

impl SystemOutput for gg_error::GResult<()> {
    fn into_result(self) -> gg_error::GResult<()> {
        self
    }
}

impl SystemOutput for () {
    fn into_result(self) -> gg_error::GResult<()> {
        Ok(())
    }
}

/// 系统转换 trait，允许普通函数直接作为系统添加到调度器
///
/// 实现此 trait 的类型可以转换为 [`SystemWrapper`]，从而添加到 [`Schedule`] 中。
/// 目前支持返回 [`GResult`](gg_error::GResult) 的函数和不返回值的函数。
pub trait IntoSystem<In, Out> {
    /// 转换为系统包装器
    fn into_system(self) -> SystemWrapper;
}

impl<F, Out> IntoSystem<(), Out> for F
where
    F: FnMut(&mut gg_ecs::World) -> Out + Send + Sync + 'static,
    Out: SystemOutput,
{
    fn into_system(self) -> SystemWrapper {
        let id = SYSTEM_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!("system_{}", id);
        let mut func = self;
        SystemWrapper::new(name, move |world: &mut gg_ecs::World| func(world).into_result())
    }
}

impl IntoSystem<(), ()> for SystemWrapper {
    fn into_system(self) -> SystemWrapper {
        self
    }
}

/// 调度器，管理一组系统的执行顺序
///
/// 每个调度器关联一个标签，包含一组系统，
/// 并支持基于 before/after 依赖约束的拓扑排序执行。
pub struct Schedule {
    /// 调度标签
    pub label: Box<dyn ScheduleLabel>,
    /// 系统列表
    pub systems: Vec<SystemWrapper>,
}

impl Schedule {
    /// 创建新的调度器
    ///
    /// # 参数
    ///
    /// - `label` - 调度标签，用于标识此调度器
    pub fn new(label: impl ScheduleLabel) -> Self {
        Self { label: label.clone_box(), systems: Vec::new() }
    }

    /// 添加系统到调度器
    ///
    /// 接受任何实现了 [`IntoSystem`] trait 的类型，包括普通函数。
    ///
    /// # 参数
    ///
    /// - `system` - 可转换为系统包装器的类型
    pub fn add_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) -> &mut Self {
        self.systems.push(system.into_system());
        self
    }

    /// 执行所有系统
    ///
    /// 根据 before/after 约束进行拓扑排序后依次执行。
    /// 若无约束则按添加顺序执行。
    /// 若检测到循环依赖则返回错误。
    pub fn run(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        let order = self.topological_sort()?;
        let mut systems = std::mem::take(&mut self.systems);
        for idx in order {
            (systems[idx].func)(world)?;
        }
        self.systems = systems;
        Ok(())
    }

    /// 基于 before/after 依赖约束进行拓扑排序
    ///
    /// 使用 Kahn 算法，若检测到循环依赖则返回错误。
    fn topological_sort(&self) -> gg_error::GResult<Vec<usize>> {
        let n = self.systems.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let mut name_to_idx: HashMap<String, usize> = HashMap::new();
        for (i, sys) in self.systems.iter().enumerate() {
            name_to_idx.insert(sys.name.clone(), i);
        }

        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut in_degree: Vec<usize> = vec![0; n];

        for (i, sys) in self.systems.iter().enumerate() {
            for before_name in &sys.before {
                if let Some(&j) = name_to_idx.get(before_name) {
                    adj[i].push(j);
                    in_degree[j] += 1;
                }
            }
            for after_name in &sys.after {
                if let Some(&j) = name_to_idx.get(after_name) {
                    adj[j].push(i);
                    in_degree[i] += 1;
                }
            }
        }

        let mut queue: VecDeque<usize> = VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i);
            }
        }

        let mut result = Vec::with_capacity(n);
        while let Some(node) = queue.pop_front() {
            result.push(node);
            for &neighbor in &adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if result.len() != n {
            return Err(gg_error::GError {
                kind: gg_error::GErrorKind::Runtime,
                message: "Cycle detected in system dependencies".to_string(),
            });
        }

        Ok(result)
    }
}

/// 多调度器协调器，管理多个 [`Schedule`]
///
/// 将游戏生命周期划分为启动、更新、固定更新、后更新、渲染和退出六个阶段，
/// 每个阶段对应一个独立的调度器。启动阶段仅执行一次，其余阶段每帧执行。
pub struct Schedules {
    /// 启动阶段调度器
    startup: Schedule,
    /// 每帧更新调度器
    update: Schedule,
    /// 固定间隔更新调度器
    fixed_update: Schedule,
    /// 后更新调度器
    post_update: Schedule,
    /// 渲染阶段调度器
    render: Schedule,
    /// 退出阶段调度器
    exit: Schedule,
    /// 启动阶段是否已执行
    startup_ran: bool,
}

impl Schedules {
    /// 创建新的多调度器协调器
    pub fn new() -> Self {
        Self {
            startup: Schedule::new(Startup),
            update: Schedule::new(Update),
            fixed_update: Schedule::new(FixedUpdate),
            post_update: Schedule::new(PostUpdate),
            render: Schedule::new(Render),
            exit: Schedule::new(Exit),
            startup_ran: false,
        }
    }

    /// 添加系统到启动阶段
    pub fn add_startup_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) {
        self.startup.add_systems(system);
    }

    /// 添加系统到每帧更新阶段
    pub fn add_update_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) {
        self.update.add_systems(system);
    }

    /// 添加系统到固定间隔更新阶段
    pub fn add_fixed_update_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) {
        self.fixed_update.add_systems(system);
    }

    /// 添加系统到后更新阶段
    pub fn add_post_update_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) {
        self.post_update.add_systems(system);
    }

    /// 添加系统到渲染阶段
    pub fn add_render_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) {
        self.render.add_systems(system);
    }

    /// 添加系统到退出阶段
    pub fn add_exit_systems<Out: SystemOutput>(&mut self, system: impl IntoSystem<(), Out>) {
        self.exit.add_systems(system);
    }

    /// 执行启动阶段，仅运行一次
    ///
    /// 首次调用时执行启动调度器中的所有系统，后续调用不再执行。
    pub fn run_startup(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        if !self.startup_ran {
            self.startup.run(world)?;
            self.startup_ran = true;
        }
        Ok(())
    }

    /// 执行每帧更新阶段
    pub fn run_update(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        self.update.run(world)
    }

    /// 执行固定间隔更新阶段
    pub fn run_fixed_update(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        self.fixed_update.run(world)
    }

    /// 执行后更新阶段
    pub fn run_post_update(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        self.post_update.run(world)
    }

    /// 执行渲染阶段
    pub fn run_render(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        self.render.run(world)
    }

    /// 执行退出阶段
    pub fn run_exit(&mut self, world: &mut gg_ecs::World) -> gg_error::GResult<()> {
        self.exit.run(world)
    }

    /// 获取启动阶段调度器的不可变引用
    pub fn startup_schedule(&self) -> &Schedule {
        &self.startup
    }

    /// 获取每帧更新阶段调度器的不可变引用
    pub fn update_schedule(&self) -> &Schedule {
        &self.update
    }

    /// 获取固定间隔更新阶段调度器的不可变引用
    pub fn fixed_update_schedule(&self) -> &Schedule {
        &self.fixed_update
    }

    /// 获取后更新阶段调度器的不可变引用
    pub fn post_update_schedule(&self) -> &Schedule {
        &self.post_update
    }

    /// 获取渲染阶段调度器的不可变引用
    pub fn render_schedule(&self) -> &Schedule {
        &self.render
    }

    /// 获取退出阶段调度器的不可变引用
    pub fn exit_schedule(&self) -> &Schedule {
        &self.exit
    }

    /// 获取启动阶段调度器的可变引用
    pub fn startup_schedule_mut(&mut self) -> &mut Schedule {
        &mut self.startup
    }

    /// 获取每帧更新阶段调度器的可变引用
    pub fn update_schedule_mut(&mut self) -> &mut Schedule {
        &mut self.update
    }

    /// 获取固定间隔更新阶段调度器的可变引用
    pub fn fixed_update_schedule_mut(&mut self) -> &mut Schedule {
        &mut self.fixed_update
    }

    /// 获取后更新阶段调度器的可变引用
    pub fn post_update_schedule_mut(&mut self) -> &mut Schedule {
        &mut self.post_update
    }

    /// 获取渲染阶段调度器的可变引用
    pub fn render_schedule_mut(&mut self) -> &mut Schedule {
        &mut self.render
    }

    /// 获取退出阶段调度器的可变引用
    pub fn exit_schedule_mut(&mut self) -> &mut Schedule {
        &mut self.exit
    }
}

impl Default for Schedules {
    fn default() -> Self {
        Self::new()
    }
}

/// 预导入模块，包含调度模块的核心类型
pub mod prelude {
    pub use crate::{
        CoreSet, Exit, FixedUpdate, IntoSystem, PostUpdate, Render, Schedule, ScheduleLabel, Schedules, Startup, SystemOutput,
        SystemSet, SystemWrapper, Update,
    };
}

