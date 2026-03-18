# wae-types

类型模块 - 提供通用类型定义。

## 主要功能

- **错误类型**: 统一错误定义
- **结果类型**: 通用 Result 类型
- **ID 类型**: 类型安全的标识符
- **时间类型**: 时间相关类型

## 技术栈

- **序列化**: serde
- **错误处理**: thiserror

## 使用示例

```rust
use wae_types::{
    Result,
    Error,
    Id,
    Timestamp,
};

fn main() -> Result<()> {
    let user_id: Id<User> = Id::new("user-001");
    let order_id: Id<Order> = Id::new("order-001");
    
    let now = Timestamp::now();
    let expires_at = now.add_hours(24);
    
    if now > expires_at {
        return Err(Error::expired("令牌已过期"));
    }
    
    Ok(())
}
```

## 类型安全 ID

```rust
use wae_types::Id;

struct User { id: Id<User> }
struct Order { id: Id<Order> }

let user_id: Id<User> = Id::new("001");
let order_id: Id<Order> = Id::new("001");

fn get_user(id: Id<User>) { }
fn get_order(id: Id<Order>) { }

get_user(user_id);
get_order(order_id);
```

## 错误类型

```rust
use wae_types::Error;

match result {
    Err(Error::NotFound(msg)) => println!("未找到: {}", msg),
    Err(Error::Unauthorized) => println!("未授权"),
    Err(Error::Validation(msg)) => println!("验证失败: {}", msg),
    _ => {}
}
```
