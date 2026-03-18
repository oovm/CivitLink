//! Session 数据结构
//!
//! 提供 Session 的核心数据结构和操作方法。

use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Session ID 类型别名
pub type SessionId = String;

/// Session 数据结构
///
/// 表示一个 HTTP Session，包含 Session ID 和存储的数据。
/// 支持类型化的数据存取。
#[derive(Debug, Clone)]
pub struct Session {
    /// Session ID
    id: SessionId,
    /// Session 数据
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// 是否已修改
    dirty: Arc<RwLock<bool>>,
    /// 是否是新创建的
    is_new: bool,
}

impl Session {
    /// 创建新的 Session
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), data: Arc::new(RwLock::new(HashMap::new())), dirty: Arc::new(RwLock::new(false)), is_new: true }
    }

    /// 从现有数据创建 Session
    pub async fn from_data(id: impl Into<String>, data: HashMap<String, serde_json::Value>) -> Self {
        Self { id: id.into(), data: Arc::new(RwLock::new(data)), dirty: Arc::new(RwLock::new(false)), is_new: false }
    }

    /// 获取 Session ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 检查是否是新创建的 Session
    pub fn is_new(&self) -> bool {
        self.is_new
    }

    /// 检查 Session 是否已修改
    pub async fn is_dirty(&self) -> bool {
        *self.dirty.read().await
    }

    /// 标记 Session 为已修改
    pub async fn mark_dirty(&self) {
        *self.dirty.write().await = true;
    }

    /// 获取值
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.read().await.get(key).cloned()
    }

    /// 获取类型化的值
    pub async fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data.read().await.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 设置值
    pub async fn set<T: Serialize>(&self, key: impl Into<String>, value: T) {
        let key = key.into();
        let json_value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        self.data.write().await.insert(key, json_value);
        *self.dirty.write().await = true;
    }

    /// 删除值
    pub async fn remove(&self, key: &str) -> Option<serde_json::Value> {
        let result = self.data.write().await.remove(key);
        if result.is_some() {
            *self.dirty.write().await = true;
        }
        result
    }

    /// 检查键是否存在
    pub async fn contains(&self, key: &str) -> bool {
        self.data.read().await.contains_key(key)
    }

    /// 清空所有数据
    pub async fn clear(&self) {
        self.data.write().await.clear();
        *self.dirty.write().await = true;
    }

    /// 获取所有键
    pub async fn keys(&self) -> Vec<String> {
        self.data.read().await.keys().cloned().collect()
    }

    /// 获取数据条目数量
    pub async fn len(&self) -> usize {
        self.data.read().await.len()
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.data.read().await.is_empty()
    }

    /// 序列化为 JSON 字符串
    pub async fn to_json(&self) -> String {
        let data = self.data.read().await;
        serde_json::to_string(&*data).unwrap_or_default()
    }

    /// 从 JSON 字符串反序列化
    pub async fn from_json(&self, json: &str) -> bool {
        if let Ok(data) = serde_json::from_str::<HashMap<String, serde_json::Value>>(json) {
            *self.data.write().await = data;
            true
        }
        else {
            false
        }
    }
}
