//! 依赖管理系统
//! 
//! 提供任务依赖关系的管理和解析功能

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use gwg_agent::prelude::*;

/// 依赖图
pub struct DependencyGraph {
    /// 任务依赖关系
    dependencies: Arc<Mutex<HashMap<String, HashSet<String>>>>,
    /// 反向依赖关系（用于快速查找依赖某个任务的所有任务）
    reverse_dependencies: Arc<Mutex<HashMap<String, HashSet<String>>>>,
}

impl DependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(Mutex::new(HashMap::new())),
            reverse_dependencies: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加依赖关系
    pub fn add_dependency(&self, task_id: &str, dependency_id: &str) {
        let mut dependencies = self.dependencies.lock().unwrap();
        let mut reverse_dependencies = self.reverse_dependencies.lock().unwrap();

        // 添加正向依赖
        dependencies
            .entry(task_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(dependency_id.to_string());

        // 添加反向依赖
        reverse_dependencies
            .entry(dependency_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(task_id.to_string());
    }

    /// 移除依赖关系
    pub fn remove_dependency(&self, task_id: &str, dependency_id: &str) {
        let mut dependencies = self.dependencies.lock().unwrap();
        let mut reverse_dependencies = self.reverse_dependencies.lock().unwrap();

        // 移除正向依赖
        if let Some(deps) = dependencies.get_mut(task_id) {
            deps.remove(dependency_id);
        }

        // 移除反向依赖
        if let Some(reverse_deps) = reverse_dependencies.get_mut(dependency_id) {
            reverse_deps.remove(task_id);
        }
    }

    /// 获取任务的所有依赖
    pub fn get_dependencies(&self, task_id: &str) -> HashSet<String> {
        let dependencies = self.dependencies.lock().unwrap();
        dependencies.get(task_id).cloned().unwrap_or_default()
    }

    /// 获取依赖某个任务的所有任务
    pub fn get_dependents(&self, task_id: &str) -> HashSet<String> {
        let reverse_dependencies = self.reverse_dependencies.lock().unwrap();
        reverse_dependencies.get(task_id).cloned().unwrap_or_default()
    }

    /// 检查任务是否有依赖
    pub fn has_dependencies(&self, task_id: &str) -> bool {
        let dependencies = self.dependencies.lock().unwrap();
        dependencies.get(task_id).map(|deps| !deps.is_empty()).unwrap_or(false)
    }

    /// 检查任务是否被其他任务依赖
    pub fn has_dependents(&self, task_id: &str) -> bool {
        let reverse_dependencies = self.reverse_dependencies.lock().unwrap();
        reverse_dependencies.get(task_id).map(|deps| !deps.is_empty()).unwrap_or(false)
    }

    /// 拓扑排序
    pub fn topological_sort(&self, tasks: &[Task]) -> Result<Vec<String>, String> {
        let dependencies = self.dependencies.lock().unwrap();
        let mut in_degree = HashMap::new();
        let mut queue = Vec::new();
        let mut result = Vec::new();

        // 初始化入度
        for task in tasks {
            in_degree.insert(task.id.clone(), 0);
        }

        // 计算入度
        for task in tasks {
            if let Some(deps) = dependencies.get(&task.id) {
                for dep in deps {
                    if in_degree.contains_key(dep) {
                        *in_degree.get_mut(dep).unwrap() += 1;
                    }
                }
            }
        }

        // 将入度为0的任务加入队列
        for (task_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push(task_id.clone());
            }
        }

        // 执行拓扑排序
        while !queue.is_empty() {
            let task_id = queue.remove(0);
            result.push(task_id.clone());

            if let Some(deps) = dependencies.get(&task_id) {
                for dep in deps {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dep.clone());
                        }
                    }
                }
            }
        }

        // 检查是否存在环
        if result.len() != tasks.len() {
            return Err("Dependency cycle detected".to_string());
        }

        Ok(result)
    }

    /// 检查依赖关系是否有效
    pub fn validate_dependencies(&self, tasks: &[Task]) -> Result<(), String> {
        let dependencies = self.dependencies.lock().unwrap();
        let task_ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();

        // 检查所有依赖是否存在
        for (task_id, deps) in dependencies.iter() {
            for dep in deps {
                if !task_ids.contains(dep) {
                    return Err(format!("Task {} depends on non-existent task {}", task_id, dep));
                }
            }
        }

        // 检查是否存在环
        match self.topological_sort(tasks) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// 依赖管理器
pub struct DependencyManager {
    /// 依赖图
    graph: DependencyGraph,
    /// 任务状态跟踪
    task_status: Arc<Mutex<HashMap<String, TaskStatus>>>,
}

impl DependencyManager {
    /// 创建新的依赖管理器
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            task_status: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 添加依赖关系
    pub fn add_dependency(&self, task_id: &str, dependency_id: &str) {
        self.graph.add_dependency(task_id, dependency_id);
    }

    /// 移除依赖关系
    pub fn remove_dependency(&self, task_id: &str, dependency_id: &str) {
        self.graph.remove_dependency(task_id, dependency_id);
    }

    /// 更新任务状态
    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) {
        let mut task_status = self.task_status.lock().unwrap();
        task_status.insert(task_id.to_string(), status);
    }

    /// 检查任务是否可以执行（所有依赖都已完成）
    pub fn can_execute(&self, task_id: &str) -> bool {
        let dependencies = self.graph.get_dependencies(task_id);
        let task_status = self.task_status.lock().unwrap();

        dependencies.iter().all(|dep| {
            task_status.get(dep) == Some(&TaskStatus::Completed)
        })
    }

    /// 获取可执行的任务
    pub fn get_executable_tasks(&self, tasks: &[Task]) -> Vec<String> {
        tasks
            .iter()
            .filter(|task| {
                task.status == TaskStatus::Pending && self.can_execute(&task.id)
            })
            .map(|task| task.id.clone())
            .collect()
    }

    /// 拓扑排序
    pub fn topological_sort(&self, tasks: &[Task]) -> Result<Vec<String>, String> {
        self.graph.topological_sort(tasks)
    }

    /// 验证依赖关系
    pub fn validate_dependencies(&self, tasks: &[Task]) -> Result<(), String> {
        self.graph.validate_dependencies(tasks)
    }
}

/// 预导入模块
pub mod prelude {
    pub use super::{DependencyGraph, DependencyManager};
}