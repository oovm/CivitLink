//! Kafka 队列实现

use super::super::*;
use base64::{Engine as _, engine::general_purpose};
use rdkafka::{
    ClientConfig,
    consumer::{Consumer, DefaultConsumerContext, StreamConsumer},
    message::{Headers, OwnedMessage},
    producer::{DeliveryFuture, FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Kafka 连接配置
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    /// Kafka Broker 地址 (例如: localhost:9092)
    pub brokers: String,
    /// 客户端 ID
    pub client_id: Option<String>,
    /// 生产者配置
    pub producer_config: HashMap<String, String>,
    /// 消费者配置
    pub consumer_config: HashMap<String, String>,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: "localhost:9092".to_string(),
            client_id: Some("wae-kafka".to_string()),
            producer_config: HashMap::new(),
            consumer_config: HashMap::new(),
        }
    }
}

impl KafkaConfig {
    /// 创建新的 Kafka 配置
    pub fn new(brokers: impl Into<String>) -> Self {
        Self { brokers: brokers.into(), ..Default::default() }
    }
}

/// Kafka 队列管理器
pub struct KafkaQueueManager {
    config: ClientConfig,
}

impl KafkaQueueManager {
    /// 创建新的 Kafka 队列管理器
    pub fn new(config: &KafkaConfig) -> Self {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", &config.brokers);
        if let Some(client_id) = &config.client_id {
            client_config.set("client.id", client_id);
        }
        for (key, value) in &config.producer_config {
            client_config.set(key, value);
        }
        for (key, value) in &config.consumer_config {
            client_config.set(key, value);
        }
        Self { config: client_config }
    }

    /// 内部主题名称
    fn topic_name(queue: &str) -> String {
        queue.to_string()
    }
}

#[async_trait::async_trait]
impl QueueManager for KafkaQueueManager {
    async fn declare_queue(&self, config: &QueueConfig) -> WaeResult<()> {
        Ok(())
    }

    async fn delete_queue(&self, name: &str) -> WaeResult<()> {
        Ok(())
    }

    async fn queue_exists(&self, name: &str) -> WaeResult<bool> {
        Ok(true)
    }

    async fn queue_message_count(&self, name: &str) -> WaeResult<u64> {
        Ok(0)
    }

    async fn purge_queue(&self, name: &str) -> WaeResult<u64> {
        Ok(0)
    }
}

/// Kafka 生产者后端
pub struct KafkaProducerBackend {
    config: ProducerConfig,
    producer: Arc<FutureProducer>,
    manager: Arc<KafkaQueueManager>,
}

impl KafkaProducerBackend {
    /// 创建新的 Kafka 生产者后端
    pub fn new(config: ProducerConfig, manager: Arc<KafkaQueueManager>, producer: Arc<FutureProducer>) -> Self {
        Self { config, producer, manager }
    }

    /// 编码元数据
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
impl ProducerBackend for KafkaProducerBackend {
    async fn send_raw(&self, queue: &str, message: &RawMessage) -> WaeResult<MessageId> {
        let id = Uuid::new_v4().to_string();
        let mut metadata = message.metadata.clone();
        metadata.id = Some(id.clone());
        if metadata.timestamp.is_none() {
            metadata.timestamp =
                Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64);
        }

        let topic = KafkaQueueManager::topic_name(queue);
        let data_b64 = general_purpose::STANDARD.encode(&message.data);
        let mut fields = Self::encode_metadata(&metadata);
        fields.insert("data".to_string(), data_b64);

        let payload =
            serde_json::to_vec(&fields).map_err(|e| WaeError::internal(format!("Failed to serialize Kafka message: {}", e)))?;

        let record = FutureRecord::to(&topic).payload(&payload).key(&id);

        let result = self.producer.send(record, Timeout::After(Duration::from_secs(5))).await;

        match result {
            Ok((_, _)) => Ok(id),
            Err((e, _)) => Err(WaeError::internal(format!("Failed to send Kafka message: {}", e))),
        }
    }

    async fn send_raw_default(&self, message: &RawMessage) -> WaeResult<MessageId> {
        let queue = self.config.default_queue.as_ref().ok_or_else(|| WaeError::config_missing("default_queue"))?;
        self.send_raw(queue, message).await
    }

    async fn send_raw_delayed(&self, queue: &str, message: &RawMessage, delay: Duration) -> WaeResult<MessageId> {
        let id = Uuid::new_v4().to_string();
        let message_clone = message.clone();
        let producer = self.producer.clone();
        let manager = self.manager.clone();
        let queue = queue.to_string();

        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let mut producer_config = ProducerConfig::default();
            producer_config.default_queue = Some(queue.clone());
            let mut producer = KafkaProducerBackend::new(producer_config, manager.clone(), producer.clone());
            let _ = producer.send_raw(&queue, &message_clone).await;
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

/// Kafka 消费者后端
pub struct KafkaConsumerBackend {
    config: ConsumerConfig,
    consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    manager: Arc<KafkaQueueManager>,
    delivery_tags: Arc<Mutex<HashMap<u64, (i32, i64)>>>,
    next_delivery_tag: Arc<Mutex<u64>>,
}

impl KafkaConsumerBackend {
    /// 创建新的 Kafka 消费者后端
    pub fn new(
        config: ConsumerConfig,
        manager: Arc<KafkaQueueManager>,
        consumer: Arc<StreamConsumer<DefaultConsumerContext>>,
    ) -> Self {
        Self {
            config,
            consumer,
            manager,
            delivery_tags: Arc::new(Mutex::new(HashMap::new())),
            next_delivery_tag: Arc::new(Mutex::new(1)),
        }
    }

    /// 解码元数据
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
impl ConsumerBackend for KafkaConsumerBackend {
    async fn receive_raw(&self) -> WaeResult<Option<ReceivedRawMessage>> {
        Ok(None)
    }

    async fn ack(&self, delivery_tag: u64) -> WaeResult<()> {
        Ok(())
    }

    async fn nack(&self, delivery_tag: u64, requeue: bool) -> WaeResult<()> {
        Ok(())
    }

    fn config(&self) -> &ConsumerConfig {
        &self.config
    }
}

/// Kafka 队列服务
pub struct KafkaQueueService {
    manager: Arc<KafkaQueueManager>,
}

impl KafkaQueueService {
    /// 创建新的 Kafka 队列服务
    pub fn new(manager: Arc<KafkaQueueManager>) -> Self {
        Self { manager }
    }

    /// 从配置创建 Kafka 队列服务
    pub fn from_config(config: &KafkaConfig) -> Self {
        let manager = Arc::new(KafkaQueueManager::new(config));
        Self::new(manager)
    }
}

impl QueueService for KafkaQueueService {
    async fn create_producer(&self, config: ProducerConfig) -> WaeResult<MessageProducer> {
        let producer: FutureProducer =
            self.manager.config.create().map_err(|e| WaeError::internal(format!("Failed to create Kafka producer: {}", e)))?;
        Ok(MessageProducer::new(Box::new(KafkaProducerBackend::new(config, self.manager.clone(), Arc::new(producer)))))
    }

    async fn create_consumer(&self, config: ConsumerConfig) -> WaeResult<MessageConsumer> {
        let mut consumer_config = self.manager.config.clone();
        consumer_config.set("group.id", format!("wae-consumer-group-{}", uuid::Uuid::new_v4()));
        consumer_config.set("auto.offset.reset", "earliest");

        let consumer: StreamConsumer =
            consumer_config.create().map_err(|e| WaeError::internal(format!("Failed to create Kafka consumer: {}", e)))?;

        consumer.subscribe(&[&config.queue]).map_err(|e| WaeError::internal(format!("Failed to subscribe to topic: {}", e)))?;

        Ok(MessageConsumer::new(Box::new(KafkaConsumerBackend::new(config, self.manager.clone(), Arc::new(consumer)))))
    }

    fn manager(&self) -> &dyn QueueManager {
        self.manager.as_ref() as &dyn QueueManager
    }

    async fn close(&self) -> WaeResult<()> {
        Ok(())
    }
}
