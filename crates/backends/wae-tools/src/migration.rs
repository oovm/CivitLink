//! 数据库迁移模块
//!
//! 提供数据库 schema 版本管理和迁移能力。

#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
mod inner {

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use std::{collections::BTreeMap, sync::Arc};
    use tracing::{info, warn};
    use wae_database::DatabaseConnection;
    use wae_types::{WaeError, WaeResult};

    /// 迁移执行结果
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MigrationResult {
        /// 迁移名称
        pub name: String,
        /// 版本号
        pub version: i64,
        /// 执行时间
        pub executed_at: DateTime<Utc>,
        /// 执行耗时 (毫秒)
        pub duration_ms: u64,
        /// 是否成功
        pub success: bool,
        /// 是否为试运行
        pub dry_run: bool,
        /// 错误信息 (如果失败)
        pub error: Option<String>,
    }

    /// 迁移记录 (存储在数据库中)
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MigrationRecord {
        /// 版本号
        pub version: i64,
        /// 迁移名称
        pub name: String,
        /// 执行时间
        pub executed_at: DateTime<Utc>,
        /// 执行耗时 (毫秒)
        pub duration_ms: u64,
        /// 校验和 (可选)
        pub checksum: Option<String>,
    }

    /// 迁移执行配置
    #[derive(Debug, Clone, Default)]
    pub struct MigrationOptions {
        /// 是否为试运行模式
        pub dry_run: bool,
        /// 是否在事务中执行迁移
        pub transactional: bool,
    }

    impl MigrationOptions {
        /// 创建默认配置
        pub fn new() -> Self {
            Self::default()
        }

        /// 设置试运行模式
        pub fn with_dry_run(mut self, dry_run: bool) -> Self {
            self.dry_run = dry_run;
            self
        }

        /// 设置事务模式
        pub fn with_transactional(mut self, transactional: bool) -> Self {
            self.transactional = transactional;
            self
        }
    }

    /// 迁移 trait
    ///
    /// 实现此 trait 以定义数据库迁移。
    #[async_trait::async_trait]
    pub trait Migration: Send + Sync {
        /// 迁移名称
        fn name(&self) -> &str;

        /// 版本号 (必须唯一且递增)
        fn version(&self) -> i64;

        /// 描述信息
        fn description(&self) -> Option<&str> {
            None
        }

        /// 是否在事务中执行
        fn transactional(&self) -> bool {
            true
        }

        /// 执行迁移
        async fn up(&self, conn: &dyn DatabaseConnection) -> WaeResult<()>;

        /// 回滚迁移 (可选)
        async fn down(&self, _conn: &dyn DatabaseConnection) -> WaeResult<()> {
            Err(WaeError::internal("Rollback not supported".to_string()))
        }

        /// 是否支持回滚
        fn reversible(&self) -> bool {
            false
        }
    }

    /// 迁移表名
    const MIGRATION_TABLE_NAME: &str = "_migrations";

    /// 创建迁移表的 SQL
    const CREATE_MIGRATION_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    executed_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    checksum TEXT
)
"#;

    /// 迁移管理器
    pub struct Migrator {
        /// 已注册的迁移
        migrations: BTreeMap<i64, Arc<dyn Migration>>,
    }

    impl Migrator {
        /// 创建新的迁移管理器
        pub fn new() -> Self {
            Self { migrations: BTreeMap::new() }
        }

        /// 注册迁移
        pub fn register<M: Migration + 'static>(&mut self, migration: M) -> &mut Self {
            let version = migration.version();
            let name = migration.name().to_string();

            if self.migrations.contains_key(&version) {
                warn!(version = version, name = name, "Migration version already registered, replacing");
            }

            self.migrations.insert(version, Arc::new(migration));
            self
        }

        /// 批量注册迁移
        pub fn register_all<M: Migration + 'static>(&mut self, migrations: Vec<M>) -> &mut Self {
            for migration in migrations {
                self.register(migration);
            }
            self
        }

        /// 获取所有已注册的迁移
        pub fn migrations(&self) -> &BTreeMap<i64, Arc<dyn Migration>> {
            &self.migrations
        }

        /// 获取最新版本号
        pub fn latest_version(&self) -> Option<i64> {
            self.migrations.keys().next_back().copied()
        }

        /// 确保迁移表存在
        async fn ensure_migration_table(conn: &dyn DatabaseConnection) -> WaeResult<()> {
            conn.execute(CREATE_MIGRATION_TABLE).await?;
            Ok(())
        }

        /// 获取已执行的迁移记录
        async fn get_executed_migrations(conn: &dyn DatabaseConnection) -> WaeResult<BTreeMap<i64, MigrationRecord>> {
            let sql = format!(
                "SELECT version, name, executed_at, duration_ms, checksum FROM {} ORDER BY version",
                MIGRATION_TABLE_NAME
            );
            let mut rows = conn.query(&sql).await?;

            let mut records = BTreeMap::new();
            while let Some(row) = rows.next().await? {
                let version = row.get::<i64>(0)?;
                let name = row.get::<String>(1)?;
                let executed_at_str = row.get::<String>(2)?;
                let duration_ms = row.get::<i64>(3)? as u64;
                let checksum = row.get::<Option<String>>(4)?;

                let executed_at = DateTime::parse_from_rfc3339(&executed_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                records.insert(version, MigrationRecord { version, name, executed_at, duration_ms, checksum });
            }

            Ok(records)
        }

        /// 记录迁移执行
        async fn record_migration(conn: &dyn DatabaseConnection, migration: &dyn Migration, duration_ms: u64) -> WaeResult<()> {
            let sql = format!(
                "INSERT INTO {} (version, name, executed_at, duration_ms, checksum) VALUES (?, ?, ?, ?, NULL)",
                MIGRATION_TABLE_NAME
            );

            let executed_at = Utc::now().to_rfc3339();
            conn.execute_with(
                &sql,
                vec![
                    wae_types::Value::Integer(migration.version()),
                    wae_types::Value::String(migration.name().to_string()),
                    wae_types::Value::String(executed_at),
                    wae_types::Value::Integer(duration_ms as i64),
                ],
            )
            .await?;

            Ok(())
        }

        /// 删除迁移记录
        async fn remove_migration_record(conn: &dyn DatabaseConnection, version: i64) -> WaeResult<()> {
            let sql = format!("DELETE FROM {} WHERE version = ?", MIGRATION_TABLE_NAME);
            conn.execute_with(&sql, vec![wae_types::Value::Integer(version)]).await?;
            Ok(())
        }

        /// 执行所有待执行的迁移
        pub async fn migrate(&self, conn: &dyn DatabaseConnection) -> WaeResult<Vec<MigrationResult>> {
            self.migrate_with_options(conn, MigrationOptions::new()).await
        }

        /// 执行所有待执行的迁移 (带配置选项)
        pub async fn migrate_with_options(
            &self,
            conn: &dyn DatabaseConnection,
            options: MigrationOptions,
        ) -> WaeResult<Vec<MigrationResult>> {
            Self::ensure_migration_table(conn).await?;

            let executed = Self::get_executed_migrations(conn).await?;
            let mut results = Vec::new();

            for (version, migration) in &self.migrations {
                if executed.contains_key(version) {
                    info!(version = version, name = migration.name(), "Migration already executed, skipping");
                    continue;
                }

                let use_transaction = options.transactional && migration.transactional();

                info!(
                    version = version,
                    name = migration.name(),
                    dry_run = options.dry_run,
                    transactional = use_transaction,
                    "Executing migration"
                );

                let start = std::time::Instant::now();
                let executed_at = Utc::now();

                let result = if options.dry_run {
                    MigrationResult {
                        name: migration.name().to_string(),
                        version: *version,
                        executed_at,
                        duration_ms: 0,
                        success: true,
                        dry_run: true,
                        error: None,
                    }
                }
                else {
                    let mut success = true;
                    let mut error = None;
                    let duration_ms;

                    if use_transaction {
                        conn.begin_transaction().await?;
                    }

                    let migration_result = migration.up(conn).await;

                    match migration_result {
                        Ok(()) => {
                            duration_ms = start.elapsed().as_millis() as u64;
                            Self::record_migration(conn, migration.as_ref(), duration_ms).await?;

                            if use_transaction {
                                conn.commit().await?;
                            }

                            info!(
                                version = version,
                                name = migration.name(),
                                duration_ms = duration_ms,
                                "Migration completed successfully"
                            );
                        }
                        Err(e) => {
                            duration_ms = start.elapsed().as_millis() as u64;
                            success = false;
                            error = Some(e.to_string());

                            if use_transaction {
                                conn.rollback().await?;
                            }

                            warn!(
                                version = version,
                                name = migration.name(),
                                error = error.as_ref().unwrap(),
                                "Migration failed"
                            );
                        }
                    }

                    MigrationResult {
                        name: migration.name().to_string(),
                        version: *version,
                        executed_at,
                        duration_ms,
                        success,
                        dry_run: false,
                        error,
                    }
                };

                if !result.success {
                    return Err(WaeError::internal(format!(
                        "Migration '{}' failed: {}",
                        result.name,
                        result.error.as_deref().unwrap_or_default()
                    )));
                }

                results.push(result);
            }

            Ok(results)
        }

        /// 回滚到指定版本
        pub async fn rollback_to(&self, conn: &dyn DatabaseConnection, target_version: i64) -> WaeResult<Vec<MigrationResult>> {
            self.rollback_to_with_options(conn, target_version, MigrationOptions::new()).await
        }

        /// 回滚到指定版本 (带配置选项)
        pub async fn rollback_to_with_options(
            &self,
            conn: &dyn DatabaseConnection,
            target_version: i64,
            options: MigrationOptions,
        ) -> WaeResult<Vec<MigrationResult>> {
            Self::ensure_migration_table(conn).await?;

            let executed = Self::get_executed_migrations(conn).await?;
            let mut results = Vec::new();

            let versions_to_rollback: Vec<i64> =
                executed.keys().filter(|&&v| v > target_version).copied().collect::<Vec<_>>().into_iter().rev().collect();

            for version in versions_to_rollback {
                let migration = self
                    .migrations
                    .get(&version)
                    .ok_or_else(|| WaeError::not_found("migration", format!("Migration version {}", version)))?;

                if !migration.reversible() {
                    return Err(WaeError::internal(format!("Migration '{}' is not reversible", migration.name())));
                }

                let use_transaction = options.transactional && migration.transactional();

                info!(
                    version = version,
                    name = migration.name(),
                    dry_run = options.dry_run,
                    transactional = use_transaction,
                    "Rolling back migration"
                );

                let start = std::time::Instant::now();
                let executed_at = Utc::now();

                let result = if options.dry_run {
                    MigrationResult {
                        name: migration.name().to_string(),
                        version,
                        executed_at,
                        duration_ms: 0,
                        success: true,
                        dry_run: true,
                        error: None,
                    }
                }
                else {
                    let mut success = true;
                    let mut error = None;
                    let duration_ms;

                    if use_transaction {
                        conn.begin_transaction().await?;
                    }

                    let rollback_result = migration.down(conn).await;

                    match rollback_result {
                        Ok(()) => {
                            duration_ms = start.elapsed().as_millis() as u64;
                            Self::remove_migration_record(conn, version).await?;

                            if use_transaction {
                                conn.commit().await?;
                            }

                            info!(
                                version = version,
                                name = migration.name(),
                                duration_ms = duration_ms,
                                "Rollback completed successfully"
                            );
                        }
                        Err(e) => {
                            duration_ms = start.elapsed().as_millis() as u64;
                            success = false;
                            error = Some(e.to_string());

                            if use_transaction {
                                conn.rollback().await?;
                            }

                            warn!(
                                version = version,
                                name = migration.name(),
                                error = error.as_ref().unwrap(),
                                "Rollback failed"
                            );
                        }
                    }

                    MigrationResult {
                        name: migration.name().to_string(),
                        version,
                        executed_at,
                        duration_ms,
                        success,
                        dry_run: false,
                        error,
                    }
                };

                if !result.success {
                    return Err(WaeError::internal(format!(
                        "Rollback of migration '{}' failed: {}",
                        result.name,
                        result.error.as_deref().unwrap_or_default()
                    )));
                }

                results.push(result);
            }

            Ok(results)
        }

        /// 回滚最后一次迁移
        pub async fn rollback(&self, conn: &dyn DatabaseConnection) -> WaeResult<Option<MigrationResult>> {
            self.rollback_with_options(conn, MigrationOptions::new()).await
        }

        /// 回滚最后一次迁移 (带配置选项)
        pub async fn rollback_with_options(
            &self,
            conn: &dyn DatabaseConnection,
            options: MigrationOptions,
        ) -> WaeResult<Option<MigrationResult>> {
            Self::ensure_migration_table(conn).await?;

            let executed = Self::get_executed_migrations(conn).await?;

            let last_version = match executed.keys().next_back().copied() {
                Some(v) => v,
                None => {
                    info!("No migrations to rollback");
                    return Ok(None);
                }
            };

            let target_version = last_version - 1;
            let results = self.rollback_to_with_options(conn, target_version, options).await?;
            Ok(results.into_iter().next())
        }

        /// 重置数据库 (回滚所有迁移)
        pub async fn reset(&self, conn: &dyn DatabaseConnection) -> WaeResult<Vec<MigrationResult>> {
            self.reset_with_options(conn, MigrationOptions::new()).await
        }

        /// 重置数据库 (回滚所有迁移，带配置选项)
        pub async fn reset_with_options(
            &self,
            conn: &dyn DatabaseConnection,
            options: MigrationOptions,
        ) -> WaeResult<Vec<MigrationResult>> {
            self.rollback_to_with_options(conn, 0, options).await
        }

        /// 刷新数据库 (重置后重新执行所有迁移)
        pub async fn refresh(&self, conn: &dyn DatabaseConnection) -> WaeResult<Vec<MigrationResult>> {
            self.refresh_with_options(conn, MigrationOptions::new()).await
        }

        /// 刷新数据库 (重置后重新执行所有迁移，带配置选项)
        pub async fn refresh_with_options(
            &self,
            conn: &dyn DatabaseConnection,
            options: MigrationOptions,
        ) -> WaeResult<Vec<MigrationResult>> {
            self.reset_with_options(conn, options.clone()).await?;
            self.migrate_with_options(conn, options).await
        }

        /// 获取迁移状态
        pub async fn status(&self, conn: &dyn DatabaseConnection) -> WaeResult<Vec<MigrationStatus>> {
            Self::ensure_migration_table(conn).await?;

            let executed = Self::get_executed_migrations(conn).await?;
            let mut status_list = Vec::new();

            for (version, migration) in &self.migrations {
                let record = executed.get(version);
                status_list.push(MigrationStatus {
                    version: *version,
                    name: migration.name().to_string(),
                    description: migration.description().map(|s| s.to_string()),
                    reversible: migration.reversible(),
                    transactional: migration.transactional(),
                    executed: record.is_some(),
                    executed_at: record.map(|r| r.executed_at),
                    duration_ms: record.map(|r| r.duration_ms),
                });
            }

            Ok(status_list)
        }

        /// 获取迁移状态摘要
        pub async fn status_summary(&self, conn: &dyn DatabaseConnection) -> WaeResult<MigrationStatusSummary> {
            let status_list = self.status(conn).await?;
            let total = status_list.len();
            let executed = status_list.iter().filter(|s| s.executed).count();
            let pending = total - executed;
            let reversible = status_list.iter().filter(|s| s.reversible).count();

            Ok(MigrationStatusSummary { total, executed, pending, reversible, latest_version: self.latest_version() })
        }
    }

    impl Default for Migrator {
        fn default() -> Self {
            Self::new()
        }
    }

    /// 迁移状态
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MigrationStatus {
        /// 版本号
        pub version: i64,
        /// 迁移名称
        pub name: String,
        /// 描述信息
        pub description: Option<String>,
        /// 是否可回滚
        pub reversible: bool,
        /// 是否在事务中执行
        pub transactional: bool,
        /// 是否已执行
        pub executed: bool,
        /// 执行时间
        pub executed_at: Option<DateTime<Utc>>,
        /// 执行耗时
        pub duration_ms: Option<u64>,
    }

    /// 迁移状态摘要
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MigrationStatusSummary {
        /// 总迁移数
        pub total: usize,
        /// 已执行迁移数
        pub executed: usize,
        /// 待执行迁移数
        pub pending: usize,
        /// 可回滚迁移数
        pub reversible: usize,
        /// 最新版本号
        pub latest_version: Option<i64>,
    }

    /// 简单 SQL 迁移
    pub struct SimpleMigration {
        /// 版本号
        version: i64,
        /// 名称
        name: String,
        /// 描述
        description: Option<String>,
        /// 升级 SQL
        up_sql: String,
        /// 降级 SQL
        down_sql: Option<String>,
        /// 是否在事务中执行
        transactional: bool,
    }

    impl SimpleMigration {
        /// 创建新的简单迁移
        pub fn new<V: Into<i64>, N: Into<String>, U: Into<String>>(version: V, name: N, up_sql: U) -> Self {
            Self {
                version: version.into(),
                name: name.into(),
                description: None,
                up_sql: up_sql.into(),
                down_sql: None,
                transactional: true,
            }
        }

        /// 设置描述
        pub fn with_description<D: Into<String>>(mut self, description: D) -> Self {
            self.description = Some(description.into());
            self
        }

        /// 设置降级 SQL
        pub fn with_down_sql<D: Into<String>>(mut self, down_sql: D) -> Self {
            self.down_sql = Some(down_sql.into());
            self
        }

        /// 设置是否在事务中执行
        pub fn with_transactional(mut self, transactional: bool) -> Self {
            self.transactional = transactional;
            self
        }
    }

    #[async_trait::async_trait]
    impl Migration for SimpleMigration {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> i64 {
            self.version
        }

        fn description(&self) -> Option<&str> {
            self.description.as_deref()
        }

        fn transactional(&self) -> bool {
            self.transactional
        }

        async fn up(&self, conn: &dyn DatabaseConnection) -> WaeResult<()> {
            conn.execute(&self.up_sql).await?;
            Ok(())
        }

        async fn down(&self, conn: &dyn DatabaseConnection) -> WaeResult<()> {
            match &self.down_sql {
                Some(sql) => {
                    conn.execute(sql).await?;
                    Ok(())
                }
                None => Err(WaeError::internal("No down SQL provided".to_string())),
            }
        }

        fn reversible(&self) -> bool {
            self.down_sql.is_some()
        }
    }
}

#[cfg(any(feature = "database-limbo", feature = "database-postgres", feature = "database-mysql"))]
pub use inner::*;
