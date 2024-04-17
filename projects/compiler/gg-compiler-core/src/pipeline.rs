//! 编译流水线模块
//! 提供 DAG 拓扑排序和增量编译的流水线执行引擎

use std::collections::HashMap;

use gg_core::{GError, GErrorKind, GResult};

use crate::{artifact::ArtifactSet, context::BuildContext, transformer::Transformer};

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
}

/// 编译流水线，支持 DAG 拓扑排序和增量编译
pub struct Pipeline {
    /// 流水线节点列表
    nodes: HashMap<String, PipelineNode>,
    /// 节点执行顺序（拓扑排序结果）
    execution_order: Vec<String>,
    /// 是否需要重新计算执行顺序
    dirty: bool,
}

impl Pipeline {
    /// 创建一个空的编译流水线
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), execution_order: Vec::new(), dirty: false }
    }

    /// 添加一个流水线节点
    pub fn add_node(&mut self, id: &str, transformer: Box<dyn Transformer>, dependencies: Vec<String>) {
        let node = PipelineNode { id: id.to_string(), transformer, dependencies, last_output_hash: None };
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
    /// 计算节点输入的哈希值，若与上次执行时相同则跳过执行
    pub fn execute_node(&mut self, node_id: &str, inputs: &ArtifactSet, context: &mut BuildContext) -> GResult<ArtifactSet> {
        let node = self
            .nodes
            .get(node_id)
            .ok_or_else(|| GError { kind: GErrorKind::Other, message: format!("节点 '{}' 不存在", node_id) })?;

        let input_keys = node.transformer.input_keys();

        let input_hash = Self::compute_input_hash(inputs, &input_keys);

        if let Some(last_hash) = node.last_output_hash {
            if input_hash == last_hash {
                return Ok(ArtifactSet::new());
            }
        }

        let output = node.transformer.transform(inputs, context)?;

        let node = self.nodes.get_mut(node_id).unwrap();
        node.last_output_hash = Some(input_hash);

        Ok(output)
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
