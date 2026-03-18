//! Schema 差异比较模块
//!
//! 比较期望 Schema 和实际 Schema，生成迁移计划。

use std::collections::HashMap;
use wae_database::{ColumnDef, ForeignKeyDef, IndexDef, TableSchema};

/// 差异操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffAction {
    /// 创建操作
    Create,
    /// 修改操作
    Alter,
    /// 删除操作
    Drop,
}

/// 列差异
#[derive(Debug, Clone)]
pub struct ColumnDiff {
    /// 差异操作类型
    pub action: DiffAction,
    /// 列名
    pub column_name: String,
    /// 期望的列定义
    pub expected: Option<ColumnDef>,
    /// 实际的列定义
    pub actual: Option<ColumnDef>,
}

impl ColumnDiff {
    /// 生成 SQL 语句
    pub fn to_sql(&self, table_name: &str) -> Option<String> {
        match &self.action {
            DiffAction::Create => {
                let col = self.expected.as_ref()?;
                Some(format!("ALTER TABLE {} ADD COLUMN {}", table_name, col.to_sql()))
            }
            DiffAction::Alter => None,
            DiffAction::Drop => None,
        }
    }
}

/// 索引差异
#[derive(Debug, Clone)]
pub struct IndexDiff {
    /// 差异操作类型
    pub action: DiffAction,
    /// 索引名
    pub index_name: String,
    /// 期望的索引定义
    pub expected: Option<IndexDef>,
    /// 实际的索引定义
    pub actual: Option<IndexDef>,
}

/// 外键约束差异
#[derive(Debug, Clone)]
pub struct ForeignKeyDiff {
    /// 差异操作类型
    pub action: DiffAction,
    /// 外键约束名称
    pub foreign_key_name: String,
    /// 期望的外键约束定义
    pub expected: Option<ForeignKeyDef>,
    /// 实际的外键约束定义
    pub actual: Option<ForeignKeyDef>,
}

impl IndexDiff {
    /// 生成 SQL 语句
    pub fn to_sql(&self) -> Option<String> {
        match &self.action {
            DiffAction::Create => {
                let idx = self.expected.as_ref()?;
                Some(idx.to_create_sql())
            }
            DiffAction::Drop => Some(format!("DROP INDEX IF EXISTS {}", self.index_name)),
            DiffAction::Alter => {
                let mut sqls = Vec::new();
                sqls.push(format!("DROP INDEX IF EXISTS {}", self.index_name));
                if let Some(idx) = &self.expected {
                    sqls.push(idx.to_create_sql());
                }
                Some(sqls.join("; "))
            }
        }
    }
}

impl ForeignKeyDiff {
    /// 生成 SQL 语句
    pub fn to_sql(&self, table_name: &str) -> Option<String> {
        match &self.action {
            DiffAction::Create => {
                let fk = self.expected.as_ref()?;
                Some(fk.to_add_sql(table_name))
            }
            DiffAction::Drop => {
                let fk = self.actual.as_ref()?;
                Some(fk.to_drop_sql(table_name))
            }
            DiffAction::Alter => {
                let mut sqls = Vec::new();
                if let Some(fk) = &self.actual {
                    sqls.push(fk.to_drop_sql(table_name));
                }
                if let Some(fk) = &self.expected {
                    sqls.push(fk.to_add_sql(table_name));
                }
                Some(sqls.join("; "))
            }
        }
    }
}

/// 表结构差异
#[derive(Debug, Clone)]
pub struct TableDiff {
    /// 差异操作类型
    pub action: DiffAction,
    /// 表名
    pub table_name: String,
    /// 期望的表结构
    pub expected: Option<TableSchema>,
    /// 实际的表结构
    pub actual: Option<TableSchema>,
    /// 列差异列表
    pub column_diffs: Vec<ColumnDiff>,
    /// 索引差异列表
    pub index_diffs: Vec<IndexDiff>,
    /// 外键约束差异列表
    pub foreign_key_diffs: Vec<ForeignKeyDiff>,
}

impl TableDiff {
    /// 生成 SQL 语句列表
    pub fn to_sql(&self) -> Vec<String> {
        let mut sqls = Vec::new();

        match &self.action {
            DiffAction::Create => {
                if let Some(schema) = &self.expected {
                    sqls.push(schema.to_create_sql());
                    for idx in &schema.indexes {
                        sqls.push(idx.to_create_sql());
                    }
                }
            }
            DiffAction::Drop => {
                sqls.push(format!("DROP TABLE IF EXISTS {}", self.table_name));
            }
            DiffAction::Alter => {
                for col_diff in &self.column_diffs {
                    if let Some(sql) = col_diff.to_sql(&self.table_name) {
                        sqls.push(sql);
                    }
                }
                for idx_diff in &self.index_diffs {
                    if let Some(sql) = idx_diff.to_sql() {
                        sqls.push(sql);
                    }
                }
                for fk_diff in &self.foreign_key_diffs {
                    if let Some(sql) = fk_diff.to_sql(&self.table_name) {
                        sqls.push(sql);
                    }
                }
            }
        }

        sqls
    }

    /// 是否需要重建表
    pub fn needs_rebuild(&self) -> bool {
        self.column_diffs.iter().any(|d| matches!(d.action, DiffAction::Alter | DiffAction::Drop))
    }
}

/// Schema 差异
#[derive(Debug, Clone)]
pub struct SchemaDiff {
    /// 表差异列表
    pub table_diffs: Vec<TableDiff>,
}

impl SchemaDiff {
    /// 创建空的 Schema 差异
    pub fn empty() -> Self {
        Self { table_diffs: Vec::new() }
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.table_diffs.is_empty()
    }

    /// 生成所有 SQL 语句
    pub fn to_sql(&self) -> Vec<String> {
        self.table_diffs.iter().flat_map(|td| td.to_sql()).collect()
    }

    /// 比较期望 Schema 和实际 Schema
    pub fn compare(expected: &HashMap<String, TableSchema>, actual: &HashMap<String, TableSchema>) -> Self {
        let mut table_diffs = Vec::new();

        let all_table_names: std::collections::HashSet<&String> = expected.keys().chain(actual.keys()).collect();

        for table_name in all_table_names {
            let expected_schema = expected.get(table_name);
            let actual_schema = actual.get(table_name);

            let table_diff = match (expected_schema, actual_schema) {
                (Some(exp), None) => TableDiff {
                    action: DiffAction::Create,
                    table_name: table_name.clone(),
                    expected: Some(exp.clone()),
                    actual: None,
                    column_diffs: Vec::new(),
                    index_diffs: Vec::new(),
                    foreign_key_diffs: Vec::new(),
                },
                (None, Some(_)) => TableDiff {
                    action: DiffAction::Drop,
                    table_name: table_name.clone(),
                    expected: None,
                    actual: None,
                    column_diffs: Vec::new(),
                    index_diffs: Vec::new(),
                    foreign_key_diffs: Vec::new(),
                },
                (Some(exp), Some(act)) => {
                    let column_diffs = Self::compare_columns(&exp.columns, &act.columns);
                    let index_diffs = Self::compare_indexes(&exp.indexes, &act.indexes);
                    let foreign_key_diffs = Self::compare_foreign_keys(&exp.foreign_keys, &act.foreign_keys);

                    if column_diffs.is_empty() && index_diffs.is_empty() && foreign_key_diffs.is_empty() {
                        continue;
                    }

                    TableDiff {
                        action: DiffAction::Alter,
                        table_name: table_name.clone(),
                        expected: Some(exp.clone()),
                        actual: Some(act.clone()),
                        column_diffs,
                        index_diffs,
                        foreign_key_diffs,
                    }
                }
                (None, None) => continue,
            };

            table_diffs.push(table_diff);
        }

        Self { table_diffs }
    }

    /// 比较列差异
    fn compare_columns(expected: &[ColumnDef], actual: &[ColumnDef]) -> Vec<ColumnDiff> {
        let mut diffs = Vec::new();

        let expected_map: HashMap<&str, &ColumnDef> = expected.iter().map(|c| (c.name.as_str(), c)).collect();
        let actual_map: HashMap<&str, &ColumnDef> = actual.iter().map(|c| (c.name.as_str(), c)).collect();

        let all_columns: std::collections::HashSet<&str> = expected_map.keys().chain(actual_map.keys()).copied().collect();

        for col_name in all_columns {
            let expected_col = expected_map.get(col_name).copied();
            let actual_col = actual_map.get(col_name).copied();

            let diff = match (expected_col, actual_col) {
                (Some(exp), None) => ColumnDiff {
                    action: DiffAction::Create,
                    column_name: col_name.to_string(),
                    expected: Some(exp.clone()),
                    actual: None,
                },
                (None, Some(_)) => {
                    ColumnDiff { action: DiffAction::Drop, column_name: col_name.to_string(), expected: None, actual: None }
                }
                (Some(exp), Some(act)) => {
                    if Self::columns_differ(exp, act) {
                        ColumnDiff {
                            action: DiffAction::Alter,
                            column_name: col_name.to_string(),
                            expected: Some(exp.clone()),
                            actual: Some(act.clone()),
                        }
                    }
                    else {
                        continue;
                    }
                }
                (None, None) => continue,
            };

            diffs.push(diff);
        }

        diffs
    }

    /// 比较两个列定义是否不同
    fn columns_differ(a: &ColumnDef, b: &ColumnDef) -> bool {
        a.col_type != b.col_type
            || a.nullable != b.nullable
            || a.primary_key != b.primary_key
            || a.default_value != b.default_value
            || a.unique != b.unique
    }

    /// 比较索引差异
    fn compare_indexes(expected: &[IndexDef], actual: &[IndexDef]) -> Vec<IndexDiff> {
        let mut diffs = Vec::new();

        let expected_map: HashMap<&str, &IndexDef> = expected.iter().map(|i| (i.name.as_str(), i)).collect();
        let actual_map: HashMap<&str, &IndexDef> = actual.iter().map(|i| (i.name.as_str(), i)).collect();

        let all_indexes: std::collections::HashSet<&str> = expected_map.keys().chain(actual_map.keys()).copied().collect();

        for idx_name in all_indexes {
            let expected_idx = expected_map.get(idx_name).copied();
            let actual_idx = actual_map.get(idx_name).copied();

            let diff = match (expected_idx, actual_idx) {
                (Some(exp), None) => IndexDiff {
                    action: DiffAction::Create,
                    index_name: idx_name.to_string(),
                    expected: Some(exp.clone()),
                    actual: None,
                },
                (None, Some(_)) => {
                    IndexDiff { action: DiffAction::Drop, index_name: idx_name.to_string(), expected: None, actual: None }
                }
                (Some(exp), Some(act)) => {
                    if Self::indexes_differ(exp, act) {
                        IndexDiff {
                            action: DiffAction::Alter,
                            index_name: idx_name.to_string(),
                            expected: Some(exp.clone()),
                            actual: Some(act.clone()),
                        }
                    }
                    else {
                        continue;
                    }
                }
                (None, None) => continue,
            };

            diffs.push(diff);
        }

        diffs
    }

    /// 比较两个索引定义是否不同
    fn indexes_differ(a: &IndexDef, b: &IndexDef) -> bool {
        a.unique != b.unique || a.columns != b.columns
    }

    /// 比较外键约束差异
    fn compare_foreign_keys(expected: &[ForeignKeyDef], actual: &[ForeignKeyDef]) -> Vec<ForeignKeyDiff> {
        let mut diffs = Vec::new();

        let expected_map: HashMap<&str, &ForeignKeyDef> = expected.iter().map(|f| (f.name.as_str(), f)).collect();
        let actual_map: HashMap<&str, &ForeignKeyDef> = actual.iter().map(|f| (f.name.as_str(), f)).collect();

        let all_foreign_keys: std::collections::HashSet<&str> = expected_map.keys().chain(actual_map.keys()).copied().collect();

        for fk_name in all_foreign_keys {
            let expected_fk = expected_map.get(fk_name).copied();
            let actual_fk = actual_map.get(fk_name).copied();

            let diff = match (expected_fk, actual_fk) {
                (Some(exp), None) => ForeignKeyDiff {
                    action: DiffAction::Create,
                    foreign_key_name: fk_name.to_string(),
                    expected: Some(exp.clone()),
                    actual: None,
                },
                (None, Some(_)) => ForeignKeyDiff {
                    action: DiffAction::Drop,
                    foreign_key_name: fk_name.to_string(),
                    expected: None,
                    actual: None,
                },
                (Some(exp), Some(act)) => {
                    if Self::foreign_keys_differ(exp, act) {
                        ForeignKeyDiff {
                            action: DiffAction::Alter,
                            foreign_key_name: fk_name.to_string(),
                            expected: Some(exp.clone()),
                            actual: Some(act.clone()),
                        }
                    }
                    else {
                        continue;
                    }
                }
                (None, None) => continue,
            };

            diffs.push(diff);
        }

        diffs
    }

    /// 比较两个外键约束定义是否不同
    fn foreign_keys_differ(a: &ForeignKeyDef, b: &ForeignKeyDef) -> bool {
        a.column != b.column
            || a.ref_table != b.ref_table
            || a.ref_column != b.ref_column
            || a.on_update != b.on_update
            || a.on_delete != b.on_delete
    }
}
