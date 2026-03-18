# wae-search

搜索服务模块 - 提供统一的搜索能力抽象，支持 Elasticsearch 和 OpenSearch 后端。

## 主要功能

- **多后端支持**: 支持 Elasticsearch 和 OpenSearch
- **异步 API**: 全异步接口设计
- **索引管理**: 创建、删除、检查索引
- **文档操作**: CRUD、批量操作
- **搜索查询**: 支持完整的 DSL 查询
- **聚合**: 支持搜索聚合

## 技术栈

- **Elasticsearch**: elasticsearch (可选)
- **OpenSearch**: opensearch (可选)
- **异步运行时**: Tokio
- **序列化**: serde

## 使用示例

```rust
use wae_search::{SearchConfig, SearchService, SearchQuery};

#[tokio::main]
async fn main() {
    let config = SearchConfig {
        url: "http://localhost:9200".to_string(),
        ..Default::default()
    };
    
    let service = SearchService::new(Box::new(ElasticsearchBackend::new(config)));
    
    let doc = json!({
        "title": "Hello World",
        "content": "This is a test document"
    });
    
    let id = service.index_document("documents", None, &doc).await?;
    
    let query = SearchQuery {
        query: json!({
            "match": {
                "content": "test"
            }
        }),
        ..Default::default()
    };
    
    let results: SearchResponse<Document> = service.search("documents", query).await?;
}
```

## 特性标志

- `default`: 无默认特性
- `elasticsearch`: 启用 Elasticsearch 后端
- `opensearch`: 启用 OpenSearch 后端
- `full`: 启用所有功能
