//! 配置表结构定义模块
//! 定义表头、表格类型和结构化表格数据

use crate::error::{SheetError, SheetResult};
use crate::reader::RawTable;
use crate::types::SheetType;

/// 字段约束类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldConstraint {
    /// 无约束
    None,
    /// 唯一约束（@field）
    Unique,
    /// 主键约束（@@field）
    Primary,
}

/// 配置表字段头信息
#[derive(Debug, Clone)]
pub struct SheetHeader {
    /// 列索引（从 0 开始）
    pub column: usize,
    /// 字段名
    pub field_name: String,
    /// 字段类型
    pub typing: SheetType,
    /// 字段注释
    pub comment: String,
    /// 字段约束
    pub constraint: FieldConstraint,
}

/// 表格类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableKind {
    /// 列表表（首列字段名为 id，数字键）
    List,
    /// 字典表（首列字段名为 key，字符串键）
    Dict,
    /// 枚举表（首列字段名为 enum）
    Enumerate,
}

impl TableKind {
    /// 根据首列字段名自动检测表格类型
    pub fn detect(first_field_name: &str) -> TableKind {
        let name = first_field_name.trim().trim_start_matches('@');
        match name {
            "id" => TableKind::List,
            "key" => TableKind::Dict,
            "enum" => TableKind::Enumerate,
            _ => TableKind::List,
        }
    }
}

/// 一行数据，以字段名到字符串值的映射表示
pub type DataRow = Vec<String>;

/// 结构化表格数据
#[derive(Debug, Clone)]
pub struct SheetTable {
    /// 表格名称
    pub name: String,
    /// 表格类型
    pub kind: TableKind,
    /// 表头列表
    pub headers: Vec<SheetHeader>,
    /// 数据行列表
    pub rows: Vec<DataRow>,
}

impl SheetTable {
    /// 从原始表格数据构建结构化表格
    pub fn from_raw(raw: RawTable) -> SheetResult<SheetTable> {
        if raw.header_row.is_empty() {
            return Err(SheetError::Parse {
                path: raw.path.clone(),
                line: 1,
                message: "表头行为空".to_string(),
            });
        }

        let first_field = parse_field_name(&raw.header_row[0]);
        let kind = TableKind::detect(&first_field.name);

        let mut headers = Vec::new();
        let col_count = raw.header_row.len();

        for col in 0..col_count {
            let field_str = raw.header_row.get(col).map(|s| s.as_str()).unwrap_or("");
            let type_str = raw.type_row.get(col).map(|s| s.as_str()).unwrap_or("");
            let comment = raw.comment_row.get(col).map(|s| s.as_str()).unwrap_or("");

            let parsed_field = parse_field_name(field_str);
            let typing = SheetType::parse(type_str).map_err(|_| SheetError::Type {
                path: raw.path.clone(),
                column: col,
                type_str: type_str.to_string(),
            })?;

            headers.push(SheetHeader {
                column: col,
                field_name: parsed_field.name,
                typing,
                comment: comment.to_string(),
                constraint: parsed_field.constraint,
            });
        }

        Ok(SheetTable {
            name: raw.name,
            kind,
            headers,
            rows: raw.data_rows,
        })
    }

    /// 获取主键字段的索引，若无主键则返回 0
    pub fn primary_key_index(&self) -> usize {
        self.headers
            .iter()
            .find(|h| h.constraint == FieldConstraint::Primary)
            .map(|h| h.column)
            .unwrap_or(0)
    }

    /// 获取枚举表的枚举名列索引（enum 列）
    pub fn enum_name_column(&self) -> Option<usize> {
        if self.kind != TableKind::Enumerate {
            return None;
        }
        self.headers.iter().find(|h| h.field_name == "enum").map(|h| h.column)
    }

    /// 获取枚举表的 ID 列索引
    pub fn enum_id_column(&self) -> Option<usize> {
        if self.kind != TableKind::Enumerate {
            return None;
        }
        self.headers.iter().find(|h| h.field_name == "id").map(|h| h.column)
    }
}

/// 解析后的字段名信息
struct ParsedFieldName {
    /// 字段名
    name: String,
    /// 约束类型
    constraint: FieldConstraint,
}

/// 解析字段名中的约束标记
fn parse_field_name(field_str: &str) -> ParsedFieldName {
    let trimmed = field_str.trim();
    if trimmed.starts_with("@@") {
        ParsedFieldName {
            name: trimmed[2..].to_string(),
            constraint: FieldConstraint::Primary,
        }
    } else if trimmed.starts_with('@') {
        ParsedFieldName {
            name: trimmed[1..].to_string(),
            constraint: FieldConstraint::Unique,
        }
    } else {
        ParsedFieldName {
            name: trimmed.to_string(),
            constraint: FieldConstraint::None,
        }
    }
}
