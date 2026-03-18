//! Session 提取器
//!
//! 提供 Session 提取器实现。

use crate::Session;
use std::sync::Arc;
use wae_types::WaeError;

/// Session 提取错误
#[derive(Debug, Clone)]
pub struct SessionRejection {
    inner: WaeError,
}

impl SessionRejection {
    fn new(error: WaeError) -> Self {
        Self { inner: error }
    }

    /// 获取内部错误
    pub fn into_inner(self) -> WaeError {
        self.inner
    }
}

impl std::fmt::Display for SessionRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for SessionRejection {}

/// Session 提取器
///
/// 用于从请求中提取 Session。
///
/// # 示例
///
/// ```rust,ignore
/// use wae_session::SessionExtractor;
///
/// async fn handler(request: &Request) {
///     let session = SessionExtractor::from_request(request).unwrap();
///     let user_id: Option<String> = session.get_typed("user_id").await;
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SessionExtractor {
    /// Session 引用
    session: Arc<Session>,
}

impl SessionExtractor {
    /// 创建 Session 提取器
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    /// 从请求的扩展中提取 Session
    pub fn from_request<B>(request: &http::Request<B>) -> Result<Self, SessionRejection> {
        request
            .extensions()
            .get::<Arc<Session>>()
            .cloned()
            .map(|session| SessionExtractor { session })
            .ok_or_else(|| SessionRejection::new(WaeError::internal("Session not found in request extensions")))
    }

    /// 获取 Session 引用
    pub fn inner(&self) -> &Session {
        &self.session
    }

    /// 获取 Session ID
    pub fn id(&self) -> &str {
        self.session.id()
    }

    /// 检查是否是新创建的 Session
    pub fn is_new(&self) -> bool {
        self.session.is_new()
    }

    /// 获取值
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.session.get(key).await
    }

    /// 获取类型化的值
    pub async fn get_typed<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.session.get_typed(key).await
    }

    /// 设置值
    pub async fn set<T: serde::Serialize>(&self, key: impl Into<String>, value: T) {
        self.session.set(key, value).await
    }

    /// 删除值
    pub async fn remove(&self, key: &str) -> Option<serde_json::Value> {
        self.session.remove(key).await
    }

    /// 检查键是否存在
    pub async fn contains(&self, key: &str) -> bool {
        self.session.contains(key).await
    }

    /// 清空所有数据
    pub async fn clear(&self) {
        self.session.clear().await
    }

    /// 获取所有键
    pub async fn keys(&self) -> Vec<String> {
        self.session.keys().await
    }

    /// 获取数据条目数量
    pub async fn len(&self) -> usize {
        self.session.len().await
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.session.is_empty().await
    }
}
