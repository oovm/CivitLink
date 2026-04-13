//! 增量构建器模块
//! 提供基于 SHA1 内容哈希的增量构建支持，自动收集需要重新构建的资产集合，
//! 支持磁盘持久化和级联重建编排

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{cache::AssetCache, dependency_graph::DependencyGraph, registry::AssetRegistry, types::Guid};

/// 增量重建结果
#[derive(Debug, Clone)]
pub struct RebuildResult {
    /// 需要重建的资产总数
    pub total_to_rebuild: usize,
    /// 成功重建的资产数
    pub rebuilt: usize,
    /// 重建失败的资产数
    pub failed: usize,
    /// 跳过的资产数（无需重建）
    pub skipped: usize,
    /// 错误列表
    pub errors: Vec<(Guid, String)>,
}

/// 增量构建器，跟踪文件哈希变化并确定需要重新构建的资产
#[derive(Debug, Serialize, Deserialize)]
pub struct IncrementalBuilder {
    /// 已知文件 SHA1 哈希映射
    file_hashes: HashMap<Guid, String>,
    /// 资产缓存
    pub cache: AssetCache,
    /// 依赖图
    dependency_graph: DependencyGraph,
}

impl IncrementalBuilder {
    /// 创建一个新的增量构建器
    pub fn new(cache: AssetCache, dependency_graph: DependencyGraph) -> Self {
        Self { file_hashes: HashMap::new(), cache, dependency_graph }
    }

    /// 检查指定资产是否需要重新构建
    pub fn needs_rebuild(&self, guid: &Guid, current_hash: &str) -> bool {
        match self.file_hashes.get(guid) {
            Some(stored_hash) => stored_hash != current_hash,
            None => true,
        }
    }

    /// 标记指定资产已完成构建并记录其哈希值
    pub fn mark_rebuilt(&mut self, guid: Guid, hash: String) {
        self.file_hashes.insert(guid, hash);
    }

    /// 收集需要重新构建的资产集合，包括传递依赖的受影响资产
    ///
    /// 对于每个发生变更的资产，通过依赖图的 `invalidate_dependents()` 方法
    /// 查找所有传递性下游依赖，返回变更资产与受影响资产的并集
    pub fn collect_rebuild_set(&self, changed: &[Guid]) -> HashSet<Guid> {
        let mut rebuild_set: HashSet<Guid> = HashSet::new();

        for guid in changed {
            rebuild_set.insert(guid.clone());
            let dependents = self.dependency_graph.invalidate_dependents(guid);
            rebuild_set.extend(dependents);
        }

        rebuild_set
    }

    /// 计算数据的 SHA1 哈希值，返回十六进制字符串
    pub fn compute_hash(data: &[u8]) -> String {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// 将增量构建状态持久化到磁盘
    pub fn persist_to_disk(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// 从磁盘加载增量构建状态
    pub fn load_from_disk(
        path: &Path,
        cache: AssetCache,
        dependency_graph: DependencyGraph,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let mut builder: IncrementalBuilder = serde_json::from_str(&json)?;
        builder.cache = cache;
        builder.dependency_graph = dependency_graph;
        Ok(builder)
    }

    /// 基于依赖图计算受影响资产的拓扑排序重建计划
    ///
    /// 首先通过 `collect_rebuild_set()` 收集所有需要重建的资产，
    /// 然后对依赖图执行拓扑排序，最后过滤出仅包含重建集合中资产的有序列表
    pub fn rebuild_plan(&self, changed_guids: &[Guid], dependency_graph: &DependencyGraph) -> Vec<Guid> {
        let rebuild_set = self.collect_rebuild_set(changed_guids);

        let topo_order = match dependency_graph.topological_sort() {
            Ok(order) => order,
            Err(_) => rebuild_set.iter().cloned().collect(),
        };

        topo_order.into_iter().filter(|guid| rebuild_set.contains(guid)).collect()
    }

    /// 按拓扑排序顺序依次触发各资产的重新编译
    ///
    /// 接受变更资产列表、资产注册表可变引用、依赖图引用和编译回调函数，
    /// 按照拓扑排序的顺序依次编译每个需要重建的资产，
    /// 返回包含成功、失败和跳过统计的重建结果
    pub fn execute_rebuild<F>(
        &mut self,
        changed_guids: &[Guid],
        registry: &mut AssetRegistry,
        dependency_graph: &DependencyGraph,
        mut compile_fn: F,
    ) -> RebuildResult
    where
        F: FnMut(&Guid, &mut AssetRegistry) -> Result<(), String>,
    {
        let plan = self.rebuild_plan(changed_guids, dependency_graph);
        let total_to_rebuild = plan.len();
        let mut rebuilt = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut errors: Vec<(Guid, String)> = Vec::new();

        for guid in &plan {
            let current_hash = registry
                .get_by_guid(guid)
                .map(|entry| std::fs::read(&entry.path).map(|data| Self::compute_hash(&data)).unwrap_or_default())
                .unwrap_or_default();

            if !self.needs_rebuild(guid, &current_hash) {
                skipped += 1;
                continue;
            }

            match compile_fn(guid, registry) {
                Ok(()) => {
                    self.mark_rebuilt(guid.clone(), current_hash);
                    rebuilt += 1;
                }
                Err(e) => {
                    errors.push((guid.clone(), e));
                    failed += 1;
                }
            }
        }

        RebuildResult { total_to_rebuild, rebuilt, failed, skipped, errors }
    }
}
