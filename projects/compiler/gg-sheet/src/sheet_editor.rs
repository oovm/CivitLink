//! 表格编辑器与编译器交互接口
//! 定义编辑器加载表格、保存表格、触发编译的接口

use std::path::{Path, PathBuf};

use crate::{compiler::SheetCompiler, error::SheetResult, reader::load_table, schema::SheetTable};

/// 表格信息摘要
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableInfo {
    /// 表格名称
    pub name: String,
    /// 行数
    pub row_count: usize,
    /// 列数
    pub column_count: usize,
}

/// 列头信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HeaderInfo {
    /// 字段名
    pub name: String,
    /// 类型名
    pub type_name: String,
    /// 注释
    pub comment: String,
}

/// 单元格信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CellInfo {
    /// 单元格值
    pub value: String,
    /// 类型名
    pub type_name: String,
}

/// 行数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RowData {
    /// 单元格列表
    pub cells: Vec<CellInfo>,
}

/// 表格完整数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableData {
    /// 表格名称
    pub name: String,
    /// 列头信息
    pub headers: Vec<HeaderInfo>,
    /// 行数据
    pub rows: Vec<RowData>,
}

/// 表格编辑器服务
///
/// 提供编辑器与编译器之间的交互接口，
/// 支持表格发现、加载、保存和编译触发。
pub struct SheetEditorService {
    /// 配置表目录
    sheet_dir: PathBuf,
    /// 输出目录
    output_dir: PathBuf,
}

impl SheetEditorService {
    /// 创建新的编辑器服务
    pub fn new(sheet_dir: &Path, output_dir: &Path) -> Self {
        Self { sheet_dir: sheet_dir.to_path_buf(), output_dir: output_dir.to_path_buf() }
    }

    /// 发现配置表目录中的所有表格
    pub fn discover_tables(&self) -> SheetResult<Vec<TableInfo>> {
        let compiler = SheetCompiler::new(&self.sheet_dir, &self.output_dir);
        let files = compiler.scan()?;
        let mut tables = Vec::new();

        for file_path in &files {
            let name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if !name.is_empty() {
                tables.push(TableInfo { name, row_count: 0, column_count: 0 });
            }
        }

        Ok(tables)
    }

    /// 加载指定表格的完整数据
    pub fn load_table(&self, table_name: &str) -> SheetResult<TableData> {
        let compiler = SheetCompiler::new(&self.sheet_dir, &self.output_dir);
        let files = compiler.scan()?;

        let target_file =
            files.iter().find(|f| f.file_stem().and_then(|s| s.to_str()).map(|s| s == table_name).unwrap_or(false));

        let file_path = target_file
            .ok_or_else(|| crate::error::SheetError::Config { message: format!("表格 '{}' 不存在", table_name) })?;

        let raw_table = load_table(file_path)?;
        let table = SheetTable::from_raw(raw_table)?;

        Ok(table_to_data(&table))
    }

    /// 触发增量编译
    pub fn trigger_compile(&self) -> SheetResult<()> {
        let mut compiler = SheetCompiler::new(&self.sheet_dir, &self.output_dir);
        compiler.incremental_compile()?;
        Ok(())
    }
}

/// 将 SheetTable 转换为 TableData
fn table_to_data(table: &SheetTable) -> TableData {
    let headers: Vec<HeaderInfo> = table
        .headers
        .iter()
        .map(|h| HeaderInfo { name: h.field_name.clone(), type_name: h.typing.to_valkyrie_type(), comment: h.comment.clone() })
        .collect();

    let rows: Vec<RowData> = table
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<CellInfo> = table
                .headers
                .iter()
                .map(|header| {
                    let value = row.get(header.column).map(|s| s.as_str()).unwrap_or("").to_string();
                    CellInfo { value, type_name: header.typing.to_valkyrie_type() }
                })
                .collect();
            RowData { cells }
        })
        .collect();

    TableData { name: table.name.clone(), headers, rows }
}
