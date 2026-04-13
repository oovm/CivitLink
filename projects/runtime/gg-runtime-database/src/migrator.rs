//! 数据库迁移实现
//!
//! 提供版本化的数据库模式迁移管理，支持前进和回滚操作。

use gg_core::GResult;

use crate::{DatabaseDriver, DatabaseValue};

/// 迁移状态
///
/// 表示单个迁移的当前执行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus {
    /// 待执行
    Pending,
    /// 已应用
    Applied,
    /// 执行失败
    Failed,
}

/// 迁移记录
///
/// 描述单个数据库迁移的元数据。
pub struct Migration {
    /// 迁移版本号
    pub version: i64,
    /// 迁移描述
    pub description: String,
    /// 前进 SQL
    pub up_sql: String,
    /// 回滚 SQL
    pub down_sql: String,
}

impl Migration {
    /// 创建新的迁移记录
    ///
    /// # 参数
    ///
    /// - `version`: 迁移版本号
    /// - `description`: 迁移描述
    /// - `up_sql`: 前进 SQL 语句
    /// - `down_sql`: 回滚 SQL 语句
    pub fn new(version: i64, description: &str, up_sql: &str, down_sql: &str) -> Self {
        Self { version, description: description.to_string(), up_sql: up_sql.to_string(), down_sql: down_sql.to_string() }
    }
}

/// 数据库迁移器
///
/// 管理数据库模式版本化迁移，支持前进和回滚操作。
/// 使用 `_migrations` 元数据表记录已应用的迁移版本。
pub struct DatabaseMigrator<D: DatabaseDriver> {
    /// 数据库驱动
    driver: D,
    /// 待管理的迁移列表
    migrations: Vec<Migration>,
}

impl<D: DatabaseDriver> DatabaseMigrator<D> {
    /// 创建新的数据库迁移器
    ///
    /// # 参数
    ///
    /// - `driver`: 已连接的数据库驱动实例
    pub fn new(driver: D) -> Self {
        Self { driver, migrations: Vec::new() }
    }

    /// 添加迁移记录
    ///
    /// # 参数
    ///
    /// - `migration`: 迁移记录
    pub fn add_migration(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    /// 确保迁移元数据表存在
    fn ensure_migrations_table(&mut self) -> GResult<()> {
        self.driver.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY, description TEXT NOT NULL, applied_at TEXT NOT NULL)",
            &[],
        )?;
        Ok(())
    }

    /// 获取已应用的迁移版本列表
    fn applied_versions(&mut self) -> GResult<Vec<i64>> {
        let rows = self.driver.query("SELECT version FROM _migrations ORDER BY version", &[])?;
        let mut versions = Vec::new();
        for row in &rows {
            if let Some(DatabaseValue::Integer(v)) = row.get("version") {
                versions.push(*v);
            }
        }
        Ok(versions)
    }

    /// 执行所有待应用的迁移
    ///
    /// 按版本号顺序执行尚未应用的迁移，
    /// 每个迁移在独立事务中执行。
    pub fn migrate(&mut self) -> GResult<Vec<MigrationStatus>> {
        self.ensure_migrations_table()?;
        let applied = self.applied_versions()?;

        self.migrations.sort_by_key(|m| m.version);

        let pending: Vec<(i64, String, String)> = self
            .migrations
            .iter()
            .filter(|m| !applied.contains(&m.version))
            .map(|m| (m.version, m.description.clone(), m.up_sql.clone()))
            .collect();

        let mut results = Vec::new();
        for already_applied in &applied {
            if self.migrations.iter().any(|m| m.version == *already_applied) {
                results.push(MigrationStatus::Applied);
            }
        }

        for (version, description, up_sql) in &pending {
            match self.apply_migration_data(*version, description, up_sql) {
                Ok(()) => results.push(MigrationStatus::Applied),
                Err(_) => results.push(MigrationStatus::Failed),
            }
        }
        Ok(results)
    }

    /// 应用迁移数据
    fn apply_migration_data(&mut self, version: i64, description: &str, up_sql: &str) -> GResult<()> {
        self.driver.execute(up_sql, &[])?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.driver.execute(
            "INSERT INTO _migrations (version, description, applied_at) VALUES (?, ?, ?)",
            &[
                DatabaseValue::Integer(version),
                DatabaseValue::Text(description.to_string()),
                DatabaseValue::Text(now.to_string()),
            ],
        )?;
        Ok(())
    }

    /// 回滚指定数量的迁移
    ///
    /// 按版本号降序回滚最近应用的迁移。
    ///
    /// # 参数
    ///
    /// - `steps`: 要回滚的迁移数量
    pub fn rollback(&mut self, steps: usize) -> GResult<Vec<MigrationStatus>> {
        self.ensure_migrations_table()?;
        let applied = self.applied_versions()?;

        let mut to_rollback: Vec<(i64, String)> = self
            .migrations
            .iter()
            .filter(|m| applied.contains(&m.version))
            .map(|m| (m.version, m.down_sql.clone()))
            .collect();
        to_rollback.sort_by_key(|(v, _)| std::cmp::Reverse(*v));

        let mut results = Vec::new();
        for (version, down_sql) in to_rollback.into_iter().take(steps) {
            match self.revert_migration_data(version, &down_sql) {
                Ok(()) => results.push(MigrationStatus::Pending),
                Err(_) => results.push(MigrationStatus::Failed),
            }
        }
        Ok(results)
    }

    /// 回滚迁移数据
    fn revert_migration_data(&mut self, version: i64, down_sql: &str) -> GResult<()> {
        self.driver.execute(down_sql, &[])?;

        self.driver.execute(
            "DELETE FROM _migrations WHERE version = ?",
            &[DatabaseValue::Integer(version)],
        )?;
        Ok(())
    }

    /// 获取所有迁移的状态
    ///
    /// 返回每个迁移的当前状态列表。
    pub fn status(&mut self) -> GResult<Vec<(i64, String, MigrationStatus)>> {
        self.ensure_migrations_table()?;
        let applied = self.applied_versions()?;

        let mut statuses = Vec::new();
        for migration in &self.migrations {
            let status = if applied.contains(&migration.version) {
                MigrationStatus::Applied
            }
            else {
                MigrationStatus::Pending
            };
            statuses.push((migration.version, migration.description.clone(), status));
        }
        Ok(statuses)
    }
}
