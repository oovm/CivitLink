//! 配置表依赖图模块
//! 追踪表间引用关系，支持依赖感知的增量编译

use std::collections::{HashMap, HashSet};

use crate::{schema::SheetTable, types::SheetType};

/// 依赖排序错误
#[derive(Debug, Clone)]
pub enum DependencySortError {
    /// 循环依赖错误
    CyclicDependency {
        /// 参与循环的表名列表
        cycle: Vec<String>,
    },
}

impl std::fmt::Display for DependencySortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencySortError::CyclicDependency { cycle } => {
                write!(f, "循环依赖: {}", cycle.join(" -> "))
            }
        }
    }
}

impl std::error::Error for DependencySortError {}

/// 配置表依赖图
///
/// 追踪表间的引用关系，用于增量编译时确定哪些表需要重编译。
/// 当被依赖的表发生变更时，依赖它的表也需要重编译。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SheetDependencyGraph {
    /// 表名到其依赖的表名集合的映射
    /// 例如：Equipment → {Item} 表示 Equipment 依赖 Item
    pub(crate) dependencies: HashMap<String, HashSet<String>>,
}

impl SheetDependencyGraph {
    /// 创建空的依赖图
    pub fn new() -> Self {
        Self::default()
    }

    /// 从表格列表构建依赖图
    ///
    /// 扫描每张表的 Reference 类型字段，提取引用关系
    pub fn build_from_tables(tables: &[SheetTable]) -> Self {
        let mut graph = Self::new();

        for table in tables {
            let mut deps = HashSet::new();
            for header in &table.headers {
                if let SheetType::Reference(ref_table) = &header.typing {
                    if ref_table != &table.name {
                        deps.insert(ref_table.clone());
                    }
                }
            }
            if !deps.is_empty() {
                graph.dependencies.insert(table.name.clone(), deps);
            }
        }

        graph
    }

    /// 获取指定表直接依赖的所有表名
    pub fn dependencies_of(&self, table_name: &str) -> HashSet<String> {
        self.dependencies.get(table_name).cloned().unwrap_or_default()
    }

    /// 获取指定表被哪些表依赖（反向依赖）
    pub fn dependents_of(&self, table_name: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        for (name, deps) in &self.dependencies {
            if deps.contains(table_name) {
                dependents.insert(name.clone());
            }
        }
        dependents
    }

    /// 获取所有受变更表影响的表（包括间接依赖）
    ///
    /// 递归查找所有依赖变更表的表，返回需要重编译的表名集合
    pub fn affected_tables(&self, changed_tables: &HashSet<String>) -> HashSet<String> {
        let mut affected = HashSet::new();
        let mut queue: Vec<String> = changed_tables.iter().cloned().collect();

        while let Some(table_name) = queue.pop() {
            let dependents = self.dependents_of(&table_name);
            for dep in dependents {
                if !affected.contains(&dep) {
                    affected.insert(dep.clone());
                    queue.push(dep);
                }
            }
        }

        affected
    }

    /// 获取所有表名
    pub fn all_tables(&self) -> HashSet<String> {
        let mut tables = HashSet::new();
        for name in self.dependencies.keys() {
            tables.insert(name.clone());
        }
        for deps in self.dependencies.values() {
            for dep in deps {
                tables.insert(dep.clone());
            }
        }
        tables
    }

    /// 对所有表进行拓扑排序
    ///
    /// 使用 Kahn 算法，确保被依赖的表排在前面。
    /// 如果检测到循环依赖，返回错误。
    pub fn topological_sort(&self) -> Result<Vec<String>, DependencySortError> {
        let all = self.all_tables();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();

        for table in &all {
            in_degree.entry(table.clone()).or_insert(0);
            adjacency.entry(table.clone()).or_default();
        }

        for (table, deps) in &self.dependencies {
            for dep in deps {
                if all.contains(dep) {
                    *in_degree.entry(table.clone()).or_insert(0) += 1;
                    adjacency.entry(dep.clone()).or_default().insert(table.clone());
                }
            }
        }

        let mut queue: Vec<String> = in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(name, _)| name.clone()).collect();
        queue.sort();

        let mut result: Vec<String> = Vec::new();

        while let Some(table) = queue.pop() {
            result.push(table.clone());

            if let Some(neighbors) = adjacency.get(&table) {
                let mut ready: Vec<String> = Vec::new();
                for neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            ready.push(neighbor.clone());
                        }
                    }
                }
                ready.sort();
                queue.extend(ready);
            }
        }

        if result.len() != all.len() {
            let cycle: Vec<String> = all.difference(&result.iter().cloned().collect()).cloned().collect();
            return Err(DependencySortError::CyclicDependency { cycle });
        }

        Ok(result)
    }
}
