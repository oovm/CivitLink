//! 配置表文件读取模块
//! 支持 Excel(.xlsx)、CSV、TSV 三种格式的文件读取

use std::path::{Path, PathBuf};

use calamine::Reader;
use crate::error::{SheetError, SheetResult};

/// 文件格式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileFormat {
    /// Excel 格式 (.xlsx)
    Xlsx,
    /// CSV 格式 (.csv)
    Csv,
    /// TSV 格式 (.tsv)
    Tsv,
}

impl FileFormat {
    /// 根据文件扩展名检测文件格式
    pub fn from_extension(path: &Path) -> SheetResult<FileFormat> {
        match path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref() {
            Some("xlsx") | Some("xls") => Ok(FileFormat::Xlsx),
            Some("csv") => Ok(FileFormat::Csv),
            Some("tsv") => Ok(FileFormat::Tsv),
            _ => Err(SheetError::Io {
                path: path.to_path_buf(),
                message: format!("不支持的文件格式: {}", path.display()),
            }),
        }
    }
}

/// 原始表格数据，统一三种格式的读取结果
#[derive(Debug, Clone)]
pub struct RawTable {
    /// 表格名称（文件名不含扩展名）
    pub name: String,
    /// 文件路径
    pub path: PathBuf,
    /// 表头行（字段名）
    pub header_row: Vec<String>,
    /// 类型行
    pub type_row: Vec<String>,
    /// 注释行（可选）
    pub comment_row: Vec<String>,
    /// 数据行
    pub data_rows: Vec<Vec<String>>,
}

/// 表格读取器 trait
pub trait TableReader {
    /// 从文件加载表格数据
    fn load(path: &Path) -> SheetResult<RawTable>;
}

/// Excel 文件读取器
pub struct ExcelReader;

impl TableReader for ExcelReader {
    fn load(path: &Path) -> SheetResult<RawTable> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut workbook = calamine::open_workbook_auto(path).map_err(|e| SheetError::Io {
            path: path.to_path_buf(),
            message: format!("无法打开 Excel 文件: {}", e),
        })?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names.first().ok_or_else(|| SheetError::Io {
            path: path.to_path_buf(),
            message: "Excel 文件不包含任何工作表".to_string(),
        })?;

        let range = workbook.worksheet_range(sheet_name).map_err(|e| SheetError::Io {
            path: path.to_path_buf(),
            message: format!("无法读取工作表: {}", e),
        })?;

        let mut all_rows: Vec<Vec<String>> = Vec::new();
        for row in range.rows() {
            let cells: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
            all_rows.push(cells);
        }

        if all_rows.len() < 2 {
            return Err(SheetError::Parse {
                path: path.to_path_buf(),
                line: 0,
                message: "Excel 文件至少需要包含字段名行和类型行".to_string(),
            });
        }

        let col_count = all_rows[0].len();
        let header_row = all_rows[0].clone();
        let type_row = if all_rows.len() > 1 { all_rows[1].clone() } else { vec![String::new(); col_count] };
        let comment_row = if all_rows.len() > 2 { all_rows[2].clone() } else { vec![String::new(); col_count] };
        let data_rows = if all_rows.len() > 3 { all_rows[3..].to_vec() } else { Vec::new() };

        Ok(RawTable {
            name,
            path: path.to_path_buf(),
            header_row,
            type_row,
            comment_row,
            data_rows,
        })
    }
}

/// CSV/TSV 文件读取器
pub struct CsvReader;

impl CsvReader {
    /// 创建 CSV 读取器
    pub fn new() -> Self {
        Self
    }

    /// 创建 TSV 读取器
    pub fn new_tsv() -> Self {
        Self
    }
}

impl Default for CsvReader {
    fn default() -> Self {
        Self::new()
    }
}

impl TableReader for CsvReader {
    fn load(path: &Path) -> SheetResult<RawTable> {
        let format = FileFormat::from_extension(path)?;
        let is_tsv = format == FileFormat::Tsv;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let content = std::fs::read_to_string(path).map_err(|e| SheetError::Io {
            path: path.to_path_buf(),
            message: format!("无法读取文件: {}", e),
        })?;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(if is_tsv { b'\t' } else { b',' })
            .from_reader(content.as_bytes());

        let mut all_rows: Vec<Vec<String>> = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| SheetError::Parse {
                path: path.to_path_buf(),
                line: all_rows.len() + 1,
                message: format!("CSV 解析错误: {}", e),
            })?;
            let cells: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            all_rows.push(cells);
        }

        if all_rows.len() < 2 {
            return Err(SheetError::Parse {
                path: path.to_path_buf(),
                line: 0,
                message: "CSV 文件至少需要包含字段名行和类型行".to_string(),
            });
        }

        let col_count = all_rows[0].len();
        let header_row = all_rows[0].clone();
        let type_row = if all_rows.len() > 1 { all_rows[1].clone() } else { vec![String::new(); col_count] };
        let comment_row = if all_rows.len() > 2 { all_rows[2].clone() } else { vec![String::new(); col_count] };
        let data_rows = if all_rows.len() > 3 { all_rows[3..].to_vec() } else { Vec::new() };

        Ok(RawTable {
            name,
            path: path.to_path_buf(),
            header_row,
            type_row,
            comment_row,
            data_rows,
        })
    }
}

/// 根据文件格式自动选择读取器加载表格
pub fn load_table(path: &Path) -> SheetResult<RawTable> {
    let format = FileFormat::from_extension(path)?;
    match format {
        FileFormat::Xlsx => ExcelReader::load(path),
        FileFormat::Csv | FileFormat::Tsv => CsvReader::load(path),
    }
}
