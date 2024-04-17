//! 依赖图模块
//! 提供有向无环图（DAG）结构的依赖关系管理，支持环检测和拓扑排序

use std::collections::{HashMap, HashSet, VecDeque};

use gg_core::{GError, GErrorKind, GResult};
use serde::{Deserialize, Serialize};

use crate::types::Guid;

/// 依赖图，维护资产之间的有向依赖关系，保证无环
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// 正向邻接表：源资产 -> 依赖的目标资产列表
    adjacency: HashMap<Guid, Vec<Guid>>,
    /// 反向邻接表：目标资产 -> 被依赖的源资产列表
    reverse_adjacency: HashMap<Guid, Vec<Guid>>,
}

impl DependencyGraph {
    /// 创建一个空的依赖图
    pub fn new() -> Self {
        Self { adjacency: HashMap::new(), reverse_adjacency: HashMap::new() }
    }

    /// 添加一条依赖边，如果会形成环则返回错误
    pub fn add_dependency(&mut self, source: Guid, target: Guid) -> GResult<()> {
        if source == target {
            return Err(GError::with_kind(GErrorKind::Asset, &format!("Self-dependency detected: {}", source)));
        }

        self.adjacency.entry(target.clone()).or_default();
        self.reverse_adjacency.entry(source.clone()).or_default();

        if self.detect_cycle_from_with_edge(&source, &target) {
            let cycle_path = self.find_cycle_path(&source, &target).unwrap_or_else(|| vec![target.clone(), source.clone()]);
            let cycle_str = cycle_path.iter().map(|g| g.as_str()).collect::<Vec<_>>().join(" -> ");
            return Err(GError::with_kind(
                GErrorKind::Asset,
                &format!("Circular dependency detected: {} -> {}", cycle_str, source),
            ));
        }

        self.adjacency.entry(source.clone()).or_default().push(target.clone());
        self.reverse_adjacency.entry(target).or_default().push(source);

        Ok(())
    }

    /// 移除一条依赖边
    pub fn remove_dependency(&mut self, source: &Guid, target: &Guid) {
        if let Some(deps) = self.adjacency.get_mut(source) {
            deps.retain(|g| g != target);
        }
        if let Some(deps) = self.reverse_adjacency.get_mut(target) {
            deps.retain(|g| g != source);
        }
    }

    /// 更新依赖边，将源资产的旧依赖目标替换为新依赖目标
    ///
    /// 如果旧依赖边不存在则返回错误，如果新依赖边会形成环也返回错误
    pub fn update_dependency(&mut self, source: Guid, old_target: Guid, new_target: Guid) -> GResult<()> {
        let deps = self.adjacency.get(&source).map(|v| v.as_slice()).unwrap_or(&[]);
        if !deps.contains(&old_target) {
            return Err(GError::with_kind(
                GErrorKind::Asset,
                &format!("Dependency {} -> {} does not exist", source, old_target),
            ));
        }

        self.remove_dependency(&source, &old_target);

        if source == new_target {
            self.adjacency.entry(source.clone()).or_default().push(old_target.clone());
            self.reverse_adjacency.entry(old_target).or_default().push(source.clone());
            return Err(GError::with_kind(GErrorKind::Asset, &format!("Self-dependency detected: {}", source)));
        }

        self.adjacency.entry(new_target.clone()).or_default();
        self.reverse_adjacency.entry(source.clone()).or_default();

        if self.detect_cycle_from_with_edge(&source, &new_target) {
            let cycle_path =
                self.find_cycle_path(&source, &new_target).unwrap_or_else(|| vec![new_target.clone(), source.clone()]);
            let cycle_str = cycle_path.iter().map(|g| g.as_str()).collect::<Vec<_>>().join(" -> ");
            self.adjacency.entry(source.clone()).or_default().push(old_target.clone());
            self.reverse_adjacency.entry(old_target).or_default().push(source.clone());
            return Err(GError::with_kind(
                GErrorKind::Asset,
                &format!("Circular dependency detected: {} -> {}", cycle_str, source),
            ));
        }

        self.adjacency.entry(source.clone()).or_default().push(new_target.clone());
        self.reverse_adjacency.entry(new_target).or_default().push(source);

        Ok(())
    }

    /// 获取指定资产直接依赖的所有目标资产
    pub fn dependencies_of(&self, guid: &Guid) -> &[Guid] {
        self.adjacency.get(guid).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 获取直接依赖指定资产的所有源资产
    pub fn dependents_of(&self, guid: &Guid) -> &[Guid] {
        self.reverse_adjacency.get(guid).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// 收集指定资产的所有传递性下游依赖（被依赖者），用于级联失效
    ///
    /// 返回包含所有受影响资产 GUID 的集合（不包含起始资产本身）
    pub fn invalidate_dependents(&self, guid: &Guid) -> HashSet<Guid> {
        let mut result = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(dependents) = self.reverse_adjacency.get(guid) {
            for dep in dependents {
                if result.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = self.reverse_adjacency.get(&current) {
                for dep in dependents {
                    if result.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        result
    }

    /// 对所有资产执行拓扑排序，如果存在环则返回错误
    pub fn topological_sort(&self) -> GResult<Vec<Guid>> {
        if self.has_cycle() {
            return Err(GError::with_kind(
                GErrorKind::Asset,
                "Cannot perform topological sort: cycle detected in dependency graph",
            ));
        }

        let mut in_degree: HashMap<&Guid, usize> = HashMap::new();
        for guid in self.adjacency.keys() {
            in_degree.entry(guid).or_insert(0);
        }
        for guid in self.reverse_adjacency.keys() {
            in_degree.entry(guid).or_insert(0);
        }
        for (_, deps) in &self.adjacency {
            for dep in deps {
                *in_degree.entry(dep).or_insert(0) += 1;
            }
        }

        let mut queue: Vec<&Guid> = in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(g, _)| *g).collect();

        let mut result = Vec::new();
        while let Some(guid) = queue.pop() {
            result.push(guid.clone());
            if let Some(deps) = self.adjacency.get(guid) {
                for dep in deps {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(dep);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// 检查依赖图中是否存在环
    pub fn has_cycle(&self) -> bool {
        let all_guids: HashSet<&Guid> = self.adjacency.keys().chain(self.reverse_adjacency.keys()).collect();

        let mut visited: HashSet<&Guid> = HashSet::new();
        let mut rec_stack: HashSet<&Guid> = HashSet::new();

        for guid in &all_guids {
            if !visited.contains(guid) {
                if Self::dfs_cycle(guid, &self.adjacency, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// 从指定节点开始检测是否存在环（深度优先搜索）
    fn detect_cycle_from_with_edge(&self, source: &Guid, target: &Guid) -> bool {
        let mut visited: HashSet<&Guid> = HashSet::new();
        let mut stack: Vec<&Guid> = vec![target];

        while let Some(current) = stack.pop() {
            if current == source {
                return true;
            }
            if visited.contains(current) {
                continue;
            }
            visited.insert(current);
            if let Some(deps) = self.adjacency.get(current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        stack.push(dep);
                    }
                }
            }
        }

        false
    }

    /// 从 target 开始沿正向邻接表搜索回到 source 的路径
    fn find_cycle_path(&self, source: &Guid, target: &Guid) -> Option<Vec<Guid>> {
        let mut visited: HashSet<&Guid> = HashSet::new();
        let mut path: Vec<Guid> = vec![target.clone()];

        if self.dfs_find_cycle(source, target, &mut visited, &mut path) { Some(path) } else { None }
    }

    /// DFS 辅助方法，从 current 开始搜索回到 source 的路径
    fn dfs_find_cycle<'a>(
        &'a self,
        source: &Guid,
        current: &'a Guid,
        visited: &mut HashSet<&'a Guid>,
        path: &mut Vec<Guid>,
    ) -> bool {
        if current == source && path.len() > 1 {
            return true;
        }

        visited.insert(current);

        if let Some(deps) = self.adjacency.get(current) {
            for dep in deps {
                if dep == source {
                    path.push(dep.clone());
                    return true;
                }
                if !visited.contains(dep) {
                    path.push(dep.clone());
                    if self.dfs_find_cycle(source, dep, visited, path) {
                        return true;
                    }
                    path.pop();
                }
            }
        }

        false
    }

    /// DFS 辅助方法，检测从指定节点开始的环
    fn dfs_cycle<'a>(
        guid: &'a Guid,
        adjacency: &'a HashMap<Guid, Vec<Guid>>,
        visited: &mut HashSet<&'a Guid>,
        rec_stack: &mut HashSet<&'a Guid>,
    ) -> bool {
        visited.insert(guid);
        rec_stack.insert(guid);

        if let Some(deps) = adjacency.get(guid) {
            for dep in deps {
                if !visited.contains(dep) {
                    if Self::dfs_cycle(dep, adjacency, visited, rec_stack) {
                        return true;
                    }
                }
                else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(guid);
        false
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DependencyGraph {
    fn clone(&self) -> Self {
        Self { adjacency: self.adjacency.clone(), reverse_adjacency: self.reverse_adjacency.clone() }
    }
}
