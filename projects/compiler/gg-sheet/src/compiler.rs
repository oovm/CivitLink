//! 配置表编译管线模块
//! 提供完整的编译管线：扫描目录 → 解析文件 → 检测类型 → 生成代码

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    codegen::{self, CodegenConfig},
    config::SheetConfig,
    error::{SheetError, SheetResult},
    merge::merge_tables,
    reader::load_table,
    schema::{SheetTable, TableKind},
};

/// 配置表编译器
pub struct SheetCompiler {
    /// 配置表目录
    sheet_dir: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
    /// 文件内容哈希缓存（用于增量编译）
    file_hashes: HashMap<PathBuf, u64>,
}

impl SheetCompiler {
    /// 创建新的配置表编译器
    pub fn new(sheet_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self { sheet_dir: sheet_dir.into(), output_dir: output_dir.into(), file_hashes: HashMap::new() }
    }

    /// 从配置创建编译器
    pub fn from_config(config: &SheetConfig) -> Self {
        Self {
            sheet_dir: PathBuf::from(&config.sheet_dir),
            output_dir: PathBuf::from(&config.output_dir),
            file_hashes: HashMap::new(),
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
                Some("xlsx") | Some("xls") | Some("csv") | Some("tsv") => {
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

        let config = CodegenConfig::new(self.output_dir.clone());
        let code = codegen::generate_table(&table, &config)?;

        let output_path = self.output_path(&table.name);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SheetError::Io {
                    path: parent.to_path_buf(), message: format!("无法创建输出目录: {}", e)
                })?;
        }

        std::fs::write(&output_path, &code)
            .map_err(|e| SheetError::Io { path: output_path.clone(), message: format!("无法写入输出文件: {}", e) })?;

        self.file_hashes.insert(path.to_path_buf(), hash);
        Ok(())
    }

    /// 执行全量编译
    pub fn compile(&mut self) -> SheetResult<()> {
        let files = self.scan()?;

        let mut raw_tables: Vec<SheetTable> = Vec::new();
        for path in &files {
            let raw_table = load_table(path)?;
            let table = SheetTable::from_raw(raw_table)?;
            raw_tables.push(table);
        }

        let merged = merge_tables(&raw_tables)?;

        let tables: Vec<SheetTable> = merged;
        let sorted_tables = sort_by_dependency(&tables);

        for table in &sorted_tables {
            let config = CodegenConfig::new(self.output_dir.clone());
            let code = codegen::generate_table(table, &config)?;

            let output_path = self.output_path(&table.name);
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SheetError::Io {
                        path: parent.to_path_buf(), message: format!("无法创建输出目录: {}", e)
                    })?;
            }

            std::fs::write(&output_path, &code)
                .map_err(|e| SheetError::Io {
                    path: output_path.clone(), message: format!("无法写入输出文件: {}", e)
                })?;
        }

        Ok(())
    }

    /// 获取输出文件路径
    fn output_path(&self, table_name: &str) -> PathBuf {
        self.output_dir.join(format!("{}Table.v", table_name))
    }
}

/// 按依赖关系排序表格，枚举表优先编译
fn sort_by_dependency(tables: &[SheetTable]) -> Vec<SheetTable> {
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

/// 计算文件内容的哈希值
fn compute_hash(data: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
