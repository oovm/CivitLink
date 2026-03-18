//! Elasticsearch 搜索后端实现

use crate::{BulkItemResult, BulkOperation, BulkResponse, SearchBackend, SearchConfig, SearchHit, SearchQuery, SearchResponse, SearchResult};
use async_trait::async_trait;
use elasticsearch::{Elasticsearch, http::transport::{Transport, SingleNodeConnectionPool}, SearchParts, IndexParts, GetParts, UpdateParts, DeleteParts, BulkParts, CountParts};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Elasticsearch 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElasticsearchConfig {
    /// Elasticsearch 服务器地址
    pub url: String,
    /// 用户名（可选）
    pub username: Option<String>,
    /// 密码（可选）
    pub password: Option<String>,
    /// API Key（可选）
    pub api_key: Option<String>,
}

impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:9200".to_string(),
            username: None,
            password: None,
            api_key: None,
        }
    }
}

impl ElasticsearchConfig {
    /// 创建新的 Elasticsearch 配置
    pub fn new(url: String) -> Self {
        Self { url, ..Default::default() }
    }

    /// 设置用户名
    pub fn with_username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    /// 设置密码
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// 设置 API Key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
}

/// Elasticsearch 搜索后端
pub struct ElasticsearchBackend {
    client: Elasticsearch,
    config: SearchConfig,
}

impl ElasticsearchBackend {
    /// 创建新的 Elasticsearch 搜索后端
    pub fn new(es_config: ElasticsearchConfig, search_config: SearchConfig) -> SearchResult<Self> {
        let transport = Transport::single_node(&es_config.url)
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to create Elasticsearch transport: {}", e)))?;
        
        let client = Elasticsearch::new(transport);
        
        Ok(Self { client, config: search_config })
    }
}

#[async_trait]
impl SearchBackend for ElasticsearchBackend {
    async fn ping(&self) -> SearchResult<bool> {
        let response = self.client.ping().send().await;
        match response {
            Ok(resp) => Ok(resp.status_code().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn create_index(&self, index: &str, settings: Option<Value>, mappings: Option<Value>) -> SearchResult<bool> {
        let mut body = serde_json::json!({});
        if let Some(s) = settings {
            body["settings"] = s;
        }
        if let Some(m) = mappings {
            body["mappings"] = m;
        }
        
        let response = self.client.indices().create(elasticsearch::indices::IndicesCreateParts::Index(index))
            .body(body)
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to create index: {}", e)))?;
        
        Ok(response.status_code().is_success())
    }

    async fn delete_index(&self, index: &str) -> SearchResult<bool> {
        let response = self.client.indices().delete(elasticsearch::indices::IndicesDeleteParts::Index(index))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to delete index: {}", e)))?;
        
        Ok(response.status_code().is_success())
    }

    async fn index_exists(&self, index: &str) -> SearchResult<bool> {
        let response = self.client.indices().exists(elasticsearch::indices::IndicesExistsParts::Index(index))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to check index existence: {}", e)))?;
        
        Ok(response.status_code().is_success())
    }

    async fn get_index(&self, index: &str) -> SearchResult<Value> {
        let response = self.client.indices().get(elasticsearch::indices::IndicesGetParts::Index(index))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get index: {}", e)))?;
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse index response: {}", e)))?;
        
        Ok(json)
    }

    async fn index_document(&self, index: &str, id: Option<&str>, document: &Value) -> SearchResult<String> {
        let parts = match id {
            Some(i) => IndexParts::IndexId(index, i),
            None => IndexParts::Index(index),
        };
        
        let response = self.client.index(parts)
            .body(document)
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to index document: {}", e)))?;
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse index response: {}", e)))?;
        
        let doc_id = json["_id"].as_str().ok_or_else(|| wae_types::WaeError::internal("Invalid document ID in response"))?;
        
        Ok(doc_id.to_string())
    }

    async fn get_document(&self, index: &str, id: &str) -> SearchResult<Option<Value>> {
        let response = self.client.get(GetParts::IndexId(index, id))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to get document: {}", e)))?;
        
        if !response.status_code().is_success() {
            return Ok(None);
        }
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse document response: {}", e)))?;
        
        if json["found"].as_bool() == Some(true) {
            Ok(Some(json["_source"].clone()))
        } else {
            Ok(None)
        }
    }

    async fn update_document(&self, index: &str, id: &str, doc: &Value) -> SearchResult<bool> {
        let body = serde_json::json!({ "doc": doc });
        
        let response = self.client.update(UpdateParts::IndexId(index, id))
            .body(body)
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to update document: {}", e)))?;
        
        Ok(response.status_code().is_success())
    }

    async fn delete_document(&self, index: &str, id: &str) -> SearchResult<bool> {
        let response = self.client.delete(DeleteParts::IndexId(index, id))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to delete document: {}", e)))?;
        
        Ok(response.status_code().is_success())
    }

    async fn document_exists(&self, index: &str, id: &str) -> SearchResult<bool> {
        let response = self.client.exists(GetParts::IndexId(index, id))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to check document existence: {}", e)))?;
        
        Ok(response.status_code().is_success())
    }

    async fn bulk(&self, index: &str, operations: Vec<BulkOperation>) -> SearchResult<BulkResponse> {
        let mut bulk_ops = Vec::new();
        
        for op in operations {
            match op {
                BulkOperation::Index { id, source } => {
                    let mut index_op = serde_json::json!({ "index": {} });
                    if let Some(i) = id {
                        index_op["index"]["_id"] = i.into();
                    }
                    bulk_ops.push(index_op);
                    bulk_ops.push(source);
                }
                BulkOperation::Create { id, source } => {
                    bulk_ops.push(serde_json::json!({ "create": { "_id": id } }));
                    bulk_ops.push(source);
                }
                BulkOperation::Update { id, doc } => {
                    bulk_ops.push(serde_json::json!({ "update": { "_id": id } }));
                    bulk_ops.push(serde_json::json!({ "doc": doc }));
                }
                BulkOperation::Delete { id } => {
                    bulk_ops.push(serde_json::json!({ "delete": { "_id": id } }));
                }
            }
        }
        
        let response = self.client.bulk(BulkParts::Index(index))
            .body(bulk_ops)
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to execute bulk operation: {}", e)))?;
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse bulk response: {}", e)))?;
        
        let took = json["took"].as_u64().unwrap_or(0);
        let has_errors = json["errors"].as_bool().unwrap_or(false);
        
        let mut items = Vec::new();
        if let Some(items_array) = json["items"].as_array() {
            for item in items_array {
                for (op_type, result) in item.as_object().unwrap() {
                    let success = result["error"].is_null();
                    items.push(BulkItemResult {
                        operation: op_type.to_string(),
                        id: result["_id"].as_str().unwrap_or("").to_string(),
                        index: result["_index"].as_str().unwrap_or("").to_string(),
                        success,
                        error: result["error"].as_str().map(|s| s.to_string()),
                        version: result["_version"].as_u64(),
                    });
                }
            }
        }
        
        Ok(BulkResponse { has_errors, items, took })
    }

    async fn search<T: DeserializeOwned>(&self, index: &str, query: SearchQuery) -> SearchResult<SearchResponse<T>> {
        let mut body = serde_json::json!({ "query": query.query });
        
        if let Some(from) = query.from {
            body["from"] = from.into();
        }
        if let Some(size) = query.size {
            body["size"] = size.into();
        }
        if let Some(sort) = query.sort {
            body["sort"] = sort;
        }
        if let Some(aggs) = query.aggregations {
            body["aggregations"] = aggs;
        }
        
        let response = self.client.search(SearchParts::Index(&[index]))
            .body(body)
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to execute search: {}", e)))?;
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse search response: {}", e)))?;
        
        let took = json["took"].as_u64().unwrap_or(0);
        let timed_out = json["timed_out"].as_bool().unwrap_or(false);
        let total = json["hits"]["total"]["value"].as_u64().unwrap_or(0);
        
        let mut hits = Vec::new();
        if let Some(hits_array) = json["hits"]["hits"].as_array() {
            for hit in hits_array {
                let source: T = serde_json::from_value(hit["_source"].clone())
                    .map_err(|_| wae_types::WaeError::deserialization_failed(std::any::type_name::<T>()))?;
                
                hits.push(SearchHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    index: hit["_index"].as_str().unwrap_or("").to_string(),
                    score: hit["_score"].as_f64(),
                    source,
                });
            }
        }
        
        Ok(SearchResponse { total, hits, took, timed_out })
    }

    async fn delete_by_query(&self, index: &str, query: Value) -> SearchResult<u64> {
        let response = self.client.delete_by_query(elasticsearch::DeleteByQueryParts::Index(&[index]))
            .body(serde_json::json!({ "query": query }))
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to execute delete by query: {}", e)))?;
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse delete by query response: {}", e)))?;
        
        Ok(json["deleted"].as_u64().unwrap_or(0))
    }

    async fn update_by_query(&self, index: &str, query: Value, script: Value) -> SearchResult<u64> {
        let body = serde_json::json!({
            "query": query,
            "script": script
        });
        
        let response = self.client.update_by_query(elasticsearch::UpdateByQueryParts::Index(&[index]))
            .body(body)
            .send()
            .await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to execute update by query: {}", e)))?;
        
        let json: Value = response.json().await
            .map_err(|e| wae_types::WaeError::internal(format!("Failed to parse update by query response: {}", e)))?;
        
        Ok(json["updated"].as_u64().unwrap_or(0))
    }

    fn config(&self) -> &SearchConfig {
        &self.config
    }
}

/// 便捷函数：创建 Elasticsearch 搜索服务
pub fn elasticsearch_search(es_config: ElasticsearchConfig, search_config: SearchConfig) -> SearchResult<crate::SearchService> {
    let backend = ElasticsearchBackend::new(es_config, search_config)?;
    Ok(crate::SearchService::new(Box::new(backend)))
}
