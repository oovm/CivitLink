# wae-testing

测试支持模块，提供统一的测试工具集，包含 Mock 工具、断言扩展、数据生成器和测试环境管理。

## 概述

深度融合 tokio 运行时，支持异步测试场景。微服务架构友好，提供完整的测试基础设施。

## 快速开始

### 基础测试示例

```rust
use wae::testing::*;

#[test]
fn test_with_mock() -> TestingResult<()> {
    let mock = MockBuilder::new()
        .return_value(42)
        .expect(MockExpectation::new().times(1))
        .build();

    let result = mock.call(vec![])?;
    assert_eq!(result, 42);

    verify(&mock)?;
    Ok(())
}
```

### 异步测试示例

```rust
use wae::testing::*;

#[tokio::test]
async fn test_async_operations() -> TestingResult<()> {
    let env = TestEnvBuilder::new()
        .name("async_test")
        .timeout(Duration::from_secs(10))
        .build();

    env.run_test(|| async {
        let mock = MockBuilder::new()
            .sequence(vec!["first", "second", "third"])
            .build();

        assert_eq!(mock.call_async(vec![]).await?, "first");
        assert_eq!(mock.call_async(vec![]).await?, "second");

        assert_eventually(
            || async { mock.call_count() == 2 },
            Duration::from_secs(1),
            Duration::from_millis(50),
        ).await?;

        Ok(())
    }).await
}
```

## 核心用法

### Mock 工具

#### 创建 Mock 函数

```rust
use wae::testing::{MockBuilder, MockExpectation, verify};

let mock = MockBuilder::new()
    .return_value("success".to_string())
    .expect(MockExpectation::new().times(2))
    .build();

let result = mock.call(vec!["arg1".to_string()])?;
assert_eq!(result, "success");

verify(&mock)?;
```

#### 序列返回

每次调用返回不同的值：

```rust
let mock = MockBuilder::<i32>::new()
    .sequence(vec![1, 2, 3])
    .build();

assert_eq!(mock.call(vec![])?, 1);
assert_eq!(mock.call(vec![])?, 2);
assert_eq!(mock.call(vec![])?, 3);
```

#### 模拟错误

```rust
let mock = MockBuilder::<String>::new()
    .error("模拟的错误")
    .build();

let result = mock.call(vec![]);
assert!(result.is_err());
```

#### 设置调用期望

```rust
let mock = MockBuilder::new()
    .return_value(42)
    .expect(MockExpectation::new()
        .times(3)
        .description("应该被调用3次"))
    .build();

mock.call(vec![])?;
mock.call(vec![])?;
mock.call(vec![])?;

verify(&mock)?;
```

### 断言宏

#### Result 断言

```rust
let result: Result<i32, &str> = Ok(42);
let value = assert_ok!(result);

let result: Result<i32, &str> = Err("error");
let error = assert_err!(result);

let result: Result<i32, &str> = Ok(42);
let value = assert_ok!(result, "Should be Ok");
```

#### Option 断言

```rust
let option = Some(42);
let value = assert_some!(option);

let option: Option<i32> = None;
assert_none!(option);

let value = assert_some!(option, "Should be Some");
```

#### 集合断言

```rust
let vec = vec![1, 2, 3];
assert_contains!(vec, 2);
assert_contains!(vec, 2, "Vec should contain 2");
```

#### 浮点数近似相等

```rust
let a = 1.0001;
let b = 1.0002;
assert_approx_eq!(a, b, 0.001);
```

#### 异步断言

等待条件最终满足：

```rust
use wae::testing::assert_eventually;
use std::time::Duration;

assert_eventually(
    || async { some_async_condition().await },
    Duration::from_secs(5),
    Duration::from_millis(100),
).await?;
```

使用 AsyncAssert 获取值：

```rust
use wae::testing::AsyncAssert;

let value = AsyncAssert::eventually_with_value(
    || async { get_value_if_ready().await },
    Duration::from_secs(5),
    Duration::from_millis(100),
).await?;
```

#### 正则断言

```rust
use wae::testing::assert_matches_regex;

assert_matches_regex("hello@example.com", r"^\w+@\w+\.\w+$")?;
```

#### JSON 断言

```rust
use wae::testing::assert_json_contains;
use serde_json::json;

let json = json!({
    "user": {
        "name": "Alice",
        "age": 30
    }
});

assert_json_contains(&json, "user.name")?;
```

### Fixture 数据生成器

#### 随机字符串

```rust
use wae::testing::RandomString;

let random = RandomString::new()
    .length(16)
    .prefix("user_")
    .suffix("_test")
    .generate()?;
```

#### 随机数字

```rust
use wae::testing::RandomNumber;

let age = RandomNumber::new(18, 65).generate()?;
let price = RandomNumber::new(0.0, 1000.0).generate()?;
```

#### 随机邮箱

```rust
use wae::testing::RandomEmail;

let email = RandomEmail::new()
    .domain("company.com")
    .generate()?;
```

#### 随机 UUID

```rust
use wae::testing::RandomUuid;

let uuid_v4 = RandomUuid::new().generate()?;
let uuid_v7 = RandomUuid::new().version7().generate()?;
let uuid_str = RandomUuid::new().generate_string()?;
```

#### 随机布尔值

```rust
use wae::testing::RandomBool;

let coin_flip = RandomBool::new().generate()?;
let likely_true = RandomBool::new().probability(0.9).generate()?;
```

#### 随机选择

```rust
use wae::testing::RandomChoice;

let status = RandomChoice::new(vec!["active", "inactive", "pending"]).generate()?;
```

#### 随机日期时间

```rust
use wae::testing::RandomDateTime;

let timestamp = RandomDateTime::new()
    .range(1609459200, 1640995200)
    .generate_timestamp()?;
```

#### 批量生成

```rust
use wae::testing::FixtureGenerator;

let usernames = FixtureGenerator::strings(10, 8)?;
let emails = FixtureGenerator::emails(5, "test.com")?;
let ids = FixtureGenerator::uuids(3)?;
let ages = FixtureGenerator::i32_numbers(20, 18, 65)?;
```

### TestEnv 测试环境管理

#### 基础使用

```rust
use wae::testing::{TestEnv, TestEnvConfig};

let env = TestEnv::new(TestEnvConfig::new()
    .name("integration_test")
    .with_logging(true)
    .timeout(Duration::from_secs(60)));

env.setup()?;

env.set("user_id", 12345);
let user_id: Option<i32> = env.get("user_id");

env.teardown()?;
```

#### 使用构建器

```rust
use wae::testing::TestEnvBuilder;

let env = TestEnvBuilder::new()
    .name("api_test")
    .with_tracing(true)
    .custom("api_url", "http://localhost:8080")
    .build();
```

#### 运行异步测试

```rust
let env = create_test_env();

env.run_test(|| async {
    let result = call_api().await?;
    assert_ok!(result);
    Ok(())
}).await?;
```

#### 注册清理函数

```rust
let env = TestEnv::default_env();
env.setup()?;

env.on_cleanup(|| {
    cleanup_temp_files();
});

env.teardown()?;
```

## 最佳实践

### 组织测试代码

```rust
mod tests {
    use super::*;
    use wae::testing::*;

    fn setup_env() -> TestEnv {
        TestEnvBuilder::new()
            .name("unit_test")
            .with_logging(false)
            .build()
    }

    #[test]
    fn test_user_creation() -> TestingResult<()> {
        let env = setup_env();
        env.setup()?;

        let user = create_test_user(&env)?;
        assert_some!(user);

        env.teardown()?;
        Ok(())
    }
}
```

### 使用 Fixture 生成测试数据

```rust
fn create_test_user(env: &TestEnv) -> TestingResult<User> {
    let email = RandomEmail::new().domain("test.com").generate()?;
    let name = RandomString::new().length(8).generate()?;

    let user = User {
        id: RandomNumber::new(1, 10000).generate()?,
        name,
        email,
    };

    env.set("test_user", user.clone());
    Ok(user)
}
```

### 异步测试超时处理

```rust
#[tokio::test]
async fn test_with_timeout() -> TestingResult<()> {
    let env = TestEnvBuilder::new()
        .timeout(Duration::from_secs(5))
        .build();

    env.run_test(|| async {
        assert_eventually(
            || async { check_condition().await },
            Duration::from_secs(3),
            Duration::from_millis(100),
        ).await
    }).await
}
```

## 常见问题

### Q: Mock 验证失败怎么办？

确保调用了 `verify(&mock)` 来验证期望：

```rust
let mock = MockBuilder::new()
    .return_value(42)
    .expect(MockExpectation::new().times(2))
    .build();

mock.call(vec![])?;
mock.call(vec![])?;

verify(&mock)?;
```

### Q: 如何在测试间共享环境？

使用 `lazy_static` 或 `once_cell`：

```rust
use once_cell::sync::Lazy;

static TEST_ENV: Lazy<TestEnv> = Lazy::new(|| {
    TestEnvBuilder::new()
        .name("shared_test")
        .build()
});
```

### Q: 异步断言超时怎么办？

增加超时时间或检查间隔：

```rust
assert_eventually(
    || async { check_condition().await },
    Duration::from_secs(10),
    Duration::from_millis(200),
).await?;
```

### Q: 如何生成特定格式的测试数据？

组合使用多个生成器：

```rust
let username = RandomString::new()
    .length(8)
    .prefix("user_")
    .generate()?;

let email = format!("{}@test.com", username);
```
