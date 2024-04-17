#![warn(missing_docs)]

//! 配置表数据验证模块
//! 提供类型检查、唯一约束检查和引用完整性检查功能

use std::{collections::HashMap, path::PathBuf};

use crate::{
    schema::{FieldConstraint, SheetTable},
    types::{SheetType, SheetValue},
};

/// 数据验证错误类型
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// 类型不匹配错误，单元格值与声明类型不符
    TypeMismatch {
        /// 表格文件路径
        path: PathBuf,
        /// 行索引（从 0 开始）
        row: usize,
        /// 列索引（从 0 开始）
        column: usize,
        /// 字段名
        field_name: String,
        /// 期望的类型
        expected: String,
        /// 实际的值
        actual: String,
    },
    /// 唯一约束违反错误，字段值出现重复
    UniqueConstraintViolation {
        /// 表格文件路径
        path: PathBuf,
        /// 字段名
        field_name: String,
        /// 重复的值
        value: String,
        /// 出现重复的行索引列表
        rows: Vec<usize>,
    },
    /// 引用完整性错误，引用的目标值不存在
    ReferenceIntegrity {
        /// 表格文件路径
        path: PathBuf,
        /// 行索引（从 0 开始）
        row: usize,
        /// 列索引（从 0 开始）
        column: usize,
        /// 字段名
        field_name: String,
        /// 引用的目标表名
        ref_table: String,
        /// 引用的目标值
        ref_value: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::TypeMismatch { path, row, column, field_name, expected, actual } => {
                write!(
                    f,
                    "类型不匹配 ({} 行{} 列{}): 字段 '{}' 期望类型 '{}', 实际值 '{}'",
                    path.display(),
                    row,
                    column,
                    field_name,
                    expected,
                    actual
                )
            }
            ValidationError::UniqueConstraintViolation { path, field_name, value, rows } => {
                write!(f, "唯一约束违反 ({}): 字段 '{}' 的值 '{}' 在行 {:?} 中重复", path.display(), field_name, value, rows)
            }
            ValidationError::ReferenceIntegrity { path, row, column, field_name, ref_table, ref_value } => {
                write!(
                    f,
                    "引用完整性错误 ({} 行{} 列{}): 字段 '{}' 引用表 '{}' 的值 '{}' 不存在",
                    path.display(),
                    row,
                    column,
                    field_name,
                    ref_table,
                    ref_value
                )
            }
        }
    }
}

/// 数据验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// 验证错误列表
    pub errors: Vec<ValidationError>,
    /// 验证警告列表
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// 判断验证是否通过（无错误）
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// 将验证报告格式化为可读字符串
    pub fn format_report(&self) -> String {
        let mut output = String::new();

        if self.errors.is_empty() && self.warnings.is_empty() {
            output.push_str("验证通过，无错误或警告。\n");
            return output;
        }

        if !self.errors.is_empty() {
            output.push_str(&format!("发现 {} 个错误:\n", self.errors.len()));
            for (i, err) in self.errors.iter().enumerate() {
                output.push_str(&format!("  [{}] {}\n", i + 1, err));
            }
        }

        if !self.warnings.is_empty() {
            output.push_str(&format!("发现 {} 个警告:\n", self.warnings.len()));
            for (i, warn) in self.warnings.iter().enumerate() {
                output.push_str(&format!("  [{}] {}\n", i + 1, warn));
            }
        }

        output
    }

    /// 合并另一个验证报告到当前报告
    fn merge(&mut self, other: ValidationReport) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        ValidationReport { errors: Vec::new(), warnings: Vec::new() }
    }
}

/// 验证单张配置表的数据
///
/// 执行类型检查、唯一约束检查和引用完整性检查
pub fn validate_table(table: &SheetTable) -> ValidationReport {
    let mut report = ValidationReport::default();
    let table_path = PathBuf::from(&table.name);

    check_types(table, &table_path, &mut report);
    check_unique_constraints(table, &table_path, &mut report);
    check_reference_integrity(table, &table_path, &mut report);

    report
}

/// 批量验证多张配置表的数据
pub fn validate_tables(tables: &[SheetTable]) -> ValidationReport {
    let mut report = ValidationReport::default();

    for table in tables {
        let sub_report = validate_table(table);
        report.merge(sub_report);
    }

    report
}

/// 类型检查：验证每个单元格值与声明类型是否匹配
fn check_types(table: &SheetTable, path: &PathBuf, report: &mut ValidationReport) {
    for (row_idx, row) in table.rows.iter().enumerate() {
        for header in &table.headers {
            let cell_value = row.get(header.column).map(|s| s.as_str()).unwrap_or("");

            if cell_value.is_empty() {
                continue;
            }

            if SheetValue::parse_from_str(cell_value, &header.typing).is_err() {
                report.errors.push(ValidationError::TypeMismatch {
                    path: path.clone(),
                    row: row_idx,
                    column: header.column,
                    field_name: header.field_name.clone(),
                    expected: format_type_name(&header.typing),
                    actual: cell_value.to_string(),
                });
            }
        }
    }
}

/// 唯一约束检查：验证标记为 Unique 或 Primary 的字段值是否唯一
fn check_unique_constraints(table: &SheetTable, path: &PathBuf, report: &mut ValidationReport) {
    for header in &table.headers {
        if header.constraint != FieldConstraint::Unique && header.constraint != FieldConstraint::Primary {
            continue;
        }

        let mut value_rows: HashMap<String, Vec<usize>> = HashMap::new();

        for (row_idx, row) in table.rows.iter().enumerate() {
            let cell_value = row.get(header.column).map(|s| s.trim().to_string()).unwrap_or_default();

            if cell_value.is_empty() {
                continue;
            }

            value_rows.entry(cell_value.clone()).or_default().push(row_idx);
        }

        for (value, rows) in value_rows {
            if rows.len() > 1 {
                report.errors.push(ValidationError::UniqueConstraintViolation {
                    path: path.clone(),
                    field_name: header.field_name.clone(),
                    value,
                    rows,
                });
            }
        }
    }
}

/// 引用完整性检查：检查 Reference 类型字段的值是否在目标表中存在
///
/// 当前仅记录警告，因为跨表引用验证需要加载所有表数据，留待后续完善
fn check_reference_integrity(table: &SheetTable, path: &PathBuf, report: &mut ValidationReport) {
    for header in &table.headers {
        if let SheetType::Reference(ref_table) = &header.typing {
            for (row_idx, row) in table.rows.iter().enumerate() {
                let cell_value = row.get(header.column).map(|s| s.trim().to_string()).unwrap_or_default();

                if cell_value.is_empty() {
                    continue;
                }

                report.warnings.push(format!(
                    "引用完整性检查暂未实现: 表 '{}' 字段 '{}' (行{}) 引用表 '{}' 的值 '{}'",
                    path.display(),
                    header.field_name,
                    row_idx,
                    ref_table,
                    cell_value
                ));
            }
        }
    }
}

/// 将 SheetType 格式化为可读的类型名称
fn format_type_name(ty: &SheetType) -> String {
    match ty {
        SheetType::Boolean => "bool".to_string(),
        SheetType::Integer(kind) => match kind {
            crate::types::IntegerKind::Integer8 => "i8".to_string(),
            crate::types::IntegerKind::Integer16 => "i16".to_string(),
            crate::types::IntegerKind::Integer32 => "i32".to_string(),
            crate::types::IntegerKind::Integer64 => "i64".to_string(),
            crate::types::IntegerKind::Unsigned8 => "u8".to_string(),
            crate::types::IntegerKind::Unsigned16 => "u16".to_string(),
            crate::types::IntegerKind::Unsigned32 => "u32".to_string(),
            crate::types::IntegerKind::Unsigned64 => "u64".to_string(),
        },
        SheetType::Decimal(kind) => match kind {
            crate::types::DecimalKind::Float32 => "f32".to_string(),
            crate::types::DecimalKind::Float64 => "f64".to_string(),
        },
        SheetType::String => "string".to_string(),
        SheetType::Color => "color".to_string(),
        SheetType::Vector(kind) => match kind {
            crate::types::VectorKind::Vec2 => "vec2".to_string(),
            crate::types::VectorKind::Vec3 => "vec3".to_string(),
            crate::types::VectorKind::Vec4 => "vec4".to_string(),
        },
        SheetType::List(inner) => format!("[{}]", format_type_name(inner)),
        SheetType::Map { key, value } => {
            format!("Map<{}, {}>", format_type_name(key), format_type_name(value))
        }
        SheetType::Optional(inner) => format!("{}?", format_type_name(inner)),
        SheetType::Reference(table) => format!("&{}", table),
        SheetType::Enumerate(name) => name.clone(),
    }
}
