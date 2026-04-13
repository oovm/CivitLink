//! 资产转换器模块
//! 实现 Transformer trait，将元数据源文件解析并注册到资产注册表和依赖图中

use std::path::PathBuf;

use gg_compiler::{
    artifact::{Artifact, ArtifactKey, ArtifactSet},
    context::BuildContext,
    transformer::Transformer,
};
use gg_core::{GError, GErrorKind, GResult};
use gg_meta::MetaFile;

use crate::{
    dependency_graph::DependencyGraph,
    registry::AssetRegistry,
    types::{AssetEntry, AssetType},
};

/// 元数据源产物类型名称
const META_SOURCE_TYPE: &str = "meta_source";

/// 资产注册表产物类型名称
const ASSET_REGISTRY_TYPE: &str = "asset_registry";

/// 依赖图产物类型名称
const DEPENDENCY_GRAPH_TYPE: &str = "dependency_graph";

/// 资产转换器，解析元数据源文件并构建资产注册表和依赖图
pub struct AssetTransformer;

impl AssetTransformer {
    /// 创建一个新的资产转换器
    pub fn new() -> Self {
        Self
    }
}

impl Default for AssetTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for AssetTransformer {
    fn name(&self) -> &str {
        "asset"
    }

    fn input_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(META_SOURCE_TYPE, "*")]
    }

    fn output_keys(&self) -> Vec<ArtifactKey> {
        vec![ArtifactKey::new(ASSET_REGISTRY_TYPE, "*"), ArtifactKey::new(DEPENDENCY_GRAPH_TYPE, "*")]
    }

    fn transform(&self, inputs: &ArtifactSet, _context: &mut BuildContext) -> GResult<ArtifactSet> {
        let mut registry = AssetRegistry::new();
        let mut dep_graph = DependencyGraph::new();

        for key in inputs.keys() {
            if key.type_name != META_SOURCE_TYPE {
                continue;
            }

            let artifact = inputs.get(&key).ok_or_else(|| GError {
                kind: GErrorKind::Asset,
                message: format!("Artifact not found for key: {}/{}", key.type_name, key.id),
            })?;

            let meta: MetaFile = serde_json::from_slice(&artifact.data)
                .map_err(|e| GError::with_kind(GErrorKind::Asset, &format!("Failed to parse meta file: {}", e)))?;

            let asset_type = AssetType::from_str(&meta.asset.r#type);
            let guid = meta.asset.guid.clone();

            let mut dependencies = Vec::new();
            for dep in &meta.dependencies {
                dependencies.push(dep.guid.clone());
                if let Err(e) = dep_graph.add_dependency(guid.clone(), dep.guid.clone()) {
                    return Err(e);
                }
            }

            let mut references = Vec::new();
            for ref_ in &meta.references {
                references.push(ref_.path.clone());
            }

            let entry = AssetEntry {
                guid: guid.clone(),
                path: PathBuf::from(&meta.asset.path),
                asset_type,
                hash: meta.hash.unwrap_or_default(),
                dependencies,
                references,
                compiled_artifact: None,
            };

            registry.register(entry);
        }

        let mut output = ArtifactSet::new();

        let registry_data = serde_json::to_vec(&registry)
            .map_err(|e| GError::with_kind(GErrorKind::Asset, &format!("Failed to serialize asset registry: {}", e)))?;
        output.insert(Artifact::new(ArtifactKey::new(ASSET_REGISTRY_TYPE, "default"), registry_data));

        let dep_graph_data = serde_json::to_vec(&dep_graph)
            .map_err(|e| GError::with_kind(GErrorKind::Asset, &format!("Failed to serialize dependency graph: {}", e)))?;
        output.insert(Artifact::new(ArtifactKey::new(DEPENDENCY_GRAPH_TYPE, "default"), dep_graph_data));

        Ok(output)
    }
}
