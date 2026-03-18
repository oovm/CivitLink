# wae-testing

测试模块 - 提供测试工具和辅助功能。

## 主要功能

- **测试夹具**: 测试数据准备
- **Mock 支持**: 服务 Mock
- **测试容器**: 集成测试容器
- **断言扩展**: 丰富的断言方法

## 技术栈

- **测试框架**: Rust 内置测试
- **Mock**: mockall (可选)
- **容器**: testcontainers (可选)

## 使用示例

```rust
use wae_testing::{TestFixture, MockDatabase};

#[TestFixture]
async fn test_user_creation() {
    let db = MockDatabase::new().await;
    let service = UserService::new(db);
    
    let user = service.create("张三", "zhangsan@example.com").await.unwrap();
    
    assert_eq!(user.username, "张三");
    assert!(db.contains_user(&user.id));
}

#[TestFixture]
async fn test_with_real_database() {
    let container = TestContainer::postgres().await;
    let db = DatabasePool::connect(container.url()).await.unwrap();
    
    let result = db.insert(&test_user).await;
    assert!(result.is_ok());
}
```

## Mock 服务

```rust
use wae_testing::mocks::{MockHttpClient, MockResponse};

let mock_client = MockHttpClient::new();
mock_client.expect_get("/users")
    .returning(|| MockResponse::json(vec![User::default()]));

let users = mock_client.get("/users").await.unwrap();
```
