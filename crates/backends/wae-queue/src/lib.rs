//! WAE Queue - 消息队列抽象层
//!
//! 提供统一的消息队列能力抽象，支持多种消息队列后端。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持消息确认、重试、延迟队列等特性。

#![warn(missing_docs)]

use std::time::Duration;

// 模块声明
mod backends;
mod consumers;
mod producers;
mod services;
mod types;

// 重新导出所有公共类型和结构体
pub use backends::memory::*;
pub use consumers::{ConsumerBackend, MessageConsumer};
pub use producers::{MessageProducer, ProducerBackend};
pub use services::{QueueManager, QueueService};
pub use types::*;
pub use wae_types::{WaeError, WaeResult};

#[cfg(feature = "redis-backend")]
pub use backends::redis::*;

#[cfg(feature = "kafka-backend")]
pub use backends::kafka::*;

/// 便捷函数：创建内存队列服务
pub fn memory_queue_service() -> MemoryQueueService {
    MemoryQueueService::new()
}
