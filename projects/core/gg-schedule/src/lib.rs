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

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

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
    pub func: Box<dyn FnMut(&mut gg_ecs::GgWorld) -> gg_error::GResult<()> + Send + Sync>,
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
        func: impl FnMut(&mut gg_ecs::GgWorld) -> gg_error::GResult<()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            func: Box::new(func),
            set: None,
            before: Vec::new(),
            after: Vec::new(),
        }
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
        Self {
            label: label.clone_box(),
            systems: Vec::new(),
        }
    }

    /// 添加系统到调度器
    ///
    /// # 参数
    ///
    /// - `system` - 系统包装器
    pub fn add_systems(&mut self, system: SystemWrapper) -> &mut Self {
        self.systems.push(system);
        self
    }

    /// 执行所有系统
    ///
    /// 根据 before/after 约束进行拓扑排序后依次执行。
    /// 若无约束则按添加顺序执行。
    /// 若检测到循环依赖则返回错误。
    pub fn run(&mut self, world: &mut gg_ecs::GgWorld) -> gg_error::GResult<()> {
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
