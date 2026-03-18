//! 消息消费者实现

use super::{WaeResult, types::*};
use serde::de::DeserializeOwned;

/// 消息消费者后端 trait (dyn 兼容)
#[async_trait::async_trait]
pub trait ConsumerBackend: Send + Sync {
    /// 接收原始消息
    async fn receive_raw(&self) -> WaeResult<Option<ReceivedRawMessage>>;

    /// 确认消息
    async fn ack(&self, delivery_tag: u64) -> WaeResult<()>;

    /// 拒绝消息
    async fn nack(&self, delivery_tag: u64, requeue: bool) -> WaeResult<()>;

    /// 获取消费者配置
    fn config(&self) -> &ConsumerConfig;
}

/// 消息消费者 (提供泛型封装)
pub struct MessageConsumer {
    backend: Box<dyn ConsumerBackend>,
}

impl MessageConsumer {
    /// 从后端创建消费者
    pub fn new(backend: Box<dyn ConsumerBackend>) -> Self {
        Self { backend }
    }

    /// 接收消息
    pub async fn receive<T: DeserializeOwned + Send>(&self) -> WaeResult<Option<ReceivedMessage<T>>> {
        let raw = match self.backend.receive_raw().await? {
            Some(r) => r,
            None => return Ok(None),
        };

        let message = raw.message.into_typed()?;
        Ok(Some(ReceivedMessage { message, delivery_tag: raw.delivery_tag, redelivery_count: raw.redelivery_count }))
    }

    /// 确认消息
    pub async fn ack(&self, delivery_tag: u64) -> WaeResult<()> {
        self.backend.ack(delivery_tag).await
    }

    /// 拒绝消息
    pub async fn nack(&self, delivery_tag: u64, requeue: bool) -> WaeResult<()> {
        self.backend.nack(delivery_tag, requeue).await
    }

    /// 获取配置
    pub fn config(&self) -> &ConsumerConfig {
        self.backend.config()
    }
}
