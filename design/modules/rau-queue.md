# wae-queue

消息队列抽象层，提供统一的消息队列能力抽象，支持多种消息队列后端。

## 概述

`wae-queue` 是一个异步优先的消息队列抽象层，深度融合 tokio 运行时。它为微服务架构设计，支持消息确认、重试、延迟队列等特性。目前提供内存队列实现（`MemoryQueueService`），适用于开发测试环境，后续可扩展支持 RabbitMQ、Kafka 等后端。

核心概念：
- **QueueService** - 队列服务，用于创建生产者和消费者
- **MessageProducer** - 消息生产者，负责发送消息
- **MessageConsumer** - 消息消费者，负责接收和确认消息
- **QueueManager** - 队列管理器，负责队列的声明、删除、查询等管理操作

## 快速开始

一个最小化的生产-消费示例：

```rust
use wae_queue::{
    memory_queue_service, Message, QueueConfig, ProducerConfig, ConsumerConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Order {
    id: String,
    product: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = memory_queue_service();
    
    service.manager().declare_queue(&QueueConfig::new("orders")).await?;
    
    let producer = service.create_producer(ProducerConfig {
        default_queue: Some("orders".to_string()),
        ..Default::default()
    }).await?;
    
    let consumer = service.create_consumer(ConsumerConfig::new("orders")).await?;
    
    producer.send("orders", &Message::new(Order {
        id: "order-001".to_string(),
        product: "Widget".to_string(),
    })).await?;
    
    if let Some(received) = consumer.receive::<Order>().await? {
        println!("Received: {:?}", received.message.payload.id);
        consumer.ack(received.delivery_tag).await?;
    }
    
    Ok(())
}
```

## 核心用法

### 创建队列服务

使用 `memory_queue_service()` 创建内存队列服务：

```rust
use wae_queue::{memory_queue_service, QueueConfig};

let service = memory_queue_service();
let manager = service.manager();
```

### 声明队列

通过 `QueueConfig` 配置队列属性：

```rust
manager.declare_queue(&QueueConfig::new("orders")
    .durable(true)
    .message_ttl(86400000)
    .dead_letter_queue("orders.dead")).await?;
```

常用配置：
- `durable(true)` - 持久化队列（重启后保留）
- `message_ttl(ms)` - 消息过期时间（毫秒）
- `dead_letter_queue(name)` - 死信队列，过期或被拒绝的消息转入
- `max_messages(n)` - 队列最大消息数

### 发送消息

创建生产者并发送消息：

```rust
use wae_queue::{Message, ProducerConfig};

let producer = service.create_producer(ProducerConfig {
    default_queue: Some("orders".to_string()),
    confirm_timeout: Duration::from_secs(5),
    retry_count: 3,
    ..Default::default()
}).await?;

let message = Message::new(order)
    .with_priority(5)
    .with_correlation_id("req-123")
    .with_header("source", "web");

let message_id = producer.send("orders", &message).await?;
```

消息元数据构建器：
- `with_priority(u8)` - 消息优先级（0-9）
- `with_correlation_id(id)` - 关联 ID，用于请求-响应模式
- `with_reply_to(queue)` - 回复队列
- `with_expiration(ms)` - 消息过期时间
- `with_header(k, v)` - 自定义头

### 接收消息

创建消费者并接收消息：

```rust
use wae_queue::ConsumerConfig;

let consumer = service.create_consumer(ConsumerConfig::new("orders")
    .auto_ack(false)
    .prefetch(10)).await?;

while let Some(received) = consumer.receive::<Order>().await? {
    let order = received.message.payload;
    println!("Order: {}, redelivery count: {}", 
        order.id, received.redelivery_count);
    
    consumer.ack(received.delivery_tag).await?;
}
```

消费者配置：
- `auto_ack(false)` - 手动确认模式（推荐）
- `prefetch(n)` - 预取消息数，控制消费速率

### 消息确认

手动确认模式下，处理完成后必须显式确认：

```rust
match process_order(&order).await {
    Ok(_) => {
        consumer.ack(received.delivery_tag).await?;
    }
    Err(e) if received.redelivery_count < 3 => {
        consumer.nack(received.delivery_tag, true).await?;
    }
    Err(_) => {
        consumer.nack(received.delivery_tag, false).await?;
    }
}
```

- `ack(tag)` - 确认消息已处理
- `nack(tag, requeue)` - 拒绝消息；`requeue=true` 重新入队，`false` 进入死信队列

## 延迟队列

使用 `send_delayed` 发送延迟消息：

```rust
use std::time::Duration;

let message = Message::new(OrderCreated { order_id: "123" });

producer.send_delayed("orders.delayed", &message, Duration::from_secs(30)).await?;
```

典型场景：
- 订单超时取消（30分钟后检查订单状态）
- 延迟通知（用户注册后发送欢迎邮件）
- 重试退避（失败后延迟重试）

结合消息过期时间实现延迟消费：

```rust
let message = Message::new(order)
    .with_expiration(60000);

producer.send_delayed("orders", &message, Duration::from_secs(30)).await?;
```

## 批量操作

批量发送消息提高吞吐量：

```rust
let messages: Vec<Message<Order>> = orders
    .into_iter()
    .map(|o| Message::new(o).with_priority(3))
    .collect();

let message_ids = producer.send_batch("orders", &messages).await?;
println!("Sent {} messages", message_ids.len());
```

批量操作注意事项：
- 单次批量大小建议控制在 100-1000 条
- 批量发送失败时，已发送的消息无法回滚
- 适用于允许少量丢失的高吞吐场景

## 最佳实践

### 队列命名规范

```rust
QueueConfig::new("orders.created")
QueueConfig::new("orders.paid")
QueueConfig::new("notifications.email")
```

使用 `.` 分隔的命名空间，便于管理和监控。

### 消费者错误处理

```rust
async fn consume_orders(consumer: &MessageConsumer) -> QueueResult<()> {
    while let Some(received) = consumer.receive::<Order>().await? {
        match process_order(&received.message.payload).await {
            Ok(_) => {
                consumer.ack(received.delivery_tag).await?;
            }
            Err(e) => {
                tracing::error!("Process failed: {}", e);
                
                if received.redelivery_count >= 3 {
                    tracing::warn!("Max retries exceeded, sending to DLQ");
                    consumer.nack(received.delivery_tag, false).await?;
                } else {
                    consumer.nack(received.delivery_tag, true).await?;
                }
            }
        }
    }
    Ok(())
}
```

### 死信队列处理

```rust
manager.declare_queue(&QueueConfig::new("orders")
    .dead_letter_queue("orders.dead")).await?;

manager.declare_queue(&QueueConfig::new("orders.dead")).await?;

let dlq_consumer = service.create_consumer(
    ConsumerConfig::new("orders.dead")
).await?;

while let Some(received) = dlq_consumer.receive::<Order>().await? {
    tracing::error!("Dead letter: {:?}", received.message.payload);
    consumer.ack(received.delivery_tag).await?;
}
```

### 优雅关闭

```rust
use tokio::signal;

tokio::select! {
    _ = consume_orders(&consumer) => {}
    _ = signal::ctrl_c() => {
        tracing::info!("Shutting down...");
    }
}

service.close().await?;
```

## 常见问题

### Q: 消息丢失怎么办？

确保：
1. 使用 `auto_ack(false)` 手动确认模式
2. 处理成功后调用 `ack()`
3. 配置 `dead_letter_queue` 捕获失败消息

### Q: 如何保证消息顺序？

同一队列内的消息按发送顺序消费。如需严格顺序：
1. 单消费者消费（`prefetch(1)`）
2. 使用消息优先级时注意可能影响顺序

### Q: 消费者如何实现重试？

```rust
if processing_failed {
    if received.redelivery_count < MAX_RETRIES {
        consumer.nack(delivery_tag, true).await?;
    } else {
        consumer.nack(delivery_tag, false).await?;
    }
}
```

### Q: 如何实现请求-响应模式？

```rust
let reply_queue = format!("replies.{}", request_id);

producer.send("requests", &Message::new(request)
    .with_correlation_id(&request_id)
    .with_reply_to(&reply_queue)).await?;

let consumer = service.create_consumer(ConsumerConfig::new(&reply_queue)).await?;

let response = consumer.receive::<Response>().await?;
```

### Q: 内存队列适合生产环境吗？

`MemoryQueueService` 适用于：
- 单元测试和集成测试
- 本地开发环境
- 临时性、可丢失的消息

生产环境建议等待后续 RabbitMQ 或 Kafka 后端实现。

### Q: 如何监控队列状态？

```rust
let count = manager.queue_message_count("orders").await?;
println!("Pending messages: {}", count);

let exists = manager.queue_exists("orders").await?;
```
