//! Redis 队列实现

use super::super::*;
use ::redis::{AsyncCommands, Client, FromRedisValue, RedisResult, streams::StreamReadOptions};
use base64::{Engine as _, engine::general_purpose};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Redis 队列管理器
pub struct RedisQueueManager {
    client: Client,
    configs: Arc<Mutex<HashMap<String, QueueConfig>>>,
}

impl RedisQueueManager {
    /// 创建新的 Redis 队列管理器
    pub fn new(client: Client) -> Self {
        Self { client, configs: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// 从 URL 创建 Redis 队列管理器
    pub async fn from_url(url: &str) -> WaeResult<Self> {
        let client = Client::open(url).map_err(|e| WaeError::internal(format!("Failed to create Redis client: {}", e)))?;
        Ok(Self::new(client))
    }

    fn stream_name(queue: &str) -> String {
        format!("wae:stream:{}", queue)
    }

    fn group_name() -> &'static str {
        "wae-consumer-group"
    }
}

#[async_trait::async_trait]
impl QueueManager for RedisQueueManager {
    async fn declare_queue(&self, config: &QueueConfig) -> WaeResult<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = Self::stream_name(&config.name);
        let group_name = Self::group_name();

        let _: RedisResult<()> = conn.xgroup_create_mkstream(&stream_name, group_name, "0").await;

        let mut configs = self.configs.lock().await;
        configs.insert(config.name.clone(), config.clone());

        Ok(())
    }

    async fn delete_queue(&self, name: &str) -> WaeResult<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = Self::stream_name(name);

        let _: RedisResult<()> = conn.del(&stream_name).await;

        let mut configs = self.configs.lock().await;
        configs.remove(name);

        Ok(())
    }

    async fn queue_exists(&self, name: &str) -> WaeResult<bool> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = Self::stream_name(name);
        let exists: bool = conn
            .exists(&stream_name)
            .await
            .map_err(|e| WaeError::internal(format!("Failed to check if stream exists: {}", e)))?;

        Ok(exists)
    }

    async fn queue_message_count(&self, name: &str) -> WaeResult<u64> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = Self::stream_name(name);
        let len: u64 =
            conn.xlen(&stream_name).await.map_err(|e| WaeError::internal(format!("Failed to get stream length: {}", e)))?;

        Ok(len)
    }

    async fn purge_queue(&self, name: &str) -> WaeResult<u64> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = Self::stream_name(name);
        let len: u64 =
            conn.xlen(&stream_name).await.map_err(|e| WaeError::internal(format!("Failed to get stream length: {}", e)))?;

        let _: RedisResult<()> = conn.del(&stream_name).await;

        Ok(len)
    }
}

/// Redis 生产者后端
pub struct RedisProducerBackend {
    config: ProducerConfig,
    client: Client,
    manager: Arc<RedisQueueManager>,
}

impl RedisProducerBackend {
    /// 创建新的 Redis 生产者后端
    pub fn new(config: ProducerConfig, manager: Arc<RedisQueueManager>) -> Self {
        Self { config, client: manager.client.clone(), manager }
    }

    fn encode_metadata(metadata: &MessageMetadata) -> HashMap<String, String> {
        let mut fields = HashMap::new();

        if let Some(id) = &metadata.id {
            fields.insert("id".to_string(), id.clone());
        }
        if let Some(correlation_id) = &metadata.correlation_id {
            fields.insert("correlation_id".to_string(), correlation_id.clone());
        }
        if let Some(reply_to) = &metadata.reply_to {
            fields.insert("reply_to".to_string(), reply_to.clone());
        }
        if let Some(content_type) = &metadata.content_type {
            fields.insert("content_type".to_string(), content_type.clone());
        }
        if let Some(timestamp) = metadata.timestamp {
            fields.insert("timestamp".to_string(), timestamp.to_string());
        }
        if let Some(priority) = metadata.priority {
            fields.insert("priority".to_string(), priority.to_string());
        }
        if let Some(expiration) = metadata.expiration {
            fields.insert("expiration".to_string(), expiration.to_string());
        }

        for (key, value) in &metadata.headers {
            fields.insert(format!("header:{}", key), value.clone());
        }

        fields
    }
}

#[async_trait::async_trait]
impl ProducerBackend for RedisProducerBackend {
    async fn send_raw(&self, queue: &str, message: &RawMessage) -> WaeResult<MessageId> {
        self.manager.declare_queue(&QueueConfig::new(queue)).await?;

        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = RedisQueueManager::stream_name(queue);
        let data_b64 = general_purpose::STANDARD.encode(&message.data);

        let mut fields_vec = Vec::new();
        let mut fields = Self::encode_metadata(&message.metadata);
        fields.insert("data".to_string(), data_b64);
        for (k, v) in fields {
            fields_vec.push((k, v));
        }

        let id: String = conn
            .xadd(&stream_name, "*", &fields_vec)
            .await
            .map_err(|e| WaeError::internal(format!("Failed to add message to stream: {}", e)))?;

        Ok(id)
    }

    async fn send_raw_default(&self, message: &RawMessage) -> WaeResult<MessageId> {
        let queue = self.config.default_queue.as_ref().ok_or_else(|| WaeError::config_missing("default_queue"))?;
        self.send_raw(queue, message).await
    }

    async fn send_raw_delayed(&self, queue: &str, message: &RawMessage, delay: Duration) -> WaeResult<MessageId> {
        let delayed_queue = format!("{}:delayed", queue);
        let delayed_stream = RedisQueueManager::stream_name(&delayed_queue);

        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let data_b64 = general_purpose::STANDARD.encode(&message.data);
        let mut fields = Self::encode_metadata(&message.metadata);
        fields.insert("data".to_string(), data_b64);
        fields.insert("target_queue".to_string(), queue.to_string());

        let mut fields_vec = Vec::new();
        for (k, v) in fields {
            fields_vec.push((k, v));
        }

        let id: String = conn
            .xadd(&delayed_stream, "*", &fields_vec)
            .await
            .map_err(|e| WaeError::internal(format!("Failed to add delayed message: {}", e)))?;

        let message_clone = message.clone();
        tokio::spawn({
            let client = self.client.clone();
            let manager = self.manager.clone();
            let queue = queue.to_string();
            let delayed_stream = delayed_stream.clone();
            let id = id.clone();
            let delay = delay;

            async move {
                tokio::time::sleep(delay).await;

                if let Ok(mut conn) = client.get_async_connection().await {
                    let _: RedisResult<()> = conn.xdel(&delayed_stream, &[&id]).await;

                    let mut producer = RedisProducerBackend::new(ProducerConfig::default(), manager.clone());
                    let _ = producer.send_raw(&queue, &message_clone).await;
                }
            }
        });

        Ok(id)
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

/// Redis 消费者后端
pub struct RedisConsumerBackend {
    config: ConsumerConfig,
    client: Client,
    manager: Arc<RedisQueueManager>,
    consumer_name: String,
    delivery_tags: Arc<Mutex<HashMap<u64, String>>>,
    next_delivery_tag: Arc<Mutex<u64>>,
}

impl RedisConsumerBackend {
    /// 创建新的 Redis 消费者后端
    pub fn new(config: ConsumerConfig, manager: Arc<RedisQueueManager>) -> Self {
        let consumer_name = format!("wae-consumer-{}", uuid::Uuid::new_v4());
        Self {
            config,
            client: manager.client.clone(),
            manager,
            consumer_name,
            delivery_tags: Arc::new(Mutex::new(HashMap::new())),
            next_delivery_tag: Arc::new(Mutex::new(1)),
        }
    }

    fn decode_metadata(fields: &HashMap<String, String>) -> MessageMetadata {
        let mut metadata = MessageMetadata::default();

        if let Some(id) = fields.get("id") {
            metadata.id = Some(id.clone());
        }
        if let Some(correlation_id) = fields.get("correlation_id") {
            metadata.correlation_id = Some(correlation_id.clone());
        }
        if let Some(reply_to) = fields.get("reply_to") {
            metadata.reply_to = Some(reply_to.clone());
        }
        if let Some(content_type) = fields.get("content_type") {
            metadata.content_type = Some(content_type.clone());
        }
        if let Some(timestamp) = fields.get("timestamp").and_then(|s| s.parse().ok()) {
            metadata.timestamp = Some(timestamp);
        }
        if let Some(priority) = fields.get("priority").and_then(|s| s.parse().ok()) {
            metadata.priority = Some(priority);
        }
        if let Some(expiration) = fields.get("expiration").and_then(|s| s.parse().ok()) {
            metadata.expiration = Some(expiration);
        }

        for (key, value) in fields {
            if let Some(header_key) = key.strip_prefix("header:") {
                metadata.headers.insert(header_key.to_string(), value.clone());
            }
        }

        metadata
    }
}

#[async_trait::async_trait]
impl ConsumerBackend for RedisConsumerBackend {
    async fn receive_raw(&self) -> WaeResult<Option<ReceivedRawMessage>> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = RedisQueueManager::stream_name(&self.config.queue);
        let group_name = RedisQueueManager::group_name();

        let opts = StreamReadOptions::default().group(group_name, &self.consumer_name).count(1).block(1000);

        let result: RedisResult<redis::streams::StreamReadReply> = conn.xread_options(&[&stream_name], &[">"], &opts).await;

        match result {
            Ok(streams) => {
                if let Some(stream) = streams.keys.into_iter().next() {
                    if let Some(entry) = stream.ids.into_iter().next() {
                        let mut fields = HashMap::new();
                        for (key, value) in entry.map {
                            if let Ok(s) = String::from_redis_value(&value) {
                                fields.insert(key, s);
                            }
                        }

                        let data_b64 = fields.get("data").ok_or_else(|| WaeError::internal("Missing data field"))?;
                        let data = general_purpose::STANDARD
                            .decode(data_b64)
                            .map_err(|e| WaeError::internal(format!("Failed to decode data: {}", e)))?;

                        let metadata = Self::decode_metadata(&fields);

                        let mut next_tag = self.next_delivery_tag.lock().await;
                        let delivery_tag = *next_tag;
                        *next_tag += 1;
                        drop(next_tag);

                        let mut delivery_tags = self.delivery_tags.lock().await;
                        delivery_tags.insert(delivery_tag, entry.id.clone());

                        let redelivery_count =
                            metadata.headers.get("x-redelivery-count").and_then(|s| s.parse().ok()).unwrap_or(0);

                        let message = RawMessage { data, metadata };
                        return Ok(Some(ReceivedRawMessage { message, delivery_tag, redelivery_count }));
                    }
                }
                Ok(None)
            }
            Err(e) => {
                if e.to_string().contains("NOGROUP") {
                    self.manager.declare_queue(&QueueConfig::new(&self.config.queue)).await?;
                    Ok(None)
                }
                else {
                    Err(WaeError::internal(format!("Failed to read from stream: {}", e)))
                }
            }
        }
    }

    async fn ack(&self, delivery_tag: u64) -> WaeResult<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = RedisQueueManager::stream_name(&self.config.queue);
        let group_name = RedisQueueManager::group_name();

        let mut delivery_tags = self.delivery_tags.lock().await;
        if let Some(message_id) = delivery_tags.remove(&delivery_tag) {
            let _: RedisResult<()> = conn.xack(&stream_name, group_name, &[&message_id]).await;
            let _: RedisResult<()> = conn.xdel(&stream_name, &[&message_id]).await;
        }

        Ok(())
    }

    async fn nack(&self, delivery_tag: u64, requeue: bool) -> WaeResult<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| WaeError::internal(format!("Failed to get Redis connection: {}", e)))?;

        let stream_name = RedisQueueManager::stream_name(&self.config.queue);
        let group_name = RedisQueueManager::group_name();

        let mut delivery_tags = self.delivery_tags.lock().await;
        if let Some(message_id) = delivery_tags.remove(&delivery_tag) {
            if requeue {
                let _: RedisResult<()> = conn.xclaim(&stream_name, group_name, &self.consumer_name, 0, &[&message_id]).await;
            }
            else {
                let configs = self.manager.configs.lock().await;
                let dlq_name = configs.get(&self.config.queue).and_then(|c| c.dead_letter_queue.clone());
                drop(configs);

                if let Some(dlq_name) = dlq_name {
                    let _: RedisResult<()> = conn.xack(&stream_name, group_name, &[&message_id]).await;
                    let _: RedisResult<()> = conn.xdel(&stream_name, &[&message_id]).await;
                }
                else {
                    let _: RedisResult<()> = conn.xack(&stream_name, group_name, &[&message_id]).await;
                    let _: RedisResult<()> = conn.xdel(&stream_name, &[&message_id]).await;
                }
            }
        }

        Ok(())
    }

    fn config(&self) -> &ConsumerConfig {
        &self.config
    }
}

/// Redis 队列服务
pub struct RedisQueueService {
    manager: Arc<RedisQueueManager>,
}

impl RedisQueueService {
    /// 创建新的 Redis 队列服务
    pub fn new(manager: Arc<RedisQueueManager>) -> Self {
        Self { manager }
    }

    /// 从 URL 创建 Redis 队列服务
    pub async fn from_url(url: &str) -> WaeResult<Self> {
        let manager = Arc::new(RedisQueueManager::from_url(url).await?);
        Ok(Self::new(manager))
    }
}

impl QueueService for RedisQueueService {
    async fn create_producer(&self, config: ProducerConfig) -> WaeResult<MessageProducer> {
        Ok(MessageProducer::new(Box::new(RedisProducerBackend::new(config, self.manager.clone()))))
    }

    async fn create_consumer(&self, config: ConsumerConfig) -> WaeResult<MessageConsumer> {
        self.manager.declare_queue(&QueueConfig::new(&config.queue)).await?;
        Ok(MessageConsumer::new(Box::new(RedisConsumerBackend::new(config, self.manager.clone()))))
    }

    fn manager(&self) -> &dyn QueueManager {
        self.manager.as_ref() as &dyn QueueManager
    }

    async fn close(&self) -> WaeResult<()> {
        Ok(())
    }
}
