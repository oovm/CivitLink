//! WAE Search - Elasticsearch/OpenSearch 搜索服务抽象层
//!
//! 提供统一的搜索能力抽象，支持 Elasticsearch 和 OpenSearch 后端。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持索引操作、文档 CRUD、搜索查询等特性。

#![warn(missing_docs)]

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;
use wae_types::WaeError;

/// 搜索操作结果类型
pub type SearchResult<T> = Result<T, WaeError>;

/// 搜索配置
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// 搜索服务地址
    pub url: String,
    /// 连接超时
    pub connection_timeout: Duration,
    /// 操作超时
    pub operation_timeout: Duration,
    /// 最大重试次数
    pub max_retries: usize,
    /// 重试间隔
    pub retry_interval: Duration,
    /// 默认索引名前缀
    pub index_prefix: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9200".to_string(),
            connection_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_interval: Duration::from_millis(100),
            index_prefix: String::new(),
        }
    }
}

/// 搜索命中结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit<T> {
    /// 文档 ID
    pub id: String,
    /// 文档索引
    pub index: String,
    /// 文档分数
    pub score: Option<f64>,
    /// 文档源数据
    pub source: T,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse<T> {
    /// 总命中数
    pub total: u64,
    /// 命中结果列表
    pub hits: Vec<SearchHit<T>>,
    /// 搜索耗时（毫秒）
    pub took: u64,
    /// 搜索是否超时
    pub timed_out: bool,
}

/// 搜索查询参数
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// 查询 DSL（JSON 格式）
    pub query: serde_json::Value,
    /// 起始位置
    pub from: Option<u64>,
    /// 返回数量
    pub size: Option<u64>,
    /// 排序配置
    pub sort: Option<serde_json::Value>,
    /// 聚合配置
    pub aggregations: Option<serde_json::Value>,
}

/// 文档批量操作项
#[derive(Debug, Clone)]
pub enum BulkOperation {
    /// 索引文档（创建或更新）
    Index {
        /// 文档 ID（可选）
        id: Option<String>,
        /// 文档源数据
        source: serde_json::Value,
    },
    /// 创建文档（仅当不存在时）
    Create {
        /// 文档 ID
        id: String,
        /// 文档源数据
        source: serde_json::Value,
    },
    /// 更新文档
    Update {
        /// 文档 ID
        id: String,
        /// 更新数据
        doc: serde_json::Value,
    },
    /// 删除文档
    Delete {
        /// 文档 ID
        id: String,
    },
}

/// 批量操作结果
#[derive(Debug, Clone)]
pub struct BulkResponse {
    /// 是否有错误
    pub has_errors: bool,
    /// 操作结果列表
    pub items: Vec<BulkItemResult>,
    /// 操作耗时（毫秒）
    pub took: u64,
}

/// 批量操作项结果
#[derive(Debug, Clone)]
pub struct BulkItemResult {
    /// 操作类型
    pub operation: String,
    /// 文档 ID
    pub id: String,
    /// 文档索引
    pub index: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 版本号
    pub version: Option<u64>,
}

/// 搜索服务核心 trait (dyn 兼容)
///
/// 定义统一的搜索操作接口，支持索引管理、文档 CRUD、搜索查询。
/// 所有方法都是异步的，适配 tokio 运行时。
#[async_trait::async_trait]
pub trait SearchBackend: Send + Sync {
    /// 检查搜索服务是否可用
    async fn ping(&self) -> SearchResult<bool>;

    /// 创建索引
    async fn create_index(
        &self,
        index: &str,
        settings: Option<serde_json::Value>,
        mappings: Option<serde_json::Value>,
    ) -> SearchResult<bool>;

    /// 删除索引
    async fn delete_index(&self, index: &str) -> SearchResult<bool>;

    /// 检查索引是否存在
    async fn index_exists(&self, index: &str) -> SearchResult<bool>;

    /// 获取索引信息
    async fn get_index(&self, index: &str) -> SearchResult<serde_json::Value>;

    /// 索引文档（创建或更新）
    async fn index_document(&self, index: &str, id: Option<&str>, document: &serde_json::Value) -> SearchResult<String>;

    /// 获取文档
    async fn get_document(&self, index: &str, id: &str) -> SearchResult<Option<serde_json::Value>>;

    /// 更新文档
    async fn update_document(&self, index: &str, id: &str, doc: &serde_json::Value) -> SearchResult<bool>;

    /// 删除文档
    async fn delete_document(&self, index: &str, id: &str) -> SearchResult<bool>;

    /// 文档是否存在
    async fn document_exists(&self, index: &str, id: &str) -> SearchResult<bool>;

    /// 批量操作
    async fn bulk(&self, index: &str, operations: Vec<BulkOperation>) -> SearchResult<BulkResponse>;

    /// 搜索文档（返回原始 JSON 响应）
    async fn search_raw(&self, index: &str, query: SearchQuery) -> SearchResult<SearchResponse<serde_json::Value>>;

    /// 按查询删除文档
    async fn delete_by_query(&self, index: &str, query: serde_json::Value) -> SearchResult<u64>;

    /// 按查询更新文档
    async fn update_by_query(&self, index: &str, query: serde_json::Value, script: serde_json::Value) -> SearchResult<u64>;

    /// 获取搜索配置
    fn config(&self) -> &SearchConfig;
}

/// 搜索服务 (提供泛型封装)
pub struct SearchService {
    backend: Box<dyn SearchBackend>,
}

impl SearchService {
    /// 从后端创建搜索服务
    pub fn new(backend: Box<dyn SearchBackend>) -> Self {
        Self { backend }
    }

    /// 检查搜索服务是否可用
    pub async fn ping(&self) -> SearchResult<bool> {
        self.backend.ping().await
    }

    /// 创建索引
    pub async fn create_index(
        &self,
        index: &str,
        settings: Option<serde_json::Value>,
        mappings: Option<serde_json::Value>,
    ) -> SearchResult<bool> {
        let full_index = self.build_index_name(index);
        self.backend.create_index(&full_index, settings, mappings).await
    }

    /// 删除索引
    pub async fn delete_index(&self, index: &str) -> SearchResult<bool> {
        let full_index = self.build_index_name(index);
        self.backend.delete_index(&full_index).await
    }

    /// 检查索引是否存在
    pub async fn index_exists(&self, index: &str) -> SearchResult<bool> {
        let full_index = self.build_index_name(index);
        self.backend.index_exists(&full_index).await
    }

    /// 获取索引信息
    pub async fn get_index(&self, index: &str) -> SearchResult<serde_json::Value> {
        let full_index = self.build_index_name(index);
        self.backend.get_index(&full_index).await
    }

    /// 索引文档（创建或更新）
    pub async fn index_document<T: Serialize + ?Sized>(
        &self,
        index: &str,
        id: Option<&str>,
        document: &T,
    ) -> SearchResult<String> {
        let full_index = self.build_index_name(index);
        let doc = serde_json::to_value(document).map_err(|_| WaeError::serialization_failed(std::any::type_name::<T>()))?;
        self.backend.index_document(&full_index, id, &doc).await
    }

    /// 获取文档
    pub async fn get_document<T: DeserializeOwned>(&self, index: &str, id: &str) -> SearchResult<Option<T>> {
        let full_index = self.build_index_name(index);
        let result = self.backend.get_document(&full_index, id).await?;
        match result {
            Some(value) => {
                let doc =
                    serde_json::from_value(value).map_err(|_| WaeError::deserialization_failed(std::any::type_name::<T>()))?;
                Ok(Some(doc))
            }
            None => Ok(None),
        }
    }

    /// 更新文档
    pub async fn update_document<T: Serialize + ?Sized>(&self, index: &str, id: &str, doc: &T) -> SearchResult<bool> {
        let full_index = self.build_index_name(index);
        let doc_value = serde_json::to_value(doc).map_err(|_| WaeError::serialization_failed(std::any::type_name::<T>()))?;
        self.backend.update_document(&full_index, id, &doc_value).await
    }

    /// 删除文档
    pub async fn delete_document(&self, index: &str, id: &str) -> SearchResult<bool> {
        let full_index = self.build_index_name(index);
        self.backend.delete_document(&full_index, id).await
    }

    /// 文档是否存在
    pub async fn document_exists(&self, index: &str, id: &str) -> SearchResult<bool> {
        let full_index = self.build_index_name(index);
        self.backend.document_exists(&full_index, id).await
    }

    /// 批量操作
    pub async fn bulk(&self, index: &str, operations: Vec<BulkOperation>) -> SearchResult<BulkResponse> {
        let full_index = self.build_index_name(index);
        self.backend.bulk(&full_index, operations).await
    }

    /// 搜索文档
    pub async fn search<T: DeserializeOwned>(&self, index: &str, query: SearchQuery) -> SearchResult<SearchResponse<T>> {
        let full_index = self.build_index_name(index);
        let raw_response = self.backend.search_raw(&full_index, query).await?;

        let mut hits = Vec::with_capacity(raw_response.hits.len());
        for hit in raw_response.hits {
            let source = serde_json::from_value(hit.source)
                .map_err(|_| wae_types::WaeError::deserialization_failed(std::any::type_name::<T>()))?;
            hits.push(SearchHit { id: hit.id, index: hit.index, score: hit.score, source });
        }

        Ok(SearchResponse { total: raw_response.total, hits, took: raw_response.took, timed_out: raw_response.timed_out })
    }

    /// 按查询删除文档
    pub async fn delete_by_query(&self, index: &str, query: serde_json::Value) -> SearchResult<u64> {
        let full_index = self.build_index_name(index);
        self.backend.delete_by_query(&full_index, query).await
    }

    /// 按查询更新文档
    pub async fn update_by_query(&self, index: &str, query: serde_json::Value, script: serde_json::Value) -> SearchResult<u64> {
        let full_index = self.build_index_name(index);
        self.backend.update_by_query(&full_index, query, script).await
    }

    /// 获取搜索配置
    pub fn config(&self) -> &SearchConfig {
        self.backend.config()
    }

    /// 构建带前缀的完整索引名
    pub fn build_index_name(&self, index: &str) -> String {
        let config = self.config();
        if config.index_prefix.is_empty() { index.to_string() } else { format!("{}_{}", config.index_prefix, index) }
    }
}
