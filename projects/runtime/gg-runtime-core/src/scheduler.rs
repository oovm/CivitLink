//! 阶段驱动的系统调度器
//!
//! 实现按阶段顺序执行系统的调度器，支持系统排序约束、
//! 系统集合配置和拓扑排序。

use crate::stage::{Stage, SystemDescriptor, SystemFn, SystemSetId};
use gg_core::{GError, GErrorKind, GResult};
use gg_ecs::World;
use std::collections::{HashMap, HashSet};

/// 系统构建器，用于配置系统排序约束
///
/// 当构建器被丢弃时，系统描述符会自动插入到调度器中。
/// 通过链式调用 `before` 和 `after` 方法指定排序约束。
pub struct SystemBuilder<'a> {
    /// 调度器引用
    scheduler: &'a mut StageScheduler,
    /// 待插入的系统描述符
    descriptor: Option<SystemDescriptor>,
}

impl<'a> SystemBuilder<'a> {
    /// 指定当前系统必须在名为 `before_name` 的系统之前执行
    pub fn before(mut self, before_name: impl Into<String>) -> Self {
        if let Some(ref mut desc) = self.descriptor {
            desc.before.push(before_name.into());
        }
        self
    }

    /// 指定当前系统必须在名为 `after_name` 的系统之后执行
    pub fn after(mut self, after_name: impl Into<String>) -> Self {
        if let Some(ref mut desc) = self.descriptor {
            desc.after.push(after_name.into());
        }
        self
    }

    /// 指定当前系统所属的系统集合
    pub fn in_set(mut self, set: SystemSetId) -> Self {
        if let Some(ref mut desc) = self.descriptor {
            desc.system_set = Some(set);
        }
        self
    }
}

impl<'a> Drop for SystemBuilder<'a> {
    fn drop(&mut self) {
        if let Some(descriptor) = self.descriptor.take() {
            let stage = descriptor.stage;
            self.scheduler.systems.entry(stage).or_default().push(descriptor);
        }
    }
}

/// 系统集合配置
struct SetConfig {
    /// 该集合中的系统在以下系统之前执行
    before: Vec<String>,
    /// 该集合中的系统在以下系统之后执行
    after: Vec<String>,
}

/// 系统集合配置构建器
///
/// 当构建器被丢弃时，配置会自动保存到调度器中。
pub struct SetConfigBuilder<'a> {
    /// 调度器引用
    scheduler: &'a mut StageScheduler,
    /// 集合标识符
    set_id: SystemSetId,
    /// 前置约束列表
    before: Vec<String>,
    /// 后置约束列表
    after: Vec<String>,
}

impl<'a> SetConfigBuilder<'a> {
    /// 指定该集合中的系统在名为 `before_name` 的系统之前执行
    pub fn before(mut self, before_name: impl Into<String>) -> Self {
        self.before.push(before_name.into());
        self
    }

    /// 指定该集合中的系统在名为 `after_name` 的系统之后执行
    pub fn after(mut self, after_name: impl Into<String>) -> Self {
        self.after.push(after_name.into());
        self
    }
}

impl<'a> Drop for SetConfigBuilder<'a> {
    fn drop(&mut self) {
        let config = SetConfig { before: std::mem::take(&mut self.before), after: std::mem::take(&mut self.after) };
        self.scheduler.set_configs.insert(self.set_id.clone(), config);
    }
}

/// 阶段调度器
///
/// 按阶段顺序执行系统，支持系统排序约束和系统集合配置。
/// Startup 阶段仅在首次 tick 时执行一次，Exit 阶段在 stop 时执行。
pub struct StageScheduler {
    /// 按阶段分组的系统描述符
    systems: HashMap<Stage, Vec<SystemDescriptor>>,
    /// 系统集合配置
    set_configs: HashMap<SystemSetId, SetConfig>,
    /// 启动阶段是否已执行
    startup_executed: bool,
}

impl StageScheduler {
    /// 创建新的阶段调度器
    pub fn new() -> Self {
        Self { systems: HashMap::new(), set_configs: HashMap::new(), startup_executed: false }
    }

    /// 添加系统到默认阶段（Update）
    ///
    /// 返回系统构建器，可链式调用 `before`/`after` 指定排序约束。
    /// 构建器被丢弃时系统自动插入。
    pub fn add_system(&mut self, name: impl Into<String>, system: SystemFn) -> SystemBuilder<'_> {
        self.add_system_to_stage(name, system, Stage::Update)
    }

    /// 添加系统到指定阶段
    ///
    /// 返回系统构建器，可链式调用 `before`/`after` 指定排序约束。
    /// 构建器被丢弃时系统自动插入。
    pub fn add_system_to_stage(&mut self, name: impl Into<String>, system: SystemFn, stage: Stage) -> SystemBuilder<'_> {
        let descriptor = SystemDescriptor::new(name, system, stage);
        SystemBuilder { scheduler: self, descriptor: Some(descriptor) }
    }

    /// 配置系统集合的排序约束
    ///
    /// 返回集合配置构建器，可链式调用 `before`/`after` 指定集合级别的排序约束。
    /// 构建器被丢弃时配置自动保存。
    pub fn configure_set(&mut self, set: SystemSetId) -> SetConfigBuilder<'_> {
        SetConfigBuilder { scheduler: self, set_id: set, before: Vec::new(), after: Vec::new() }
    }

    /// 执行一帧，按阶段顺序运行系统
    ///
    /// 首次调用时执行顺序为：Startup → PreUpdate → Update → PostUpdate → Render，
    /// 后续调用跳过 Startup 阶段。
    pub fn tick(&mut self, world: &mut World) -> GResult<()> {
        let stages: Vec<Stage> = if !self.startup_executed {
            self.startup_executed = true;
            vec![Stage::Startup, Stage::PreUpdate, Stage::Update, Stage::PostUpdate, Stage::Render]
        }
        else {
            vec![Stage::PreUpdate, Stage::Update, Stage::PostUpdate, Stage::Render]
        };

        for stage in stages {
            self.run_stage(stage, world)?;
        }

        Ok(())
    }

    /// 停止调度器，执行退出阶段系统
    pub fn stop(&mut self, world: &mut World) -> GResult<()> {
        self.run_stage(Stage::Exit, world)
    }

    /// 获取指定阶段的系统数量
    pub fn system_count(&self, stage: Stage) -> usize {
        self.systems.get(&stage).map_or(0, |v| v.len())
    }

    /// 检查启动阶段是否已执行
    pub fn is_startup_executed(&self) -> bool {
        self.startup_executed
    }

    /// 执行指定阶段的所有系统
    fn run_stage(&mut self, stage: Stage, world: &mut World) -> GResult<()> {
        let has_systems = self.systems.get(&stage).map_or(false, |v| !v.is_empty());
        if !has_systems {
            return Ok(());
        }

        let sorted_indices = {
            let systems = self.systems.get(&stage).unwrap();
            self.topological_sort(systems)?
        };

        let systems = self.systems.get_mut(&stage).unwrap();
        for idx in sorted_indices {
            let system = &mut systems[idx];
            (system.system)(world).map_err(|e| GError {
                kind: GErrorKind::Runtime,
                message: format!("System '{}' failed: {}", system.name, e),
            })?;
        }

        Ok(())
    }

    /// 对系统进行拓扑排序
    ///
    /// 根据系统的 `before`/`after` 约束构建依赖图，
    /// 使用 Kahn 算法进行拓扑排序。检测到循环依赖时返回错误。
    fn topological_sort(&self, systems: &[SystemDescriptor]) -> GResult<Vec<usize>> {
        let n = systems.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let name_to_idx: HashMap<&str, usize> = systems.iter().enumerate().map(|(i, s)| (s.name.as_str(), i)).collect();

        let mut adj: Vec<HashSet<usize>> = vec![HashSet::new(); n];
        let mut in_degree = vec![0usize; n];

        for (i, sys) in systems.iter().enumerate() {
            for after_name in &sys.after {
                if let Some(&j) = name_to_idx.get(after_name.as_str()) {
                    if adj[j].insert(i) {
                        in_degree[i] += 1;
                    }
                }
            }

            for before_name in &sys.before {
                if let Some(&j) = name_to_idx.get(before_name.as_str()) {
                    if adj[i].insert(j) {
                        in_degree[j] += 1;
                    }
                }
            }

            if let Some(ref set_id) = sys.system_set {
                if let Some(config) = self.set_configs.get(set_id) {
                    for after_name in &config.after {
                        if let Some(&j) = name_to_idx.get(after_name.as_str()) {
                            if adj[j].insert(i) {
                                in_degree[i] += 1;
                            }
                        }
                    }
                    for before_name in &config.before {
                        if let Some(&j) = name_to_idx.get(before_name.as_str()) {
                            if adj[i].insert(j) {
                                in_degree[j] += 1;
                            }
                        }
                    }
                }
            }
        }

        let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut result = Vec::with_capacity(n);

        while let Some(node) = queue.pop() {
            result.push(node);
            for &neighbor in &adj[node] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push(neighbor);
                }
            }
        }

        if result.len() != n {
            return Err(GError {
                kind: GErrorKind::Runtime,
                message: "Circular dependency detected in system ordering".to_string(),
            });
        }

        Ok(result)
    }
}

impl Default for StageScheduler {
    fn default() -> Self {
        Self::new()
    }
}
