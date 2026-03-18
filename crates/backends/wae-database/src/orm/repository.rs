//! 仓储实现模块
//!
//! 提供 CRUD 操作封装

use super::{
    builder::{DeleteBuilder, InsertBuilder, QueryBuilder, SelectBuilder, UpdateBuilder},
    condition::Condition,
    entity::{Entity, FromRow, ToRow},
};
use crate::{DatabaseResult, connection::DatabaseConnection};
use std::marker::PhantomData;

#[cfg(feature = "limbo")]
use crate::DatabaseError;
#[cfg(any(feature = "limbo", feature = "mysql"))]
use crate::connection::DatabaseRows;
#[cfg(feature = "limbo")]
use limbo::Value as LimboValue;

/// 仓储 trait - 提供 CRUD 操作
#[allow(async_fn_in_trait)]
pub trait Repository<E: Entity + FromRow>: Send + Sync {
    /// 根据主键查找
    async fn find_by_id(&self, id: E::Id) -> DatabaseResult<Option<E>>;

    /// 查找所有
    async fn find_all(&self) -> DatabaseResult<Vec<E>>;

    /// 根据条件查找
    async fn find_by(&self, condition: Condition) -> DatabaseResult<Vec<E>>;

    /// 根据条件查找一条
    async fn find_one(&self, condition: Condition) -> DatabaseResult<Option<E>>;

    /// 插入实体
    async fn insert(&self, entity: &E) -> DatabaseResult<i64>
    where
        E: ToRow;

    /// 更新实体
    async fn update(&self, entity: &E) -> DatabaseResult<u64>
    where
        E: ToRow;

    /// 根据主键删除
    async fn delete_by_id(&self, id: E::Id) -> DatabaseResult<u64>;

    /// 根据条件删除
    async fn delete_by(&self, condition: Condition) -> DatabaseResult<u64>;

    /// 统计数量
    async fn count(&self) -> DatabaseResult<u64>;

    /// 根据条件统计
    async fn count_by(&self, condition: Condition) -> DatabaseResult<u64>;

    /// 检查是否存在
    async fn exists(&self, id: E::Id) -> DatabaseResult<bool>;
}

#[cfg(feature = "limbo")]
/// 基于数据库连接的仓储实现
pub struct DbRepository<E: Entity + FromRow> {
    conn: Box<dyn DatabaseConnection>,
    _marker: PhantomData<E>,
}

#[cfg(feature = "limbo")]
impl<E: Entity + FromRow> DbRepository<E> {
    /// 创建仓储
    pub fn new(conn: Box<dyn DatabaseConnection>) -> Self {
        Self { conn, _marker: PhantomData }
    }

    /// 获取 SELECT 构建器
    pub fn select(&self) -> SelectBuilder<E> {
        QueryBuilder::select()
    }

    /// 获取 INSERT 构建器
    pub fn insert_builder(&self) -> InsertBuilder<E> {
        QueryBuilder::insert()
    }

    /// 获取 UPDATE 构建器
    pub fn update_builder(&self) -> UpdateBuilder<E> {
        QueryBuilder::update()
    }

    /// 获取 DELETE 构建器
    pub fn delete_builder(&self) -> DeleteBuilder<E> {
        QueryBuilder::delete()
    }

    /// 执行查询并解析结果
    async fn query_and_parse(&self, sql: &str, params: Vec<LimboValue>) -> DatabaseResult<Vec<E>> {
        let rows = self.conn.query_with_limbo(sql, params).await?;
        self.parse_rows(rows).await
    }

    /// 解析查询结果
    async fn parse_rows(&self, mut rows: DatabaseRows) -> DatabaseResult<Vec<E>> {
        let mut entities = Vec::new();
        while let Some(row) = rows.next().await? {
            entities.push(E::from_row(&row)?);
        }
        Ok(entities)
    }
}

#[cfg(feature = "limbo")]
impl<E: Entity + FromRow> Repository<E> for DbRepository<E> {
    async fn find_by_id(&self, id: E::Id) -> DatabaseResult<Option<E>> {
        let condition = Condition::eq(E::id_column(), id);
        self.find_one(condition).await
    }

    async fn find_all(&self) -> DatabaseResult<Vec<E>> {
        let (sql, params) = QueryBuilder::select::<E>().build_limbo();
        self.query_and_parse(&sql, params).await
    }

    async fn find_by(&self, condition: Condition) -> DatabaseResult<Vec<E>> {
        let (sql, params) = QueryBuilder::select::<E>().where_(condition).build_limbo();
        self.query_and_parse(&sql, params).await
    }

    async fn find_one(&self, condition: Condition) -> DatabaseResult<Option<E>> {
        let (sql, params) = QueryBuilder::select::<E>().where_(condition).limit(1).build_limbo();
        let mut entities = self.query_and_parse(&sql, params).await?;
        Ok(entities.pop())
    }

    async fn insert(&self, entity: &E) -> DatabaseResult<i64>
    where
        E: ToRow,
    {
        let (sql, params) = InsertBuilder::<E>::from_entity(entity).build_limbo();
        self.conn.execute_with_limbo(&sql, params).await?;
        let rows = self.conn.query("SELECT last_insert_rowid()").await?;
        let mut rows = rows;
        if let Some(row) = rows.next().await? {
            row.get::<i64>(0)
        }
        else {
            Err(DatabaseError::internal("Failed to get last insert id"))
        }
    }

    async fn update(&self, entity: &E) -> DatabaseResult<u64>
    where
        E: ToRow,
    {
        let id = entity.id();
        let (sql, params) = UpdateBuilder::<E>::from_entity(entity).where_id(id).build_limbo();
        self.conn.execute_with_limbo(&sql, params).await
    }

    async fn delete_by_id(&self, id: E::Id) -> DatabaseResult<u64> {
        let (sql, params) = QueryBuilder::delete::<E>().where_id(id).build_limbo();
        self.conn.execute_with_limbo(&sql, params).await
    }

    async fn delete_by(&self, condition: Condition) -> DatabaseResult<u64> {
        let (sql, params) = QueryBuilder::delete::<E>().where_(condition).build_limbo();
        self.conn.execute_with_limbo(&sql, params).await
    }

    async fn count(&self) -> DatabaseResult<u64> {
        let sql = format!("SELECT COUNT(*) FROM {}", E::table_name());
        let rows = self.conn.query(&sql).await?;
        let mut rows = rows;
        if let Some(row) = rows.next().await? { row.get::<i64>(0).map(|n| n as u64) } else { Ok(0) }
    }

    async fn count_by(&self, condition: Condition) -> DatabaseResult<u64> {
        let (cond_sql, params) = condition.build_limbo();
        let sql = format!("SELECT COUNT(*) FROM {} WHERE {}", E::table_name(), cond_sql);
        let rows = self.conn.query_with_limbo(&sql, params).await?;
        let mut rows = rows;
        if let Some(row) = rows.next().await? { row.get::<i64>(0).map(|n| n as u64) } else { Ok(0) }
    }

    async fn exists(&self, id: E::Id) -> DatabaseResult<bool> {
        let count = self.count_by(Condition::eq(E::id_column(), id)).await?;
        Ok(count > 0)
    }
}

#[cfg(feature = "mysql")]
/// 基于 MySQL 数据库连接的仓储实现
pub struct MySqlDbRepository<E: Entity + FromRow> {
    conn: Box<dyn DatabaseConnection>,
    _marker: PhantomData<E>,
}

#[cfg(feature = "mysql")]
impl<E: Entity + FromRow> MySqlDbRepository<E> {
    /// 创建仓储
    pub fn new(conn: Box<dyn DatabaseConnection>) -> Self {
        Self { conn, _marker: PhantomData }
    }

    /// 获取 SELECT 构建器
    pub fn select(&self) -> SelectBuilder<E> {
        QueryBuilder::select()
    }

    /// 获取 INSERT 构建器
    pub fn insert_builder(&self) -> InsertBuilder<E> {
        QueryBuilder::insert()
    }

    /// 获取 UPDATE 构建器
    pub fn update_builder(&self) -> UpdateBuilder<E> {
        QueryBuilder::update()
    }

    /// 获取 DELETE 构建器
    pub fn delete_builder(&self) -> DeleteBuilder<E> {
        QueryBuilder::delete()
    }

    /// 解析查询结果
    async fn parse_rows(&self, mut rows: DatabaseRows) -> DatabaseResult<Vec<E>> {
        let mut entities = Vec::new();
        while let Some(row) = rows.next().await? {
            entities.push(E::from_row(&row)?);
        }
        Ok(entities)
    }
}

#[cfg(feature = "mysql")]
impl<E: Entity + FromRow> Repository<E> for MySqlDbRepository<E> {
    async fn find_by_id(&self, id: E::Id) -> DatabaseResult<Option<E>> {
        let condition = Condition::eq(E::id_column(), id);
        self.find_one(condition).await
    }

    async fn find_all(&self) -> DatabaseResult<Vec<E>> {
        let sql = format!("SELECT * FROM {}", E::table_name());
        let rows = self.conn.query(&sql).await?;
        self.parse_rows(rows).await
    }

    async fn find_by(&self, condition: Condition) -> DatabaseResult<Vec<E>> {
        let (cond_sql, params) = condition.build_mysql();
        let sql = format!("SELECT * FROM {} WHERE {}", E::table_name(), cond_sql);
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        let rows = self.conn.query_with(&sql, wae_params).await?;
        self.parse_rows(rows).await
    }

    async fn find_one(&self, condition: Condition) -> DatabaseResult<Option<E>> {
        let (cond_sql, params) = condition.build_mysql();
        let sql = format!("SELECT * FROM {} WHERE {} LIMIT 1", E::table_name(), cond_sql);
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        let rows = self.conn.query_with(&sql, wae_params).await?;
        let mut entities = self.parse_rows(rows).await?;
        Ok(entities.pop())
    }

    async fn insert(&self, entity: &E) -> DatabaseResult<i64>
    where
        E: ToRow,
    {
        let (sql, params) = InsertBuilder::<E>::from_entity(entity).build_mysql();
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        self.conn.execute_with(&sql, wae_params).await?;
        let rows = self.conn.query("SELECT LAST_INSERT_ID()").await?;
        let mut rows = rows;
        if let Some(row) = rows.next().await? {
            row.get::<i64>(0)
        }
        else {
            Err(crate::DatabaseError::internal("Failed to get last insert id"))
        }
    }

    async fn update(&self, entity: &E) -> DatabaseResult<u64>
    where
        E: ToRow,
    {
        let id = entity.id();
        let (sql, params) = UpdateBuilder::<E>::from_entity(entity).where_id(id).build_mysql();
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        self.conn.execute_with(&sql, wae_params).await
    }

    async fn delete_by_id(&self, id: E::Id) -> DatabaseResult<u64> {
        let (sql, params) = QueryBuilder::delete::<E>().where_id(id).build_mysql();
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        self.conn.execute_with(&sql, wae_params).await
    }

    async fn delete_by(&self, condition: Condition) -> DatabaseResult<u64> {
        let (sql, params) = QueryBuilder::delete::<E>().where_(condition).build_mysql();
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        self.conn.execute_with(&sql, wae_params).await
    }

    async fn count(&self) -> DatabaseResult<u64> {
        let sql = format!("SELECT COUNT(*) FROM {}", E::table_name());
        let rows = self.conn.query(&sql).await?;
        let mut rows = rows;
        if let Some(row) = rows.next().await? { row.get::<i64>(0).map(|n| n as u64) } else { Ok(0) }
    }

    async fn count_by(&self, condition: Condition) -> DatabaseResult<u64> {
        let (cond_sql, params) = condition.build_mysql();
        let sql = format!("SELECT COUNT(*) FROM {} WHERE {}", E::table_name(), cond_sql);
        let wae_params: Vec<wae_types::Value> = params.iter().map(|v| crate::types::mysql_value_to_wae(v.clone())).collect();
        let rows = self.conn.query_with(&sql, wae_params).await?;
        let mut rows = rows;
        if let Some(row) = rows.next().await? { row.get::<i64>(0).map(|n| n as u64) } else { Ok(0) }
    }

    async fn exists(&self, id: E::Id) -> DatabaseResult<bool> {
        let count = self.count_by(Condition::eq(E::id_column(), id)).await?;
        Ok(count > 0)
    }
}
