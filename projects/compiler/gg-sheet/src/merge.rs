#![warn(missing_docs)]

//! 配置表合表模块
//! 按下划线前缀分组，检查结构一致性，合并同组表

use std::collections::HashMap;

use crate::{
    error::{SheetError, SheetResult},
    schema::{FieldConstraint, SheetTable},
};

/// 按下划线前缀对表格分组
///
/// 表名含下划线时，取下划线前的部分作为组名（如 `Item_Weapon` → `Item`）。
/// 表名不含下划线时不参与合表。仅含一个子表的组不参与合表。
pub fn group_by_prefix(tables: &[SheetTable]) -> HashMap<String, Vec<&SheetTable>> {
    let mut groups: HashMap<String, Vec<&SheetTable>> = HashMap::new();

    for table in tables {
        if let Some(prefix) = table.name.split('_').next() {
            if prefix != table.name {
                groups.entry(prefix.to_string()).or_default().push(table);
            }
        }
    }

    groups.retain(|_, v| {
        if v.len() < 2 {
            return false;
        }
        let first_kind = &v[0].kind;
        v.iter().all(|t| t.kind == *first_kind)
    });
    groups
}

/// 检查同组表结构一致性
///
/// 字段数量必须相同，字段名和类型必须一一对应。
/// 不一致时返回 `SheetError::Codegen` 错误，列出差异字段。
pub fn check_structure_consistency(tables: &[&SheetTable]) -> SheetResult<()> {
    if tables.len() < 2 {
        return Ok(());
    }

    let base = &tables[0];

    for table in tables.iter().skip(1) {
        if table.headers.len() != base.headers.len() {
            return Err(SheetError::Codegen {
                message: format!(
                    "表 '{}' 与 '{}' 字段数量不一致: {} vs {}",
                    table.name,
                    base.name,
                    table.headers.len(),
                    base.headers.len()
                ),
            });
        }

        let mut diffs = Vec::new();
        for (j, (h1, h2)) in base.headers.iter().zip(table.headers.iter()).enumerate() {
            if h1.field_name != h2.field_name || h1.typing != h2.typing {
                diffs.push(format!(
                    "列{}: '{}({})' vs '{}({})'",
                    j,
                    h1.field_name,
                    h1.typing.to_valkyrie_type(),
                    h2.field_name,
                    h2.typing.to_valkyrie_type()
                ));
            }
        }

        if !diffs.is_empty() {
            return Err(SheetError::Codegen {
                message: format!("表 '{}' 与 '{}' 结构不一致:\n{}", table.name, base.name, diffs.join("\n")),
            });
        }
    }

    Ok(())
}

/// 合并同组表
///
/// 使用第一个表的结构作为基准，合并所有表的数据行。
/// 检测主键冲突（Primary 约束字段值重复时报错）。
/// 合并后的表名为组名（如 `Item`）。
pub fn merge_table_group(name: &str, tables: &[&SheetTable]) -> SheetResult<SheetTable> {
    let base = tables[0];

    let mut merged_rows: Vec<Vec<String>> = Vec::new();
    let mut seen_keys: HashMap<String, usize> = HashMap::new();

    let pk_index = base.primary_key_index();
    let pk_header = &base.headers[pk_index];
    let has_primary = pk_header.constraint == FieldConstraint::Primary;

    for table in tables {
        for row in &table.rows {
            if has_primary {
                let key = row.get(pk_index).map(|s| s.as_str()).unwrap_or("").trim().to_string();
                if !key.is_empty() {
                    if let Some(&prev_idx) = seen_keys.get(&key) {
                        return Err(SheetError::Codegen {
                            message: format!(
                                "合表 '{}' 主键冲突: 键 '{}' 在第 {} 行和第 {} 行重复",
                                name,
                                key,
                                prev_idx + 1,
                                merged_rows.len() + 1
                            ),
                        });
                    }
                    seen_keys.insert(key, merged_rows.len());
                }
            }
            merged_rows.push(row.clone());
        }
    }

    Ok(SheetTable { name: name.to_string(), kind: base.kind.clone(), headers: base.headers.clone(), rows: merged_rows })
}

/// 合表入口函数
///
/// 对所有表执行分组、一致性检查、合并。
/// 返回合并后的表列表（包含未参与合表的原始表和合并后的新表）。
pub fn merge_tables(tables: &[SheetTable]) -> SheetResult<Vec<SheetTable>> {
    let groups = group_by_prefix(tables);

    let mut merged_names: HashMap<String, ()> = HashMap::new();
    let mut result: Vec<SheetTable> = Vec::new();

    for (group_name, group_tables) in &groups {
        check_structure_consistency(group_tables)?;

        let merged = merge_table_group(group_name, group_tables)?;

        for table in group_tables {
            merged_names.insert(table.name.clone(), ());
        }

        result.push(merged);
    }

    for table in tables {
        if !merged_names.contains_key(&table.name) {
            result.push(table.clone());
        }
    }

    Ok(result)
}
