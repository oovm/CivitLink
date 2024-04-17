//! 资产管线模块
//! 提供统一的资产编译调度入口，协调资产注册、依赖图构建、增量构建和各编译器模块的调用

use std::{collections::HashMap, fmt, path::Path, sync::mpsc};

use crate::{
    cache::AssetCache,
    dependency_graph::DependencyGraph,
    incremental::IncrementalBuilder,
    registry::AssetRegistry,
    types::{AssetEntry, AssetType},
};

/// 资产编译器 trait，供各编译器模块实现
pub trait AssetCompiler: Send + Sync {
    /// 编译器名称
    fn name(&self) -> &str;

    /// 支持的资产类型列表
    fn supported_types(&self) -> &[AssetType];

    /// 编译指定资产
    fn compile(&self, entry: &AssetEntry, context: &CompileContext) -> Result<(), String>;

    /// 克隆编译器为 Box 形式
    fn clone_box(&self) -> Box<dyn AssetCompiler>;
}

impl Clone for Box<dyn AssetCompiler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// 编译上下文，传递给编译器
pub struct CompileContext {
    /// 资产注册表
    pub registry: AssetRegistry,
    /// 依赖图
    pub dependency_graph: DependencyGraph,
    /// 资产缓存
    pub cache: AssetCache,
}

/// 构建结果
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// 总资产数
    pub total_assets: usize,
    /// 重新编译的资产数
    pub rebuilt_assets: usize,
    /// 跳过的资产数（缓存命中）
    pub skipped_assets: usize,
    /// 构建错误列表
    pub errors: Vec<String>,
}

impl fmt::Display for BuildResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Build complete: {} total, {} rebuilt, {} skipped, {} errors",
            self.total_assets,
            self.rebuilt_assets,
            self.skipped_assets,
            self.errors.len()
        )
    }
}

/// 异步构建消息，通过 channel 通知构建进度
#[derive(Debug, Clone)]
pub enum BuildMessage {
    /// 单个资产编译完成
    AssetCompiled {
        /// 资产 GUID
        guid: String,
        /// 资产类型
        asset_type: String,
        /// 是否为重新编译
        rebuilt: bool,
    },
    /// 单个资产编译失败
    AssetFailed {
        /// 资产 GUID
        guid: String,
        /// 错误信息
        error: String,
    },
    /// 构建完成
    Completed {
        /// 构建结果
        result: BuildResult,
    },
}

/// 资产管线，统一调度所有资产编译
pub struct AssetPipeline {
    /// 资产注册表
    registry: AssetRegistry,
    /// 依赖图
    dependency_graph: DependencyGraph,
    /// 增量构建器
    builder: IncrementalBuilder,
    /// 资产缓存
    cache: AssetCache,
    /// 资产类型到编译器的映射
    compilers: HashMap<String, Box<dyn AssetCompiler>>,
}

impl AssetPipeline {
    /// 创建一个新的资产管线
    pub fn new() -> Self {
        let cache = AssetCache::new();
        let dependency_graph = DependencyGraph::new();
        let builder = IncrementalBuilder::new(cache.clone_for_pipeline(), dependency_graph.clone());

        Self { registry: AssetRegistry::new(), dependency_graph, builder, cache, compilers: HashMap::new() }
    }

    /// 创建一个有缓存容量限制的资产管线
    pub fn with_cache_capacity(max_capacity: usize) -> Self {
        let cache = AssetCache::with_capacity(max_capacity);
        let dependency_graph = DependencyGraph::new();
        let builder = IncrementalBuilder::new(cache.clone_for_pipeline(), dependency_graph.clone());

        Self { registry: AssetRegistry::new(), dependency_graph, builder, cache, compilers: HashMap::new() }
    }

    /// 注册一个编译器，按其支持的资产类型建立映射
    pub fn register_compiler(&mut self, compiler: Box<dyn AssetCompiler>) {
        for asset_type in compiler.supported_types() {
            let type_key = asset_type.as_str().to_string();
            self.compilers.insert(type_key, compiler.clone_box());
        }
    }

    /// 注册所有默认编译器，覆盖全部 11 种资产格式
    ///
    /// 注册的编译器包括：
    /// - MetaAssetCompiler（Meta）
    /// - SchemaAssetCompiler（Schema）
    /// - ShaderAssetCompiler（Shader，使用 GG Shader 格式）
    /// - ScriptAssetCompiler（Script，使用 Valkyrie 格式）
    /// - WidgetAssetCompiler（Widget）
    /// - VonAssetCompiler（Animation/Config/Material/Prefab/Scene/VON）
    pub fn register_default_compilers(&mut self) {
        self.register_compiler(Box::new(crate::compilers::MetaAssetCompiler));
        self.register_compiler(Box::new(crate::compilers::SchemaAssetCompiler::new()));
        self.register_compiler(Box::new(crate::compilers::ShaderAssetCompiler::new()));
        self.register_compiler(Box::new(crate::compilers::ScriptAssetCompiler::new()));
        self.register_compiler(Box::new(crate::compilers::WidgetAssetCompiler::new()));
        self.register_compiler(Box::new(crate::compilers::VonAssetCompiler::new()));
    }

    /// 执行完整构建流程
    ///
    /// 完整流程：扫描资产目录 → 注册资产 → 依赖解析 → 按优先级排序 →
    /// 增量检测 → 编译 → 缓存更新 → 返回构建结果
    pub fn build(&mut self, asset_dir: &Path) -> Result<BuildResult, String> {
        let mut result = BuildResult { total_assets: 0, rebuilt_assets: 0, skipped_assets: 0, errors: Vec::new() };

        self.scan_and_register(asset_dir, &mut result)?;

        let topo_order = self.dependency_graph.topological_sort().map_err(|e| e.message)?;

        let mut sorted_assets: Vec<(String, AssetType)> = Vec::new();
        for guid in &topo_order {
            if let Some(entry) = self.registry.get_by_guid(guid) {
                sorted_assets.push((guid.clone(), entry.asset_type.clone()));
            }
        }

        sorted_assets.sort_by_key(|(_, asset_type)| asset_type.compile_priority());

        for (guid, asset_type) in &sorted_assets {
            if let Some(entry) = self.registry.get_by_guid(guid) {
                result.total_assets += 1;

                let file_data = std::fs::read(&entry.path).unwrap_or_default();
                let current_hash = IncrementalBuilder::compute_hash(&file_data);

                if self.builder.needs_rebuild(guid, &current_hash) {
                    if let Some(compiler) = self.compilers.get(asset_type.as_str()) {
                        let context = CompileContext {
                            registry: self.registry.clone_for_pipeline(),
                            dependency_graph: self.dependency_graph.clone(),
                            cache: self.cache.clone_for_pipeline(),
                        };

                        match compiler.compile(entry, &context) {
                            Ok(()) => {
                                self.builder.mark_rebuilt(guid.clone(), current_hash);
                                result.rebuilt_assets += 1;
                            }
                            Err(e) => {
                                result.errors.push(format!("Failed to compile asset {}: {}", guid, e));
                            }
                        }
                    }
                    else {
                        result.skipped_assets += 1;
                    }
                }
                else {
                    result.skipped_assets += 1;
                }
            }
        }

        Ok(result)
    }

    /// 异步执行构建流程，通过 channel 通知构建进度
    ///
    /// 构建过程在当前线程执行但通过 mpsc channel 发送进度消息，
    /// 调用者可在其他线程接收消息以实现非阻塞的进度监控
    pub fn build_async(&mut self, asset_dir: &Path) -> Result<mpsc::Receiver<BuildMessage>, String> {
        let (tx, rx) = mpsc::channel();

        let mut result = BuildResult { total_assets: 0, rebuilt_assets: 0, skipped_assets: 0, errors: Vec::new() };

        self.scan_and_register(asset_dir, &mut result)?;

        let topo_order = self.dependency_graph.topological_sort().map_err(|e| e.message)?;

        let mut sorted_assets: Vec<(String, AssetType)> = Vec::new();
        for guid in &topo_order {
            if let Some(entry) = self.registry.get_by_guid(guid) {
                sorted_assets.push((guid.clone(), entry.asset_type.clone()));
            }
        }

        sorted_assets.sort_by_key(|(_, asset_type)| asset_type.compile_priority());

        for (guid, asset_type) in &sorted_assets {
            if let Some(entry) = self.registry.get_by_guid(guid) {
                result.total_assets += 1;

                let file_data = std::fs::read(&entry.path).unwrap_or_default();
                let current_hash = IncrementalBuilder::compute_hash(&file_data);

                if self.builder.needs_rebuild(guid, &current_hash) {
                    if let Some(compiler) = self.compilers.get(asset_type.as_str()) {
                        let context = CompileContext {
                            registry: self.registry.clone_for_pipeline(),
                            dependency_graph: self.dependency_graph.clone(),
                            cache: self.cache.clone_for_pipeline(),
                        };

                        match compiler.compile(entry, &context) {
                            Ok(()) => {
                                self.builder.mark_rebuilt(guid.clone(), current_hash);
                                result.rebuilt_assets += 1;
                                let _ = tx.send(BuildMessage::AssetCompiled {
                                    guid: guid.clone(),
                                    asset_type: asset_type.as_str().to_string(),
                                    rebuilt: true,
                                });
                            }
                            Err(e) => {
                                result.errors.push(format!("Failed to compile asset {}: {}", guid, e));
                                let _ = tx.send(BuildMessage::AssetFailed { guid: guid.clone(), error: e });
                            }
                        }
                    }
                    else {
                        result.skipped_assets += 1;
                    }
                }
                else {
                    result.skipped_assets += 1;
                    let _ = tx.send(BuildMessage::AssetCompiled {
                        guid: guid.clone(),
                        asset_type: asset_type.as_str().to_string(),
                        rebuilt: false,
                    });
                }
            }
        }

        let _ = tx.send(BuildMessage::Completed { result });

        Ok(rx)
    }

    /// 扫描资产目录并注册所有 .meta 文件
    fn scan_and_register(&mut self, asset_dir: &Path, result: &mut BuildResult) -> Result<(), String> {
        if !asset_dir.exists() {
            return Err(format!("Asset directory does not exist: {}", asset_dir.display()));
        }

        let entries = std::fs::read_dir(asset_dir).map_err(|e| format!("Failed to read asset directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "meta") {
                if let Err(e) = self.register_meta_file(&path) {
                    result.errors.push(format!("Failed to register meta file {}: {}", path.display(), e));
                }
            }

            if path.is_dir() {
                self.scan_and_register(&path, result)?;
            }
        }

        Ok(())
    }

    /// 解析并注册单个 .meta 文件
    fn register_meta_file(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read meta file: {}", e))?;

        let meta: gg_meta::MetaFile = toml::from_str(&content).map_err(|e| format!("Failed to parse meta file: {}", e))?;

        let asset_type = AssetType::from_str(&meta.asset.r#type);
        let guid = meta.asset.guid.clone();

        let mut dependencies = Vec::new();
        for dep in &meta.dependencies {
            dependencies.push(dep.guid.clone());
            if let Err(e) = self.dependency_graph.add_dependency(guid.clone(), dep.guid.clone()) {
                return Err(e.message);
            }
        }

        let mut references = Vec::new();
        for ref_ in &meta.references {
            references.push(ref_.path.clone());
        }

        let entry = AssetEntry {
            guid: guid.clone(),
            path: path.to_path_buf(),
            asset_type,
            hash: meta.hash.unwrap_or_default(),
            dependencies,
            references,
            compiled_artifact: None,
        };

        self.registry.register(entry);
        Ok(())
    }

    /// 获取资产注册表的引用
    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }

    /// 获取依赖图的引用
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// 获取资产缓存的引用
    pub fn cache(&self) -> &AssetCache {
        &self.cache
    }
}

impl Default for AssetPipeline {
    fn default() -> Self {
        Self::new()
    }
}
