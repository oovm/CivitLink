# wae-event

事件模块 - 提供事件驱动架构支持。

## 主要功能

- **事件发布/订阅**: 解耦的事件通信机制
- **异步处理**: 非阻塞事件处理
- **多订阅者**: 支持多个订阅者监听同一事件
- **类型安全**: 强类型事件定义

## 技术栈

- **异步运行时**: Tokio
- **序列化**: serde

## 使用示例

```rust
use wae_event::{EventBus, EventHandler, Event};

#[derive(Debug, Clone, Event)]
struct UserCreated {
    user_id: String,
    username: String,
}

struct SendWelcomeEmail;

impl EventHandler<UserCreated> for SendWelcomeEmail {
    async fn handle(&self, event: &UserCreated) {
        println!("发送欢迎邮件给: {}", event.username);
    }
}

#[tokio::main]
async fn main() {
    let bus = EventBus::new();
    
    bus.subscribe::<UserCreated>(SendWelcomeEmail);
    
    bus.publish(UserCreated {
        user_id: "001".to_string(),
        username: "张三".to_string(),
    }).await;
}
```

## 事件流程

1. 发布者调用 `publish` 发布事件
2. EventBus 查找所有订阅者
3. 异步执行所有订阅者的 `handle` 方法
