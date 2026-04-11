//! 数据迁移工具模块
//! 支持表结构版本升级，通过迁移脚本定义字段增删改操作

use crate::{
    schema::{FieldConstraint, SheetHeader, SheetTable},
    types::SheetType,
};

/// 数据迁移操作
#[derive(Debug, Clone)]
pub enum Migration {
    /// 添加字段
    AddField {
        /// 目标表名
        table_name: String,
        /// 字段名
        field_name: String,
        /// 字段类型
        field_type: String,
        /// 默认值
        default_value: String,
    },
    /// 删除字段
    RemoveField {
        /// 目标表名
        table_name: String,
        /// 字段名
        field_name: String,
    },
    /// 重命名字段
    RenameField {
        /// 目标表名
        table_name: String,
        /// 旧字段名
        old_name: String,
        /// 新字段名
        new_name: String,
    },
    /// 修改字段类型
    ChangeType {
        /// 目标表名
        table_name: String,
        /// 字段名
        field_name: String,
        /// 新类型
        new_type: String,
    },
}

/// 迁移报告条目
#[derive(Debug, Clone)]
pub enum MigrationEntry {
    /// 成功应用的迁移
    Applied {
        /// 迁移描述
        description: String,
    },
    /// 跳过的迁移
    Skipped {
        /// 迁移描述
        description: String,
        /// 跳过原因
        reason: String,
    },
    /// 失败的迁移
    Failed {
        /// 迁移描述
        description: String,
        /// 失败原因
        reason: String,
    },
}

impl std::fmt::Display for MigrationEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationEntry::Applied { description } => write!(f, "已应用: {}", description),
            MigrationEntry::Skipped { description, reason } => {
                write!(f, "已跳过: {} ({})", description, reason)
            }
            MigrationEntry::Failed { description, reason } => {
                write!(f, "失败: {} ({})", description, reason)
            }
        }
    }
}

/// 迁移报告
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// 迁移条目列表
    pub entries: Vec<MigrationEntry>,
}

impl MigrationReport {
    /// 创建空的迁移报告
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 获取成功应用的迁移数量
    pub fn applied_count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, MigrationEntry::Applied { .. })).count()
    }

    /// 获取跳过的迁移数量
    pub fn skipped_count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, MigrationEntry::Skipped { .. })).count()
    }

    /// 获取失败的迁移数量
    pub fn failed_count(&self) -> usize {
        self.entries.iter().filter(|e| matches!(e, MigrationEntry::Failed { .. })).count()
    }

    /// 将报告格式化为可读字符串
    pub fn format_report(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "迁移报告: {} 已应用, {} 跳过, {} 失败\n",
            self.applied_count(),
            self.skipped_count(),
            self.failed_count()
        ));
        for entry in &self.entries {
            output.push_str(&format!("  {}\n", entry));
        }
        output
    }
}

impl Default for MigrationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// 迁移脚本
#[derive(Debug, Clone)]
pub struct MigrationScript {
    /// 迁移操作列表
    pub migrations: Vec<Migration>,
}

impl MigrationScript {
    /// 创建空的迁移脚本
    pub fn new() -> Self {
        Self { migrations: Vec::new() }
    }

    /// 添加"添加字段"迁移
    pub fn add_field(
        mut self,
        table_name: impl Into<String>,
        field_name: impl Into<String>,
        field_type: impl Into<String>,
        default_value: impl Into<String>,
    ) -> Self {
        self.migrations.push(Migration::AddField {
            table_name: table_name.into(),
            field_name: field_name.into(),
            field_type: field_type.into(),
            default_value: default_value.into(),
        });
        self
    }

    /// 添加"删除字段"迁移
    pub fn remove_field(mut self, table_name: impl Into<String>, field_name: impl Into<String>) -> Self {
        self.migrations.push(Migration::RemoveField { table_name: table_name.into(), field_name: field_name.into() });
        self
    }

    /// 添加"重命名字段"迁移
    pub fn rename_field(
        mut self,
        table_name: impl Into<String>,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        self.migrations.push(Migration::RenameField {
            table_name: table_name.into(),
            old_name: old_name.into(),
            new_name: new_name.into(),
        });
        self
    }

    /// 添加"修改类型"迁移
    pub fn change_type(
        mut self,
        table_name: impl Into<String>,
        field_name: impl Into<String>,
        new_type: impl Into<String>,
    ) -> Self {
        self.migrations.push(Migration::ChangeType {
            table_name: table_name.into(),
            field_name: field_name.into(),
            new_type: new_type.into(),
        });
        self
    }
}

impl Default for MigrationScript {
    fn default() -> Self {
        Self::new()
    }
}

/// 对表格列表应用迁移脚本
///
/// 按顺序对匹配的表应用迁移操作，返回迁移报告
pub fn apply_migrations(tables: &mut [SheetTable], script: &MigrationScript) -> MigrationReport {
    let mut report = MigrationReport::new();

    for migration in &script.migrations {
        match migration {
            Migration::AddField { table_name, field_name, field_type, default_value } => {
                if let Some(table) = tables.iter_mut().find(|t| t.name == *table_name) {
                    let field_exists = table.headers.iter().any(|h| h.field_name == *field_name);
                    if field_exists {
                        report.entries.push(MigrationEntry::Skipped {
                            description: format!("表 '{}' 添加字段 '{}'", table_name, field_name),
                            reason: "字段已存在".to_string(),
                        });
                    }
                    else {
                        let col_index = table.headers.len();
                        match SheetType::parse(field_type) {
                            Ok(typing) => {
                                table.headers.push(SheetHeader {
                                    column: col_index,
                                    field_name: field_name.clone(),
                                    typing,
                                    comment: String::new(),
                                    constraint: FieldConstraint::None,
                                    validation_rules: vec![],
                                    default_value: Some(default_value.clone()),
                                });
                                for row in &mut table.rows {
                                    row.push(default_value.clone());
                                }
                                report.entries.push(MigrationEntry::Applied {
                                    description: format!(
                                        "表 '{}' 添加字段 '{}' 类型 '{}' 默认值 '{}'",
                                        table_name, field_name, field_type, default_value
                                    ),
                                });
                            }
                            Err(_) => {
                                report.entries.push(MigrationEntry::Failed {
                                    description: format!("表 '{}' 添加字段 '{}'", table_name, field_name),
                                    reason: format!("无法解析类型 '{}'", field_type),
                                });
                            }
                        }
                    }
                }
                else {
                    report.entries.push(MigrationEntry::Skipped {
                        description: format!("表 '{}' 添加字段 '{}'", table_name, field_name),
                        reason: "表不存在".to_string(),
                    });
                }
            }
            Migration::RemoveField { table_name, field_name } => {
                if let Some(table) = tables.iter_mut().find(|t| t.name == *table_name) {
                    if let Some(pos) = table.headers.iter().position(|h| h.field_name == *field_name) {
                        table.headers.remove(pos);
                        for row in &mut table.rows {
                            if pos < row.len() {
                                row.remove(pos);
                            }
                        }
                        for (i, header) in table.headers.iter_mut().enumerate() {
                            header.column = i;
                        }
                        report.entries.push(MigrationEntry::Applied {
                            description: format!("表 '{}' 删除字段 '{}'", table_name, field_name),
                        });
                    }
                    else {
                        report.entries.push(MigrationEntry::Skipped {
                            description: format!("表 '{}' 删除字段 '{}'", table_name, field_name),
                            reason: "字段不存在".to_string(),
                        });
                    }
                }
                else {
                    report.entries.push(MigrationEntry::Skipped {
                        description: format!("表 '{}' 删除字段 '{}'", table_name, field_name),
                        reason: "表不存在".to_string(),
                    });
                }
            }
            Migration::RenameField { table_name, old_name, new_name } => {
                if let Some(table) = tables.iter_mut().find(|t| t.name == *table_name) {
                    if let Some(header) = table.headers.iter_mut().find(|h| h.field_name == *old_name) {
                        header.field_name = new_name.clone();
                        report.entries.push(MigrationEntry::Applied {
                            description: format!("表 '{}' 字段 '{}' 重命名为 '{}'", table_name, old_name, new_name),
                        });
                    }
                    else {
                        report.entries.push(MigrationEntry::Skipped {
                            description: format!("表 '{}' 字段 '{}' 重命名", table_name, old_name),
                            reason: "字段不存在".to_string(),
                        });
                    }
                }
                else {
                    report.entries.push(MigrationEntry::Skipped {
                        description: format!("表 '{}' 字段 '{}' 重命名", table_name, old_name),
                        reason: "表不存在".to_string(),
                    });
                }
            }
            Migration::ChangeType { table_name, field_name, new_type } => {
                if let Some(table) = tables.iter_mut().find(|t| t.name == *table_name) {
                    if let Some(header) = table.headers.iter_mut().find(|h| h.field_name == *field_name) {
                        match SheetType::parse(new_type) {
                            Ok(typing) => {
                                header.typing = typing;
                                report.entries.push(MigrationEntry::Applied {
                                    description: format!("表 '{}' 字段 '{}' 修改类型为 '{}'", table_name, field_name, new_type),
                                });
                            }
                            Err(_) => {
                                report.entries.push(MigrationEntry::Failed {
                                    description: format!("表 '{}' 字段 '{}' 修改类型", table_name, field_name),
                                    reason: format!("无法解析类型 '{}'", new_type),
                                });
                            }
                        }
                    }
                    else {
                        report.entries.push(MigrationEntry::Skipped {
                            description: format!("表 '{}' 字段 '{}' 修改类型", table_name, field_name),
                            reason: "字段不存在".to_string(),
                        });
                    }
                }
                else {
                    report.entries.push(MigrationEntry::Skipped {
                        description: format!("表 '{}' 字段 '{}' 修改类型", table_name, field_name),
                        reason: "表不存在".to_string(),
                    });
                }
            }
        }
    }

    report
}
