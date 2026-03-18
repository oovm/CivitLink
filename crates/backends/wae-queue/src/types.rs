//! 消息队列类型定义

use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;
use wae_types::{WaeError, WaeResult};

/// 消息 ID 类型
pub type MessageId = String;

/// 队列名称类型
pub type QueueName = String;

/// 消息元数据
#[derive(Debug, Clone, Default)]
pub struct MessageMetadata {
    /// 消息 ID
    pub id: Option<MessageId>,
    /// 关联 ID (用于消息关联)
    pub correlation_id: Option<String>,
    /// 回复队列
    pub reply_to: Option<QueueName>,
    /// 内容类型
    pub content_type: Option<String>,
    /// 时间戳
    pub timestamp: Option<u64>,
    /// 优先级 (0-9)
    pub priority: Option<u8>,
    /// 过期时间 (毫秒)
    pub expiration: Option<u64>,
    /// 自定义头信息
    pub headers: std::collections::HashMap<String, String>,
}

/// 原始消息 (字节形式)
#[derive(Debug, Clone)]
pub struct RawMessage {
    /// 消息数据
    pub data: Vec<u8>,
    /// 消息元数据
    pub metadata: MessageMetadata,
}

impl RawMessage {
    /// 创建新消息
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, metadata: MessageMetadata::default() }
    }

    /// 设置关联 ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }

    /// 设置回复队列
    pub fn with_reply_to(mut self, queue: impl Into<String>) -> Self {
        self.metadata.reply_to = Some(queue.into());
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.metadata.priority = Some(priority.min(9));
        self
    }

    /// 设置过期时间
    pub fn with_expiration(mut self, ms: u64) -> Self {
        self.metadata.expiration = Some(ms);
        self
    }

    /// 添加自定义头
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.headers.insert(key.into(), value.into());
        self
    }
}

/// 消息封装 (泛型)
#[derive(Debug, Clone)]
pub struct Message<T> {
    /// 消息体
    pub payload: T,
    /// 消息元数据
    pub metadata: MessageMetadata,
}

impl<T> Message<T> {
    /// 创建新消息
    pub fn new(payload: T) -> Self {
        Self { payload, metadata: MessageMetadata::default() }
    }

    /// 设置关联 ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }

    /// 设置回复队列
    pub fn with_reply_to(mut self, queue: impl Into<String>) -> Self {
        self.metadata.reply_to = Some(queue.into());
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.metadata.priority = Some(priority.min(9));
        self
    }

    /// 设置过期时间
    pub fn with_expiration(mut self, ms: u64) -> Self {
        self.metadata.expiration = Some(ms);
        self
    }

    /// 添加自定义头
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.headers.insert(key.into(), value.into());
        self
    }

    /// 序列化为原始消息
    pub fn into_raw(self) -> WaeResult<RawMessage>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec(&self.payload).map_err(|_e| WaeError::serialization_failed("Message"))?;
        Ok(RawMessage { data, metadata: self.metadata })
    }

    /// 序列化为原始消息 (引用版本)
    pub fn to_raw(&self) -> WaeResult<RawMessage>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec(&self.payload).map_err(|_e| WaeError::serialization_failed("Message"))?;
        Ok(RawMessage { data, metadata: self.metadata.clone() })
    }
}

impl RawMessage {
    /// 反序列化为泛型消息
    pub fn into_typed<T: DeserializeOwned>(self) -> WaeResult<Message<T>> {
        let payload = serde_json::from_slice(&self.data).map_err(|_e| WaeError::deserialization_failed("Message"))?;
        Ok(Message { payload, metadata: self.metadata })
    }
}

/// 接收到的原始消息 (带确认能力)
#[derive(Debug)]
pub struct ReceivedRawMessage {
    /// 消息内容
    pub message: RawMessage,
    /// 消息 ID (用于确认)
    pub delivery_tag: u64,
    /// 重投递次数
    pub redelivery_count: u32,
}

/// 接收到的消息 (泛型)
#[derive(Debug)]
pub struct ReceivedMessage<T> {
    /// 消息内容
    pub message: Message<T>,
    /// 消息 ID (用于确认)
    pub delivery_tag: u64,
    /// 重投递次数
    pub redelivery_count: u32,
}

/// 队列配置
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// 队列名称
    pub name: QueueName,
    /// 是否持久化
    pub durable: bool,
    /// 是否自动删除 (当没有消费者时)
    pub auto_delete: bool,
    /// 最大消息数
    pub max_messages: Option<u64>,
    /// 最大消息大小 (字节)
    pub max_message_size: Option<u64>,
    /// 消息存活时间 (毫秒)
    pub message_ttl: Option<u64>,
    /// 死信队列
    pub dead_letter_queue: Option<QueueName>,
}

impl QueueConfig {
    /// 创建新的队列配置
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durable: true,
            auto_delete: false,
            max_messages: None,
            max_message_size: None,
            message_ttl: None,
            dead_letter_queue: None,
        }
    }

    /// 设置持久化
    pub fn durable(mut self, durable: bool) -> Self {
        self.durable = durable;
        self
    }

    /// 设置自动删除
    pub fn auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// 设置最大消息数
    pub fn max_messages(mut self, max: u64) -> Self {
        self.max_messages = Some(max);
        self
    }

    /// 设置消息 TTL
    pub fn message_ttl(mut self, ttl_ms: u64) -> Self {
        self.message_ttl = Some(ttl_ms);
        self
    }

    /// 设置死信队列
    pub fn dead_letter_queue(mut self, queue: impl Into<String>) -> Self {
        self.dead_letter_queue = Some(queue.into());
        self
    }
}

/// 生产者配置
#[derive(Debug, Clone)]
pub struct ProducerConfig {
    /// 默认队列
    pub default_queue: Option<QueueName>,
    /// 消息确认超时
    pub confirm_timeout: Duration,
    /// 重试次数
    pub retry_count: u32,
    /// 重试间隔
    pub retry_interval: Duration,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            default_queue: None,
            confirm_timeout: Duration::from_secs(5),
            retry_count: 3,
            retry_interval: Duration::from_millis(100),
        }
    }
}

/// 消费者配置
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    /// 队列名称
    pub queue: QueueName,
    /// 消费者标签
    pub consumer_tag: Option<String>,
    /// 是否自动确认
    pub auto_ack: bool,
    /// 预取数量
    pub prefetch_count: u16,
    /// 是否独占
    pub exclusive: bool,
}

impl ConsumerConfig {
    /// 创建新的消费者配置
    pub fn new(queue: impl Into<String>) -> Self {
        Self { queue: queue.into(), consumer_tag: None, auto_ack: false, prefetch_count: 10, exclusive: false }
    }

    /// 设置自动确认
    pub fn auto_ack(mut self, auto_ack: bool) -> Self {
        self.auto_ack = auto_ack;
        self
    }

    /// 设置预取数量
    pub fn prefetch(mut self, count: u16) -> Self {
        self.prefetch_count = count;
        self
    }
}
