//! 内存队列实现

use super::super::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::RwLock;

/// 待处理消息状态
struct PendingMessage {
    data: Vec<u8>,
    metadata: MessageMetadata,
    redelivery_count: u32,
    delivery_tag: u64,
}

/// 内存队列存储
struct QueueStorage {
    messages: VecDeque<(Vec<u8>, MessageMetadata)>,
    pending_messages: HashMap<u64, PendingMessage>,
    next_delivery_tag: u64,
}

impl QueueStorage {
    fn new() -> Self {
        Self { messages: VecDeque::new(), pending_messages: HashMap::new(), next_delivery_tag: 1 }
    }
}

/// 内存队列管理器
pub struct MemoryQueueManager {
    queues: Arc<RwLock<HashMap<String, QueueStorage>>>,
    configs: Arc<RwLock<HashMap<String, QueueConfig>>>,
}

impl MemoryQueueManager {
    /// 创建新的内存队列管理器
    pub fn new() -> Self {
        Self { queues: Arc::new(RwLock::new(HashMap::new())), configs: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl Default for MemoryQueueManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl QueueManager for MemoryQueueManager {
    async fn declare_queue(&self, config: &QueueConfig) -> WaeResult<()> {
        let mut queues = self.queues.write().await;
        let mut configs = self.configs.write().await;

        if !queues.contains_key(&config.name) {
            queues.insert(config.name.clone(), QueueStorage::new());
        }
        configs.insert(config.name.clone(), config.clone());
        Ok(())
    }

    async fn delete_queue(&self, name: &str) -> WaeResult<()> {
        let mut queues = self.queues.write().await;
        let mut configs = self.configs.write().await;
        queues.remove(name);
        configs.remove(name);
        Ok(())
    }

    async fn queue_exists(&self, name: &str) -> WaeResult<bool> {
        let queues = self.queues.read().await;
        Ok(queues.contains_key(name))
    }

    async fn queue_message_count(&self, name: &str) -> WaeResult<u64> {
        let queues = self.queues.read().await;
        Ok(queues.get(name).map(|q| q.messages.len() as u64 + q.pending_messages.len() as u64).unwrap_or(0))
    }

    async fn purge_queue(&self, name: &str) -> WaeResult<u64> {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(name) {
            let count = queue.messages.len() as u64 + queue.pending_messages.len() as u64;
            queue.messages.clear();
            queue.pending_messages.clear();
            return Ok(count);
        }
        Ok(0)
    }
}

/// 内存生产者后端
pub struct MemoryProducerBackend {
    config: ProducerConfig,
    queues: Arc<RwLock<HashMap<String, QueueStorage>>>,
    manager: Arc<MemoryQueueManager>,
}

impl MemoryProducerBackend {
    /// 创建新的内存生产者后端
    pub fn new(config: ProducerConfig, manager: Arc<MemoryQueueManager>) -> Self {
        Self { config, queues: manager.queues.clone(), manager }
    }

    /// 内部发送消息到指定队列（不声明队列）
    async fn send_raw_internal(&self, queue: &str, data: Vec<u8>, mut metadata: MessageMetadata) -> WaeResult<MessageId> {
        let id = uuid::Uuid::new_v4().to_string();
        metadata.id = Some(id.clone());
        metadata.timestamp =
            Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64);

        let mut queues = self.queues.write().await;
        if let Some(q) = queues.get_mut(queue) {
            q.messages.push_back((data, metadata));
        }
        Ok(id)
    }
}

#[async_trait::async_trait]
impl ProducerBackend for MemoryProducerBackend {
    async fn send_raw(&self, queue: &str, message: &RawMessage) -> WaeResult<MessageId> {
        self.manager.declare_queue(&QueueConfig::new(queue)).await?;

        let id = uuid::Uuid::new_v4().to_string();
        let mut metadata = message.metadata.clone();
        metadata.id = Some(id.clone());
        metadata.timestamp =
            Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64);

        let mut queues = self.queues.write().await;
        if let Some(q) = queues.get_mut(queue) {
            q.messages.push_back((message.data.clone(), metadata));
        }
        Ok(id)
    }

    async fn send_raw_default(&self, message: &RawMessage) -> WaeResult<MessageId> {
        let queue = self.config.default_queue.as_ref().ok_or_else(|| WaeError::config_missing("default_queue"))?;
        self.send_raw(queue, message).await
    }

    async fn send_raw_delayed(&self, queue: &str, message: &RawMessage, delay: Duration) -> WaeResult<MessageId> {
        tokio::time::sleep(delay).await;
        self.send_raw(queue, message).await
    }

    async fn send_raw_batch(&self, queue: &str, messages: &[RawMessage]) -> WaeResult<Vec<MessageId>> {
        let mut ids = Vec::with_capacity(messages.len());
        for msg in messages {
            ids.push(self.send_raw(queue, msg).await?);
        }
        Ok(ids)
    }

    fn config(&self) -> &ProducerConfig {
        &self.config
    }
}

/// 内存消费者后端
pub struct MemoryConsumerBackend {
    config: ConsumerConfig,
    queues: Arc<RwLock<HashMap<String, QueueStorage>>>,
    manager: Arc<MemoryQueueManager>,
}

impl MemoryConsumerBackend {
    /// 创建新的内存消费者后端
    pub fn new(config: ConsumerConfig, manager: Arc<MemoryQueueManager>) -> Self {
        Self { config, queues: manager.queues.clone(), manager: manager.clone() }
    }
}

#[async_trait::async_trait]
impl ConsumerBackend for MemoryConsumerBackend {
    async fn receive_raw(&self) -> WaeResult<Option<ReceivedRawMessage>> {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(&self.config.queue) {
            if let Some((data, metadata)) = queue.messages.pop_front() {
                let delivery_tag = queue.next_delivery_tag;
                queue.next_delivery_tag += 1;

                let redelivery_count = metadata.headers.get("x-redelivery-count").and_then(|s| s.parse().ok()).unwrap_or(0);

                let pending_msg =
                    PendingMessage { data: data.clone(), metadata: metadata.clone(), redelivery_count, delivery_tag };
                queue.pending_messages.insert(delivery_tag, pending_msg);

                let message = RawMessage { data, metadata };
                return Ok(Some(ReceivedRawMessage { message, delivery_tag, redelivery_count }));
            }
        }
        Ok(None)
    }

    async fn ack(&self, delivery_tag: u64) -> WaeResult<()> {
        let mut queues = self.queues.write().await;
        if let Some(queue) = queues.get_mut(&self.config.queue) {
            queue.pending_messages.remove(&delivery_tag);
        }
        Ok(())
    }

    async fn nack(&self, delivery_tag: u64, requeue: bool) -> WaeResult<()> {
        let configs = self.manager.configs.read().await;
        let queue_config = configs.get(&self.config.queue).cloned();
        drop(configs);

        let mut queues = self.queues.write().await;

        if let Some(queue) = queues.get_mut(&self.config.queue) {
            if let Some(mut pending_msg) = queue.pending_messages.remove(&delivery_tag) {
                if requeue {
                    pending_msg.redelivery_count += 1;
                    pending_msg
                        .metadata
                        .headers
                        .insert("x-redelivery-count".to_string(), pending_msg.redelivery_count.to_string());

                    queue.messages.push_back((pending_msg.data, pending_msg.metadata));
                }
                else {
                    if let Some(dlq_name) = queue_config.and_then(|c| c.dead_letter_queue) {
                        drop(queues);

                        self.manager.declare_queue(&QueueConfig::new(&dlq_name)).await?;

                        let mut queues = self.queues.write().await;
                        if let Some(dlq) = queues.get_mut(&dlq_name) {
                            dlq.messages.push_back((pending_msg.data, pending_msg.metadata));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn config(&self) -> &ConsumerConfig {
        &self.config
    }
}

/// 内存队列服务
pub struct MemoryQueueService {
    manager: Arc<MemoryQueueManager>,
}

impl MemoryQueueService {
    /// 创建新的内存队列服务
    pub fn new() -> Self {
        Self { manager: Arc::new(MemoryQueueManager::new()) }
    }
}

impl Default for MemoryQueueService {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueService for MemoryQueueService {
    async fn create_producer(&self, config: ProducerConfig) -> WaeResult<MessageProducer> {
        Ok(MessageProducer::new(Box::new(MemoryProducerBackend::new(config, self.manager.clone()))))
    }

    async fn create_consumer(&self, config: ConsumerConfig) -> WaeResult<MessageConsumer> {
        self.manager.declare_queue(&QueueConfig::new(&config.queue)).await?;
        Ok(MessageConsumer::new(Box::new(MemoryConsumerBackend::new(config, self.manager.clone()))))
    }

    fn manager(&self) -> &dyn QueueManager {
        self.manager.as_ref() as &dyn QueueManager
    }

    async fn close(&self) -> WaeResult<()> {
        Ok(())
    }
}
