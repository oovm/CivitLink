# WAE Session - Session 管理模块

提供完整的 HTTP Session 管理功能，包括：
- Session 存储 trait 定义
- 内存存储实现
- Session 中间件
- Session 配置
- axum 提取器

## 示例

```rust,ignore
use wae_session::{SessionLayer, MemorySessionStore, SessionConfig, SessionExtractor};
use axum::Router;

let store = MemorySessionStore::new();
let config = SessionConfig::default();
let layer = SessionLayer::new(store, config);

let app = Router::new()
    .route("/", get(handler))
    .layer(layer);

async fn handler(session: SessionExtractor) -> impl IntoResponse {
    let user_id: Option<String> = session.get_typed("user_id").await;
    format!("User ID: {:?}", user_id)
}
```
