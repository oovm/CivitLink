//! Mock 工具模块
//!
//! 提供简洁易用的 Mock 功能，支持同步和异步操作。
//! 当启用 `mockall` feature 时，还可以使用更强大的 trait-level mocking。

use parking_lot::RwLock;
use std::sync::Arc;
use wae_types::{WaeError, WaeErrorKind, WaeResult as TestingResult};

#[cfg(feature = "mockall")]
pub use mockall::{self, __mock_MockObject, __mock_MockStatic, automock, mock, predicate, sequence};

/// Mock 调用记录
#[derive(Debug, Clone)]
pub struct MockCall {
    /// 调用参数 (JSON 序列化)
    pub args: Vec<String>,
    /// 调用时间戳
    pub timestamp: std::time::Instant,
}

/// Mock 返回结果
#[derive(Debug)]
pub enum MockResult<T> {
    /// 返回指定值
    Return(T),
    /// 返回错误
    Error(String),
    /// 根据调用次数返回不同结果
    Sequence(Vec<T>),
}

impl<T: Clone> MockResult<T> {
    /// 创建返回值
    pub fn return_value(value: T) -> Self {
        MockResult::Return(value)
    }

    /// 创建错误返回
    pub fn error(msg: impl Into<String>) -> Self {
        MockResult::Error(msg.into())
    }

    /// 创建序列返回
    pub fn sequence(values: Vec<T>) -> Self {
        MockResult::Sequence(values)
    }
}

impl<T: Clone> Clone for MockResult<T> {
    fn clone(&self) -> Self {
        match self {
            MockResult::Return(v) => MockResult::Return(v.clone()),
            MockResult::Error(e) => MockResult::Error(e.clone()),
            MockResult::Sequence(v) => MockResult::Sequence(v.clone()),
        }
    }
}

/// Mock 期望配置
#[derive(Debug, Default)]
pub struct MockExpectation {
    /// 期望的调用次数
    pub expected_calls: Option<usize>,
    /// 描述信息
    pub description: Option<String>,
}

impl MockExpectation {
    /// 创建新的期望配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置期望调用次数
    pub fn times(mut self, count: usize) -> Self {
        self.expected_calls = Some(count);
        self
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Mock 行为 trait
pub trait Mock: Send + Sync {
    /// 获取调用记录
    fn calls(&self) -> Vec<MockCall>;

    /// 获取调用次数
    fn call_count(&self) -> usize;

    /// 验证期望
    fn verify(&self) -> TestingResult<()>;

    /// 重置 Mock 状态
    fn reset(&self);
}

/// 异步 Mock 行为 trait
#[allow(async_fn_in_trait)]
pub trait AsyncMock: Mock {
    /// 异步验证期望
    async fn verify_async(&self) -> TestingResult<()>;
}

/// Mock 构建器
pub struct MockBuilder<T> {
    result: Option<MockResult<T>>,
    expectation: MockExpectation,
    calls: Arc<RwLock<Vec<MockCall>>>,
}

impl<T: Clone + Send + Sync + 'static> MockBuilder<T> {
    /// 创建新的 Mock 构建器
    pub fn new() -> Self {
        Self { result: None, expectation: MockExpectation::default(), calls: Arc::new(RwLock::new(Vec::new())) }
    }

    /// 设置返回值
    pub fn return_value(mut self, value: T) -> Self {
        self.result = Some(MockResult::return_value(value));
        self
    }

    /// 设置错误返回
    pub fn error(mut self, msg: impl Into<String>) -> Self {
        self.result = Some(MockResult::error(msg));
        self
    }

    /// 设置序列返回
    pub fn sequence(mut self, values: Vec<T>) -> Self {
        self.result = Some(MockResult::sequence(values));
        self
    }

    /// 设置期望
    pub fn expect(mut self, expectation: MockExpectation) -> Self {
        self.expectation = expectation;
        self
    }

    /// 构建可执行的 Mock
    pub fn build(self) -> MockFn<T> {
        MockFn {
            result: self.result,
            expectation: self.expectation,
            calls: self.calls,
            sequence_index: Arc::new(RwLock::new(0)),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Default for MockBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 可执行的 Mock 函数
pub struct MockFn<T> {
    result: Option<MockResult<T>>,
    expectation: MockExpectation,
    calls: Arc<RwLock<Vec<MockCall>>>,
    sequence_index: Arc<RwLock<usize>>,
}

impl<T: Clone + Send + Sync + 'static> MockFn<T> {
    /// 执行 Mock 调用
    pub fn call(&self, args: Vec<String>) -> TestingResult<T> {
        {
            let mut calls = self.calls.write();
            calls.push(MockCall { args, timestamp: std::time::Instant::now() });
        }

        match &self.result {
            Some(MockResult::Return(v)) => Ok(v.clone()),
            Some(MockResult::Error(e)) => Err(WaeError::new(WaeErrorKind::MockError { reason: e.clone() })),
            Some(MockResult::Sequence(values)) => {
                let mut idx = self.sequence_index.write();
                if *idx < values.len() {
                    let value = values[*idx].clone();
                    *idx += 1;
                    Ok(value)
                }
                else {
                    Err(WaeError::new(WaeErrorKind::MockError { reason: "Mock sequence exhausted".to_string() }))
                }
            }
            None => Err(WaeError::new(WaeErrorKind::MockError { reason: "No mock result configured".to_string() })),
        }
    }

    /// 异步执行 Mock 调用
    pub async fn call_async(&self, args: Vec<String>) -> TestingResult<T> {
        self.call(args)
    }
}

impl<T: Clone + Send + Sync + 'static> Mock for MockFn<T> {
    fn calls(&self) -> Vec<MockCall> {
        self.calls.read().clone()
    }

    fn call_count(&self) -> usize {
        self.calls.read().len()
    }

    fn verify(&self) -> TestingResult<()> {
        let actual_calls = self.call_count();

        if let Some(expected) = self.expectation.expected_calls
            && actual_calls != expected
        {
            return Err(WaeError::new(WaeErrorKind::AssertionFailed {
                message: format!("Expected {} calls, but got {}", expected, actual_calls),
            }));
        }

        Ok(())
    }

    fn reset(&self) {
        let mut calls = self.calls.write();
        calls.clear();
        let mut idx = self.sequence_index.write();
        *idx = 0;
    }
}

impl<T: Clone + Send + Sync + 'static> AsyncMock for MockFn<T> {
    async fn verify_async(&self) -> TestingResult<()> {
        self.verify()
    }
}

/// 验证 Mock 期望
pub fn verify<M: Mock>(mock: &M) -> TestingResult<()> {
    mock.verify()
}

/// 异步验证 Mock 期望
pub async fn verify_async<M: AsyncMock>(mock: &M) -> TestingResult<()> {
    mock.verify_async().await
}
