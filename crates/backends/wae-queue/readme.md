# wae-queue

队列模块 - 提供消息队列抽象层。

## 主要功能

- **多后端支持**: 支持 Redis、RabbitMQ、内存队列
- **异步处理**: 生产者/消费者模式
- **延迟队列**: 支持延迟消息投递
- **死信队列**: 失败消息处理

## 技术栈

- **消息队列**: 多后端支持
- **异步运行时**: Tokio

## 使用示例

```rust
use wae_queue::{QueueClient, QueueConfig, Message};

#[tokio::main]
async fn main() {
    let queue = QueueClient::new(QueueConfig {
        backend: QueueBackend::Redis("redis://127.0.0.1:6379".to_string()),
    }).await?;
    
    queue.publish("tasks", &Task {
        id: "001".to_string(),
        action: "send_email".to_string(),
    }).await?;
    
    while let Some(message) = queue.consume("tasks").await? {
        let task: Task = message.deserialize()?;
        process_task(task).await;
        message.ack().await?;
    }
}
```

## 延迟队列

```rust
queue.publish_delayed("tasks", &task, Duration::from_secs(60)).await?;
```

## 支持的后端

| 后端 | 说明 |
|------|------|
| Memory | 内存队列 (开发测试) |
| Redis | Redis Streams |
| RabbitMQ | AMQP 协议 |
