//! 数据库 Mock 工具模块
//!
//! 提供 MockDatabase 结构体，用于模拟数据库操作，支持查询期望配置、调用记录和验证。

use crate::mock::{Mock, MockCall};
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

/// 数据库查询记录
#[derive(Debug, Clone)]
pub struct DatabaseQuery {
    /// 查询名称/类型
    pub query_name: String,
    /// 查询参数 (JSON 序列化)
    pub params: Vec<String>,
    /// 查询时间戳
    pub timestamp: std::time::Instant,
}

/// 数据库查询期望
#[derive(Debug, Clone)]
pub enum DatabaseResult<T> {
    /// 返回指定值
    Return(T),
    /// 返回错误
    Error(String),
    /// 根据查询名称匹配返回
    NamedReturns(HashMap<String, T>),
}

impl<T: Clone> DatabaseResult<T> {
    /// 创建返回值
    pub fn return_value(value: T) -> Self {
        DatabaseResult::Return(value)
    }

    /// 创建错误返回
    pub fn error(msg: impl Into<String>) -> Self {
        DatabaseResult::Error(msg.into())
    }

    /// 创建命名返回映射
    pub fn named_returns(map: HashMap<String, T>) -> Self {
        DatabaseResult::NamedReturns(map)
    }
}

/// 数据库 Mock 数据库
#[derive(Debug, Default)]
pub struct DatabaseExpectation {
    /// 期望的查询次数映射 (查询名称 -> 期望次数)
    pub expected_queries: HashMap<String, usize>,
    /// 描述信息
    pub description: Option<String>,
}

impl DatabaseExpectation {
    /// 创建新的数据库期望配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置期望查询次数
    pub fn expect_query(mut self, query_name: impl Into<String>, count: usize) -> Self {
        self.expected_queries.insert(query_name.into(), count);
        self
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Mock 数据库构建器
pub struct MockDatabaseBuilder<T> {
    result: Option<DatabaseResult<T>>,
    expectation: DatabaseExpectation,
    queries: Arc<RwLock<Vec<DatabaseQuery>>>,
}

impl<T: Clone + Send + Sync + 'static> MockDatabaseBuilder<T> {
    /// 创建新的 Mock 数据库构建器
    pub fn new() -> Self {
        Self { result: None, expectation: DatabaseExpectation::default(), queries: Arc::new(RwLock::new(Vec::new())) }
    }

    /// 设置返回值
    pub fn return_value(mut self, value: T) -> Self {
        self.result = Some(DatabaseResult::return_value(value));
        self
    }

    /// 设置错误返回
    pub fn error(mut self, msg: impl Into<String>) -> Self {
        self.result = Some(DatabaseResult::error(msg));
        self
    }

    /// 设置命名返回映射
    pub fn named_returns(mut self, map: HashMap<String, T>) -> Self {
        self.result = Some(DatabaseResult::named_returns(map));
        self
    }

    /// 设置期望
    pub fn expect(mut self, expectation: DatabaseExpectation) -> Self {
        self.expectation = expectation;
        self
    }

    /// 构建 Mock 数据库
    pub fn build(self) -> MockDatabase<T> {
        MockDatabase { result: self.result, expectation: self.expectation, queries: self.queries }
    }
}

impl<T: Clone + Send + Sync + 'static> Default for MockDatabaseBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock 数据库
///
/// 用于模拟数据库操作，支持查询期望配置、调用记录和验证。
pub struct MockDatabase<T> {
    result: Option<DatabaseResult<T>>,
    expectation: DatabaseExpectation,
    queries: Arc<RwLock<Vec<DatabaseQuery>>>,
}

impl<T: Clone> MockDatabase<T> {
    /// 执行数据库查询
    pub fn query(&self, query_name: impl Into<String>, params: Vec<String>) -> TestingResult<T> {
        let query_name = query_name.into();
        {
            let mut queries = self.queries.write();
            queries.push(DatabaseQuery { query_name: query_name.clone(), params, timestamp: std::time::Instant::now() });
        }

        match &self.result {
            Some(DatabaseResult::Return(v)) => Ok(v.clone()),
            Some(DatabaseResult::Error(e)) => Err(WaeError::new(WaeErrorKind::MockError { reason: e.clone() })),
            Some(DatabaseResult::NamedReturns(map)) => {
                if let Some(value) = map.get(&query_name) {
                    Ok(value.clone())
                }
                else {
                    Err(WaeError::new(WaeErrorKind::MockError { reason: format!("No mock result for query: {}", query_name) }))
                }
            }
            None => Err(WaeError::new(WaeErrorKind::MockError { reason: "No mock result configured".to_string() })),
        }
    }

    /// 异步执行数据库查询
    pub async fn query_async(&self, query_name: impl Into<String>, params: Vec<String>) -> TestingResult<T> {
        self.query(query_name, params)
    }

    /// 获取查询记录
    pub fn queries(&self) -> Vec<DatabaseQuery> {
        self.queries.read().clone()
    }

    /// 获取查询次数
    pub fn query_count(&self) -> usize {
        self.queries.read().len()
    }

    /// 获取指定查询的次数
    pub fn query_count_by_name(&self, query_name: &str) -> usize {
        self.queries.read().iter().filter(|q| q.query_name == query_name).count()
    }
}

impl<T: Clone + Send + Sync + 'static> Mock for MockDatabase<T> {
    fn calls(&self) -> Vec<MockCall> {
        self.queries
            .read()
            .iter()
            .map(|q| MockCall {
                args: vec![q.query_name.clone()].into_iter().chain(q.params.clone()).collect(),
                timestamp: q.timestamp,
            })
            .collect()
    }

    fn call_count(&self) -> usize {
        self.query_count()
    }

    fn verify(&self) -> TestingResult<()> {
        for (query_name, expected) in &self.expectation.expected_queries {
            let actual = self.query_count_by_name(query_name);
            if actual != *expected {
                return Err(WaeError::new(WaeErrorKind::AssertionFailed {
                    message: format!("Expected {} calls for query '{}', but got {}", expected, query_name, actual),
                }));
            }
        }

        Ok(())
    }

    fn reset(&self) {
        let mut queries = self.queries.write();
        queries.clear();
    }
}
