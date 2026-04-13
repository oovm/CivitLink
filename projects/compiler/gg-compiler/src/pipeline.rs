//! 编译流水线模块
//! 提供 DAG 拓扑排序和增量编译的流水线执行引擎

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};

use crate::{
    artifact::ArtifactSet,
    cache::CompilationCache,
    context::{BuildContext, Diagnostic, DiagnosticLevel},
    transformer::Transformer,
};

/// 流水线节点，包装 Transformer 及其依赖关系
pub struct PipelineNode {
    /// 节点唯一标识
    pub id: String,
    /// 转换器
    pub transformer: Box<dyn Transformer>,
    /// 依赖的节点 ID 列表
    pub dependencies: Vec<String>,
    /// 上次执行的产物哈希缓存（用于增量编译）
    pub last_output_hash: Option<u64>,
    /// 缓存的输出产物（增量跳过时返回）
    pub cached_output: Option<ArtifactSet>,
}

/// 编译流水线，支持 DAG 拓扑排序和增量编译
pub struct Pipeline {
    /// 流水线节点列表
    nodes: HashMap<String, PipelineNode>,
    /// 节点执行顺序（拓扑排序结果）
    execution_order: Vec<String>,
    /// 是否需要重新计算执行顺序
    dirty: bool,
    /// 编译缓存
    cache: CompilationCache,
}

impl Pipeline {
    /// 创建一个空的编译流水线
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), execution_order: Vec::new(), dirty: false, cache: CompilationCache::new() }
    }

    /// 添加一个流水线节点
    pub fn add_node(&mut self, id: &str, transformer: Box<dyn Transformer>, dependencies: Vec<String>) {
        let node = PipelineNode { id: id.to_string(), transformer, dependencies, last_output_hash: None, cached_output: None };
        self.nodes.insert(id.to_string(), node);
        self.dirty = true;
    }

    /// 移除一个流水线节点
    pub fn remove_node(&mut self, id: &str) {
        if self.nodes.remove(id).is_some() {
            self.dirty = true;
        }
    }

    /// 计算拓扑排序，返回执行顺序
    ///
    /// 使用 Kahn 算法，若检测到循环依赖则返回错误
    pub fn topological_sort(&mut self) -> GResult<&[String]> {
        if !self.dirty {
            return Ok(&self.execution_order);
        }

        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for id in self.nodes.keys() {
            in_degree.insert(id.as_str(), 0);
            adjacency.insert(id.as_str(), Vec::new());
        }

        for (id, node) in &self.nodes {
            for dep in &node.dependencies {
                if !self.nodes.contains_key(dep.as_str()) {
                    return Err(GError {
                        kind: GErrorKind::Other,
                        message: format!("节点 '{}' 依赖的节点 '{}' 不存在", id, dep),
                    });
                }
                adjacency.get_mut(dep.as_str()).unwrap().push(id.as_str());
                *in_degree.get_mut(id.as_str()).unwrap() += 1;
            }
        }

        let mut queue: Vec<&str> = in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(id, _)| *id).collect();

        let mut sorted = Vec::with_capacity(self.nodes.len());

        while let Some(id) = queue.pop() {
            sorted.push(id.to_string());
            if let Some(neighbors) = adjacency.get(id) {
                for &neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }

        if sorted.len() != self.nodes.len() {
            return Err(GError { kind: GErrorKind::Other, message: "流水线中存在循环依赖".to_string() });
        }

        self.execution_order = sorted;
        self.dirty = false;
        Ok(&self.execution_order)
    }

    /// 执行所有转换器，支持增量编译
    ///
    /// 按拓扑排序顺序依次执行每个节点，
    /// 若某节点的输入哈希与上次执行时相同则跳过该节点
    pub fn execute(&mut self, initial_inputs: ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        self.topological_sort()?;

        let mut artifacts = initial_inputs;

        for node_id in self.execution_order.clone() {
            let output = self.execute_node(&node_id, &artifacts, context)?;
            artifacts.merge(output);
        }

        Ok(artifacts)
    }

    /// 执行单个节点，支持增量编译
    ///
    /// 计算节点输入的哈希值，若与上次执行时相同则跳过执行。
    /// 优先查询 CompilationCache，若缓存命中则直接反序列化返回结果。
    /// 当 Transformer 返回错误时，将错误包装为 Diagnostic 添加到 BuildContext 中
    pub fn execute_node(&mut self, node_id: &str, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let (input_keys, dependencies) = {
            let node = self
                .nodes
                .get(node_id)
                .ok_or_else(|| GError { kind: GErrorKind::Other, message: format!("节点 '{}' 不存在", node_id) })?;
            (node.transformer.input_keys(), node.dependencies.clone())
        };

        let input_hash = Self::compute_input_hash(inputs, &input_keys);

        if self.cache.is_valid(node_id, input_hash) {
            if let Some(data) = self.cache.get(node_id, input_hash) {
                if let Ok((output, _)) = bincode::serde::decode_from_slice(data, bincode::config::standard()) {
                    let node = self.nodes.get_mut(node_id).unwrap();
                    node.last_output_hash = Some(input_hash);
                    node.cached_output = Some(output.clone());
                    return Ok(output);
                }
            }
        }

        {
            let node = self.nodes.get(node_id).unwrap();
            if let Some(last_hash) = node.last_output_hash {
                if input_hash == last_hash {
                    if let Some(ref cached) = node.cached_output {
                        return Ok(cached.clone());
                    }
                    return Ok(ArtifactSet::new());
                }
            }
        }

        let transformer_name = {
            let node = self.nodes.get(node_id).unwrap();
            node.transformer.name().to_string()
        };

        let transform_result = {
            let node = self.nodes.get(node_id).unwrap();
            node.transformer.transform(inputs, context)
        };

        match transform_result {
            Ok(output) => {
                let node = self.nodes.get_mut(node_id).unwrap();
                node.last_output_hash = Some(input_hash);
                node.cached_output = Some(output.clone());

                if let Ok(data) = bincode::serde::encode_to_vec(&output, bincode::config::standard()) {
                    self.cache.insert(node_id, input_hash, data, dependencies);
                }

                Ok(output)
            }
            Err(err) => {
                context.diagnostics.push(Diagnostic {
                    level: DiagnosticLevel::Error,
                    source: transformer_name,
                    message: err.message.clone(),
                    error_code: None,
                    source_location: None,
                    suggestion: None,
                    code_snippet: None,
                });
                Err(err)
            }
        }
    }

    /// 设置编译缓存
    pub fn set_cache(&mut self, cache: CompilationCache) {
        self.cache = cache;
    }

    /// 获取编译缓存的不可变引用
    pub fn cache(&self) -> &CompilationCache {
        &self.cache
    }

    /// 获取编译缓存的可变引用
    pub fn cache_mut(&mut self) -> &mut CompilationCache {
        &mut self.cache
    }

    /// 将编译缓存持久化到磁盘
    pub fn persist_cache(&self, path: &std::path::Path) -> GResult<()> {
        self.cache.persist_to_disk(path)
    }

    /// 从磁盘加载编译缓存
    pub fn load_cache(&mut self, path: &std::path::Path) -> GResult<()> {
        self.cache.load_from_disk(path)
    }

    /// 使指定节点及所有下游节点的缓存失效
    pub fn invalidate_cache(&mut self, node_id: &str) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.cached_output = None;
            node.last_output_hash = None;
        }

        let downstream: Vec<String> =
            self.nodes.iter().filter(|(_, n)| n.dependencies.iter().any(|d| d == node_id)).map(|(id, _)| id.clone()).collect();

        for downstream_id in downstream {
            self.invalidate_cache(&downstream_id);
        }
    }

    /// 验证流水线配置的完整性
    ///
    /// 检查所有依赖节点 ID 是否存在，以及是否存在重复的节点 ID
    pub fn validate(&self) -> Result<(), String> {
        let mut seen_ids = std::collections::HashSet::new();
        for id in self.nodes.keys() {
            if !seen_ids.insert(id.clone()) {
                return Err(format!("存在重复的节点 ID: '{}'", id));
            }
        }

        for (id, node) in &self.nodes {
            for dep in &node.dependencies {
                if !self.nodes.contains_key(dep.as_str()) {
                    return Err(format!("节点 '{}' 依赖的节点 '{}' 不存在", id, dep));
                }
            }
        }

        Ok(())
    }

    /// 计算指定产物键对应的输入哈希
    fn compute_input_hash(inputs: &ArtifactSet, keys: &[crate::artifact::ArtifactKey]) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        let mut sorted_keys: Vec<&crate::artifact::ArtifactKey> = keys.iter().collect();
        sorted_keys.sort_by(|a, b| a.type_name.cmp(&b.type_name).then(a.id.cmp(&b.id)));

        for key in sorted_keys {
            key.hash(&mut hasher);
            if let Some(artifact) = inputs.get(key) {
                artifact.content_hash.hash(&mut hasher);
            }
        }

        hasher.finish()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        artifact::{ArtifactKey, ArtifactSet},
        context::BuildContext,
        transformer::Transformer,
    };
    use gg_core::GResult;

    use super::Pipeline;

    struct FailingTransformer;

    impl Transformer for FailingTransformer {
        fn name(&self) -> &str {
            "failing_transformer"
        }

        fn input_keys(&self) -> Vec<ArtifactKey> {
            Vec::new()
        }

        fn output_keys(&self) -> Vec<ArtifactKey> {
            Vec::new()
        }

        fn transform(&self, _inputs: &ArtifactSet, _context: &mut BuildContext) -> GResult<ArtifactSet> {
            Err(gg_core::GError::new("transform failed"))
        }
    }

    struct OkTransformer;

    impl Transformer for OkTransformer {
        fn name(&self) -> &str {
            "ok_transformer"
        }

        fn input_keys(&self) -> Vec<ArtifactKey> {
            Vec::new()
        }

        fn output_keys(&self) -> Vec<ArtifactKey> {
            Vec::new()
        }

        fn transform(&self, _inputs: &ArtifactSet, _context: &mut BuildContext) -> GResult<ArtifactSet> {
            Ok(ArtifactSet::new())
        }
    }

    #[test]
    fn test_error_propagation_from_transformer() {
        let mut pipeline = Pipeline::new();
        pipeline.add_node("fail", Box::new(FailingTransformer), Vec::new());

        let mut context = BuildContext::new(crate::context::BuildConfig::new());
        let result = pipeline.execute(ArtifactSet::new(), &mut context);

        assert!(result.is_err());
        assert!(context.has_errors());
        assert_eq!(context.diagnostics.len(), 1);
        assert_eq!(context.diagnostics[0].source, "failing_transformer");
        assert_eq!(context.diagnostics[0].message, "transform failed");
    }

    #[test]
    fn test_validate_with_missing_dependency() {
        let mut pipeline = Pipeline::new();
        pipeline.add_node("a", Box::new(OkTransformer), vec!["nonexistent".to_string()]);

        let result = pipeline.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent"));
    }

    #[test]
    fn test_validate_with_duplicate_node_id() {
        let mut pipeline = Pipeline::new();
        pipeline.add_node("dup", Box::new(OkTransformer), Vec::new());
        pipeline.add_node("dup", Box::new(OkTransformer), Vec::new());

        let result = pipeline.validate();
        assert!(result.is_ok());
    }
}
