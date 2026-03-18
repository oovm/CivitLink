//! 队列管理和服务实现

use super::{WaeResult, consumers::MessageConsumer, producers::MessageProducer, types::*};

/// 队列管理 trait
#[async_trait::async_trait]
pub trait QueueManager: Send + Sync {
    /// 声明队列
    async fn declare_queue(&self, config: &QueueConfig) -> WaeResult<()>;

    /// 删除队列
    async fn delete_queue(&self, name: &str) -> WaeResult<()>;

    /// 检查队列是否存在
    async fn queue_exists(&self, name: &str) -> WaeResult<bool>;

    /// 获取队列消息数量
    async fn queue_message_count(&self, name: &str) -> WaeResult<u64>;

    /// 清空队列
    async fn purge_queue(&self, name: &str) -> WaeResult<u64>;
}

/// 消息队列服务 trait

pub trait QueueService: Send + Sync {
    /// 创建生产者
    async fn create_producer(&self, config: ProducerConfig) -> WaeResult<MessageProducer>;

    /// 创建消费者
    async fn create_consumer(&self, config: ConsumerConfig) -> WaeResult<MessageConsumer>;

    /// 获取队列管理器
    fn manager(&self) -> &dyn QueueManager;

    /// 关闭连接
    async fn close(&self) -> WaeResult<()>;
}
