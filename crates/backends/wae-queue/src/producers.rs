//! 消息生产者实现

use super::{WaeResult, types::*};
use serde::Serialize;
use std::time::Duration;

/// 消息生产者后端 trait (dyn 兼容)
#[async_trait::async_trait]
pub trait ProducerBackend: Send + Sync {
    /// 发送原始消息到指定队列
    async fn send_raw(&self, queue: &str, message: &RawMessage) -> WaeResult<MessageId>;

    /// 发送原始消息到默认队列
    async fn send_raw_default(&self, message: &RawMessage) -> WaeResult<MessageId>;

    /// 发送延迟消息
    async fn send_raw_delayed(&self, queue: &str, message: &RawMessage, delay: Duration) -> WaeResult<MessageId>;

    /// 批量发送消息
    async fn send_raw_batch(&self, queue: &str, messages: &[RawMessage]) -> WaeResult<Vec<MessageId>>;

    /// 获取生产者配置
    fn config(&self) -> &ProducerConfig;
}

/// 消息生产者 (提供泛型封装)
pub struct MessageProducer {
    backend: Box<dyn ProducerBackend>,
}

impl MessageProducer {
    /// 从后端创建生产者
    pub fn new(backend: Box<dyn ProducerBackend>) -> Self {
        Self { backend }
    }

    /// 发送消息到指定队列
    pub async fn send<T: Serialize + Send + Sync>(&self, queue: &str, message: &Message<T>) -> WaeResult<MessageId> {
        let raw = message.to_raw()?;
        self.backend.send_raw(queue, &raw).await
    }

    /// 发送消息到默认队列
    pub async fn send_default<T: Serialize + Send + Sync>(&self, message: &Message<T>) -> WaeResult<MessageId> {
        let raw = message.to_raw()?;
        self.backend.send_raw_default(&raw).await
    }

    /// 发送延迟消息
    pub async fn send_delayed<T: Serialize + Send + Sync>(
        &self,
        queue: &str,
        message: &Message<T>,
        delay: Duration,
    ) -> WaeResult<MessageId> {
        let raw = message.to_raw()?;
        self.backend.send_raw_delayed(queue, &raw, delay).await
    }

    /// 批量发送消息
    pub async fn send_batch<T: Serialize + Send + Sync>(
        &self,
        queue: &str,
        messages: &[Message<T>],
    ) -> WaeResult<Vec<MessageId>> {
        let raw_messages: Vec<RawMessage> = messages.iter().map(|m| m.to_raw()).collect::<WaeResult<_>>()?;
        self.backend.send_raw_batch(queue, &raw_messages).await
    }

    /// 获取配置
    pub fn config(&self) -> &ProducerConfig {
        self.backend.config()
    }
}
