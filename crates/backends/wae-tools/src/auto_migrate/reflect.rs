//! Schema 反射模块
//!
//! 从数据库读取现有表结构信息。

use std::collections::HashMap;
use wae_types::WaeResult;
use we_trust::schema::{
    ColumnDef, ColumnType, DatabaseBackend, DatabaseConnection, DatabaseResult, ForeignKeyDef, IndexDef, ReferentialAction,
    SchemaReflector as WeTrustSchemaReflector, TableSchema,
};

/// Schema 反射器
///
/// 包装 we-trust 的 SchemaReflector，保持 API 兼容性
pub struct SchemaReflector<'a> {
    inner: WeTrustSchemaReflector<'a>,
}

impl<'a> SchemaReflector<'a> {
    /// 创建新的反射器
    pub fn new(conn: &'a dyn DatabaseConnection) -> Self {
        Self { inner: WeTrustSchemaReflector::new(conn) }
    }

    /// 获取所有表名
    pub async fn get_table_names(&self) -> DatabaseResult<Vec<String>> {
        self.inner.get_table_names().await
    }

    /// 获取表的完整结构
    pub async fn get_table_schema(&self, table_name: &str) -> DatabaseResult<TableSchema> {
        self.inner.get_table_schema(table_name).await
    }

    /// 获取表的所有外键约束
    pub async fn get_foreign_keys(&self, table_name: &str) -> DatabaseResult<Vec<ForeignKeyDef>> {
        self.inner.get_foreign_keys(table_name).await
    }

    /// 获取表的所有列
    pub async fn get_columns(&self, table_name: &str) -> DatabaseResult<Vec<ColumnDef>> {
        self.inner.get_columns(table_name).await
    }

    /// 获取表的所有索引
    pub async fn get_indexes(&self, table_name: &str) -> DatabaseResult<Vec<IndexDef>> {
        self.inner.get_indexes(table_name).await
    }

    /// 获取所有表的 Schema
    pub async fn get_all_schemas(&self) -> WaeResult<HashMap<String, TableSchema>> {
        let schemas = self.inner.get_all_schemas().await?;
        Ok(schemas)
    }

    /// 检查表是否存在
    pub async fn table_exists(&self, table_name: &str) -> DatabaseResult<bool> {
        self.inner.table_exists(table_name).await
    }
}
