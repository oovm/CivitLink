//! 配置表编译管线模块
//! 提供完整的编译管线：扫描目录 → 解析文件 → 检测类型 → 生成代码

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    cache::SheetCache,
    codegen::{self, CodegenConfig},
    config::SheetConfig,
    dependency::SheetDependencyGraph,
    error::{SheetError, SheetResult},
    merge::merge_tables,
    reader::load_table,
    schema::{SheetTable, TableKind},
    von_codegen,
};

/// 配置表编译器
pub struct SheetCompiler {
    /// 配置表目录
    sheet_dir: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
    /// 文件内容哈希缓存（用于增量编译）
    file_hashes: HashMap<PathBuf, u64>,
    /// 表间依赖图
    dependency_graph: SheetDependencyGraph,
}

impl SheetCompiler {
    /// 创建新的配置表编译器
    pub fn new(sheet_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            sheet_dir: sheet_dir.into(),
            output_dir: output_dir.into(),
            file_hashes: HashMap::new(),
            dependency_graph: SheetDependencyGraph::new(),
        }
    }

    /// 从配置创建编译器
    pub fn from_config(config: &SheetConfig) -> Self {
        Self {
            sheet_dir: PathBuf::from(&config.sheet_dir),
            output_dir: PathBuf::from(&config.output_dir),
            file_hashes: HashMap::new(),
            dependency_graph: SheetDependencyGraph::new(),
        }
    }

    /// 获取配置表目录路径
    pub fn sheet_dir(&self) -> &Path {
        &self.sheet_dir
    }

    /// 扫描配置表目录，返回所有支持的配置表文件路径
    pub fn scan(&self) -> SheetResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        let entries = std::fs::read_dir(&self.sheet_dir)
            .map_err(|e| SheetError::Io { path: self.sheet_dir.clone(), message: format!("无法读取目录: {}", e) })?;

        for entry in entries {
            let entry = entry
                .map_err(|e| SheetError::Io {
                    path: self.sheet_dir.clone(), message: format!("读取目录条目失败: {}", e)
                })?;

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());
            match ext.as_deref() {
                Some("xlsx") | Some("xls") | Some("csv") | Some("tsv") | Some("json") => {
                    files.push(path);
                }
                _ => {}
            }
        }

        files.sort();
        Ok(files)
    }

    /// 编译单个配置表文件
    pub fn compile_file(&mut self, path: &Path) -> SheetResult<()> {
        let content = std::fs::read(path)
            .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法读取文件: {}", e) })?;

        let hash = compute_hash(&content);
        if let Some(&prev_hash) = self.file_hashes.get(path) {
            if prev_hash == hash {
                return Ok(());
            }
        }

        let raw_table = load_table(path)?;
        let table = SheetTable::from_raw(raw_table)?;

        self.write_table_files(&table)?;

        self.file_hashes.insert(path.to_path_buf(), hash);
        Ok(())
    }

    /// 执行全量编译
    pub fn compile(&mut self) -> SheetResult<()> {
        let files = self.scan()?;

        let mut raw_tables: Vec<SheetTable> = Vec::new();
        for path in &files {
            let content = std::fs::read(path)
                .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法读取文件: {}", e) })?;
            let hash = compute_hash(&content);
            self.file_hashes.insert(path.clone(), hash);

            let raw_table = load_table(path)?;
            let table = SheetTable::from_raw(raw_table)?;
            raw_tables.push(table);
        }

        let merged = merge_tables(&raw_tables)?;

        let tables: Vec<SheetTable> = merged;
        let sorted_tables = sort_by_dependency(&tables);

        self.dependency_graph = SheetDependencyGraph::build_from_tables(&sorted_tables);

        for table in &sorted_tables {
            self.write_table_files(table)?;
        }

        let mut cache = SheetCache::new();
        cache.file_hashes = self.file_hashes.clone();
        cache.dependency_graph = self.dependency_graph.clone();
        cache.save(&self.output_dir).map_err(|e| SheetError::Io {
            path: self.output_dir.join(".sheet-cache"),
            message: format!("无法保存缓存文件: {}", e),
        })?;

        Ok(())
    }

    /// 执行增量编译
    ///
    /// 仅重编译发生变更的表及其依赖表。
    /// 首次编译时执行全量编译，后续编译时通过文件哈希和依赖图确定重编译范围。
    pub fn incremental_compile(&mut self) -> SheetResult<()> {
        let cache = SheetCache::load(&self.output_dir);
        self.file_hashes = cache.file_hashes.clone();
        self.dependency_graph = cache.dependency_graph.clone();

        let files = self.scan()?;

        let mut changed_tables: HashSet<String> = HashSet::new();
        let mut _unchanged_raw_tables: Vec<(PathBuf, u64)> = Vec::new();

        for path in &files {
            let content = std::fs::read(path)
                .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法读取文件: {}", e) })?;
            let hash = compute_hash(&content);

            if cache.is_file_changed(path, hash) {
                let table_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
                changed_tables.insert(table_name);
            }
            else {
                _unchanged_raw_tables.push((path.clone(), hash));
            }
        }

        if changed_tables.is_empty() {
            return Ok(());
        }

        let affected = self.dependency_graph.affected_tables(&changed_tables);
        let tables_to_recompile: HashSet<String> = changed_tables.union(&affected).cloned().collect();

        let mut raw_tables: Vec<SheetTable> = Vec::new();
        for path in &files {
            let table_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();
            if tables_to_recompile.contains(&table_name) {
                let raw_table = load_table(path)?;
                let table = SheetTable::from_raw(raw_table)?;
                raw_tables.push(table);
            }
        }

        if raw_tables.is_empty() {
            return Ok(());
        }

        let merged = merge_tables(&raw_tables)?;
        let sorted_tables = sort_by_dependency(&merged);

        for table in &sorted_tables {
            self.write_table_files(table)?;
        }

        self.dependency_graph = SheetDependencyGraph::build_from_tables(&sorted_tables);

        for path in &files {
            let content = std::fs::read(path)
                .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法读取文件: {}", e) })?;
            let hash = compute_hash(&content);
            self.file_hashes.insert(path.clone(), hash);
        }

        let mut new_cache = SheetCache::new();
        new_cache.file_hashes = self.file_hashes.clone();
        new_cache.dependency_graph = self.dependency_graph.clone();
        new_cache.save(&self.output_dir).map_err(|e| SheetError::Io {
            path: self.output_dir.join(".sheet-cache"),
            message: format!("无法保存缓存文件: {}", e),
        })?;

        Ok(())
    }

    /// 获取输出文件路径
    fn output_path(&self, table_name: &str) -> PathBuf {
        self.output_dir.join(format!("{}Table.script", table_name))
    }

    /// 获取 VON 数据文件输出路径
    fn von_output_path(&self, table_name: &str) -> PathBuf {
        self.output_dir.join(format!("{}Table.von", table_name))
    }

    /// 将表格同时输出为 .script 和 .von 文件
    fn write_table_files(&self, table: &SheetTable) -> SheetResult<()> {
        let config = CodegenConfig::new(self.output_dir.clone());
        let script_code = codegen::generate_table(table, &config)?;
        let von_data = von_codegen::generate_von_data(table)?;

        let script_path = self.output_path(&table.name);
        let von_path = self.von_output_path(&table.name);

        if let Some(parent) = script_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SheetError::Io {
                    path: parent.to_path_buf(), message: format!("无法创建输出目录: {}", e)
                })?;
        }

        std::fs::write(&script_path, &script_code)
            .map_err(|e| SheetError::Io { path: script_path.clone(), message: format!("无法写入脚本文件: {}", e) })?;

        std::fs::write(&von_path, &von_data)
            .map_err(|e| SheetError::Io { path: von_path.clone(), message: format!("无法写入数据文件: {}", e) })?;

        Ok(())
    }
}

/// 按依赖关系排序表格，使用拓扑排序确保被依赖表先编译
fn sort_by_dependency(tables: &[SheetTable]) -> Vec<SheetTable> {
    let graph = SheetDependencyGraph::build_from_tables(tables);

    let table_map: HashMap<String, SheetTable> = tables.iter().map(|t| (t.name.clone(), t.clone())).collect();

    match graph.topological_sort() {
        Ok(order) => {
            let mut sorted: Vec<SheetTable> = Vec::new();
            for name in &order {
                if let Some(table) = table_map.get(name) {
                    sorted.push(table.clone());
                }
            }
            for table in tables {
                if !sorted.iter().any(|t| t.name == table.name) {
                    sorted.push(table.clone());
                }
            }
            sorted
        }
        Err(_) => {
            let mut enum_tables: Vec<SheetTable> = Vec::new();
            let mut other_tables: Vec<SheetTable> = Vec::new();

            for table in tables {
                if table.kind == TableKind::Enumerate {
                    enum_tables.push(table.clone());
                }
                else {
                    other_tables.push(table.clone());
                }
            }

            enum_tables.extend(other_tables);
            enum_tables
        }
    }
}

/// 计算文件内容的哈希值
fn compute_hash(data: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
