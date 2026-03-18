# wae-event

事件驱动模块，提供统一的事件驱动能力抽象，支持事件发布/订阅、事件存储和事件回放。

## 概述

wae-event 深度融合 tokio 运行时，所有 API 都是异步优先设计，适合微服务架构。支持事件溯源、CQRS 等模式，提供内存存储实现用于开发和测试。

## 快速开始

最小可运行的事件发布订阅示例：

```rust
use wae::event::{
    BaseEvent, EventBus, EventBusConfig, EventData,
    AsyncEventHandler, memory_event_store, event_bus_with_store,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderCreated {
    order_id: String,
    amount: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = memory_event_store();
    let config = EventBusConfig::default();
    let bus = event_bus_with_store(config, store);

    let handler = AsyncEventHandler::new(
        vec!["order.created".to_string()],
        |event: EventData| {
            Box::pin(async move {
                let order: OrderCreated = event.into_event()?;
                println!("订单创建: {} - ¥{}", order.order_id, order.amount);
                Ok(())
            })
        },
    );

    bus.subscribe(vec!["order.created".to_string()], handler)?;

    let event = BaseEvent::new("order.created", OrderCreated {
        order_id: "ORD-001".to_string(),
        amount: 99.99,
    });

    bus.publish(&event)?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    bus.shutdown().await?;

    Ok(())
}
```

## 核心用法

### 定义事件

使用 `BaseEvent` 快速定义事件，payload 只需实现 `Serialize + DeserializeOwned`：

```rust
use wae::event::BaseEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserCreated {
    user_id: String,
    username: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PaymentCompleted {
    payment_id: String,
    order_id: String,
    amount: f64,
}

let user_event = BaseEvent::new("user.created", UserCreated {
    user_id: "u-001".to_string(),
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
});

let payment_event = BaseEvent::new("payment.completed", PaymentCompleted {
    payment_id: "p-001".to_string(),
    order_id: "ORD-001".to_string(),
    amount: 199.99,
});
```

### 发布事件

事件总线支持同步和异步两种发布方式：

```rust
use wae::event::{event_bus, EventBusConfig, BaseEvent};

let bus = event_bus(EventBusConfig::default());

bus.publish(&event)?;

bus.publish_async(&event).await?;
```

发布时可以附加元数据，用于追踪和过滤：

```rust
use wae::event::EventData;

let event_data = EventData::new(&event)?
    .with_metadata("source", "web-api")
    .with_metadata("trace_id", "abc-123")
    .with_metadata("version", "1.0");

bus.publish(&event_data)?;
```

### 订阅事件

#### 异步处理器

使用 `AsyncEventHandler` 处理异步逻辑：

```rust
use wae::event::{AsyncEventHandler, EventData};

let handler = AsyncEventHandler::new(
    vec!["user.created".to_string()],
    |event: EventData| {
        Box::pin(async move {
            let user: UserCreated = event.into_event()?;
            send_welcome_email(&user.email).await?;
            Ok(())
        })
    },
);

bus.subscribe(vec!["user.created".to_string()], handler)?;
```

#### 同步处理器

使用 `SyncEventHandlerWrapper` 处理同步逻辑：

```rust
use wae::event::{SyncEventHandlerWrapper, EventData};

let handler = SyncEventHandlerWrapper::new(
    vec!["user.deleted".to_string()],
    |event: EventData| {
        let user: UserDeleted = event.into_event()?;
        cleanup_user_data(&user.user_id);
        Ok(())
    },
);

bus.subscribe(vec!["user.deleted".to_string()], handler)?;
```

#### 订阅多种事件类型

一个处理器可以订阅多种事件类型：

```rust
let handler = AsyncEventHandler::new(
    vec!["order.created".to_string(), "order.updated".to_string()],
    |event: EventData| {
        Box::pin(async move {
            match event.event_type() {
                "order.created" => {
                    let order: OrderCreated = event.into_event()?;
                    println!("新订单: {}", order.order_id);
                }
                "order.updated" => {
                    let order: OrderUpdated = event.into_event()?;
                    println!("订单更新: {}", order.order_id);
                }
                _ => {}
            }
            Ok(())
        })
    },
);

bus.subscribe(
    vec!["order.created".to_string(), "order.updated".to_string()],
    handler,
)?;
```

### 事件过滤

使用过滤器只处理符合条件的事件：

#### 按类型过滤

```rust
use wae::event::TypeEventFilter;

let filter = TypeEventFilter::new(vec![
    "user.created".to_string(),
    "user.updated".to_string(),
]);

bus.subscribe_with_filter(
    vec!["user.*".to_string()],
    handler,
    Some(filter),
)?;
```

#### 按元数据过滤

```rust
use wae::event::MetadataEventFilter;

let filter = MetadataEventFilter::new("priority", "high");

bus.subscribe_with_filter(
    vec!["task.assigned".to_string()],
    handler,
    Some(filter),
)?;
```

#### 组合过滤

```rust
let source_filter = MetadataEventFilter::new("source", "web-api");
let version_filter = MetadataEventFilter::new("version", "2.0");

bus.subscribe_with_filter(
    vec!["order.*".to_string()],
    handler,
    Some(source_filter),
)?;
```

## 事件存储

### 创建带存储的事件总线

```rust
use wae::event::{event_bus_with_store, memory_event_store, EventBusConfig};

let store = memory_event_store();
let bus = event_bus_with_store(EventBusConfig::default(), store.clone());
```

### 事件持久化

启用存储后，所有发布的事件会自动持久化：

```rust
let mut config = EventBusConfig::default();
config.enable_store = true;
let bus = event_bus_with_store(config, store);
```

### 事件回放

从存储中回放历史事件：

```rust
use wae::event::AsyncEventHandler;

let replay_handler = AsyncEventHandler::new(
    vec!["order.created".to_string(), "order.updated".to_string()],
    |event: EventData| {
        Box::pin(async move {
            println!("回放事件: {} [{}]", event.id(), event.event_type());
            Ok(())
        })
    },
);

let count = bus.replay_events(&store, &replay_handler, Some("order.created")).await?;
println!("已回放 {} 个事件", count);
```

### 查询历史事件

直接查询存储中的事件：

```rust
let all_events = store.get_all_events().await?;

let order_events = store.get_events("order.created").await?;

let time_range_events = store.get_events_by_time(
    start_timestamp,
    end_timestamp,
).await?;

let total = store.count().await?;
```

## 最佳实践

### 事件命名规范

使用领域驱动的命名方式，格式为 `<聚合>.<动作>`：

```rust
"order.created"
"order.cancelled"
"payment.completed"
"user.email.verified"
"inventory.stock.depleted"
```

### 事件结构设计

事件应包含足够的上下文信息，避免消费者需要额外查询：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderCreated {
    order_id: String,
    user_id: String,
    items: Vec<OrderItem>,
    total_amount: f64,
    created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrderItem {
    product_id: String,
    product_name: String,
    quantity: u32,
    unit_price: f64,
}
```

### 处理器幂等性

事件可能被重复投递，处理器应实现幂等性：

```rust
let handler = AsyncEventHandler::new(
    vec!["order.created".to_string()],
    |event: EventData| {
        Box::pin(async move {
            let order: OrderCreated = event.into_event()?;

            if order_exists(&order.order_id).await? {
                return Ok(());
            }

            create_order(&order).await?;
            Ok(())
        })
    },
);
```

### 错误处理

处理器返回错误时，事件总线会根据配置进行重试：

```rust
let mut config = EventBusConfig::default();
config.max_retries = 3;
config.retry_interval = Duration::from_millis(100);
config.handler_timeout = Duration::from_secs(30);
```

### 优雅关闭

应用退出时应正确关闭事件总线：

```rust
bus.shutdown().await?;
```

### 测试策略

使用内存存储进行单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_order_created_event() {
        let store = memory_event_store();
        let bus = event_bus_with_store(EventBusConfig::default(), store.clone());

        let received = Arc::new(AtomicBool::new(false));
        let received_clone = received.clone();

        let handler = AsyncEventHandler::new(
            vec!["order.created".to_string()],
            move |_event: EventData| {
                let received = received_clone.clone();
                Box::pin(async move {
                    received.store(true, Ordering::SeqCst);
                    Ok(())
                })
            },
        );

        bus.subscribe(vec!["order.created".to_string()], handler).unwrap();
        bus.publish(&BaseEvent::new("order.created", OrderCreated {
            order_id: "test".to_string(),
            amount: 0.0,
        })).unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(received.load(Ordering::SeqCst));
    }
}
```

## 常见问题

### 事件处理器没有收到事件？

检查以下几点：

1. 确保订阅的事件类型与发布的事件类型完全匹配
2. 确保在发布事件之前完成订阅
3. 给异步处理留出足够时间，或使用 `tokio::time::sleep` 等待

```rust
bus.subscribe(vec!["user.created".to_string()], handler)?;

bus.publish(&event)?;

tokio::time::sleep(Duration::from_millis(100)).await;
```

### 如何实现事件优先级？

使用元数据标记优先级，配合过滤器实现：

```rust
let high_priority_event = EventData::new(&event)?
    .with_metadata("priority", "high");

let filter = MetadataEventFilter::new("priority", "high");
bus.subscribe_with_filter(event_types, handler, Some(filter))?;
```

### 如何处理事件顺序？

同一事件类型的事件按发布顺序处理。跨类型事件需要使用 saga 或流程管理器协调。

### 内存存储会丢失数据吗？

`InMemoryEventStore` 仅用于开发和测试。生产环境应实现 `EventStore` trait 并使用持久化存储（如 PostgreSQL、MongoDB）。

### 如何实现跨服务事件传递？

可以实现自定义的 `EventHandler`，将事件发送到消息队列：

```rust
let mq_handler = AsyncEventHandler::new(
    vec!["*".to_string()],
    |event: EventData| {
        Box::pin(async move {
            message_queue.publish(event).await?;
            Ok(())
        })
    },
);
```

### 事件总线性能如何？

默认配置下，单机吞吐量可达每秒数万事件。调优建议：

```rust
let config = EventBusConfig {
    queue_capacity: 10000,
    handler_timeout: Duration::from_secs(10),
    max_retries: 5,
    ..Default::default()
};
```
