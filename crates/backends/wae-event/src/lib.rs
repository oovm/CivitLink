//! WAE Event - 事件驱动模块
//!
//! 提供统一的事件驱动能力抽象，支持事件发布/订阅、事件存储和事件回放。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持事件溯源、CQRS 等模式。

#![warn(missing_docs)]

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};
use wae_types::{WaeError, WaeResult};

/// 事件 ID 类型
pub type EventId = String;

/// 订阅 ID 类型
pub type SubscriptionId = String;

/// 事件类型名称
pub type EventTypeName = String;

/// 事件基础 trait
///
/// 所有事件必须实现此 trait，定义事件的基本属性。
pub trait Event: Send + Sync + 'static {
    /// 获取事件类型名称
    fn event_type(&self) -> EventTypeName;

    /// 获取事件 ID
    fn event_id(&self) -> &EventId;

    /// 获取事件时间戳
    fn timestamp(&self) -> u64;
}

/// 事件数据封装
///
/// 用于序列化和传输事件的通用容器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// 事件 ID
    pub id: EventId,
    /// 事件类型名称
    pub event_type: EventTypeName,
    /// 事件时间戳 (Unix 毫秒)
    pub timestamp: u64,
    /// 事件载荷 (JSON 序列化)
    pub payload: serde_json::Value,
    /// 事件元数据
    pub metadata: HashMap<String, String>,
}

impl EventData {
    /// 创建新的事件数据
    pub fn new<E: Event + Serialize>(event: &E) -> WaeResult<Self> {
        let payload = serde_json::to_value(event).map_err(|_e| WaeError::serialization_failed("Event"))?;
        Ok(Self {
            id: event.event_id().clone(),
            event_type: event.event_type(),
            timestamp: event.timestamp(),
            payload,
            metadata: HashMap::new(),
        })
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 反序列化为具体事件类型
    pub fn into_event<E: DeserializeOwned>(self) -> WaeResult<E> {
        serde_json::from_value(self.payload).map_err(|_e| WaeError::deserialization_failed("Event"))
    }

    /// 获取事件 ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取事件类型
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// 获取时间戳
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

/// 事件处理器 trait (异步)
///
/// 定义异步事件处理器的接口。
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件
    async fn handle(&self, event: EventData) -> WaeResult<()>;

    /// 获取处理器感兴趣的事件类型
    fn event_types(&self) -> Vec<EventTypeName>;
}

/// 同步事件处理器 trait
///
/// 定义同步事件处理器的接口。
pub trait SyncEventHandler: Send + Sync {
    /// 处理事件
    fn handle(&self, event: EventData) -> WaeResult<()>;

    /// 获取处理器感兴趣的事件类型
    fn event_types(&self) -> Vec<EventTypeName>;
}

/// 异步事件处理器包装器
///
/// 将闭包包装为 EventHandler trait 对象。
pub struct AsyncEventHandler<F> {
    handler: F,
    event_types: Vec<EventTypeName>,
}

impl<F> AsyncEventHandler<F>
where
    F: Fn(EventData) -> std::pin::Pin<Box<dyn std::future::Future<Output = WaeResult<()>> + Send + Sync>>
        + Send
        + Sync
        + 'static,
{
    /// 创建新的异步事件处理器
    pub fn new(event_types: Vec<EventTypeName>, handler: F) -> Self {
        Self { handler, event_types }
    }
}

#[async_trait]
impl<F> EventHandler for AsyncEventHandler<F>
where
    F: Fn(EventData) -> std::pin::Pin<Box<dyn std::future::Future<Output = WaeResult<()>> + Send + Sync>>
        + Send
        + Sync
        + 'static,
{
    async fn handle(&self, event: EventData) -> WaeResult<()> {
        (self.handler)(event).await
    }

    fn event_types(&self) -> Vec<EventTypeName> {
        self.event_types.clone()
    }
}

/// 同步事件处理器包装器
///
/// 将闭包包装为 SyncEventHandler trait 对象。
pub struct SyncEventHandlerWrapper<F> {
    handler: F,
    event_types: Vec<EventTypeName>,
}

impl<F> SyncEventHandlerWrapper<F>
where
    F: Fn(EventData) -> WaeResult<()> + Send + Sync + 'static,
{
    /// 创建新的同步事件处理器
    pub fn new(event_types: Vec<EventTypeName>, handler: F) -> Self {
        Self { handler, event_types }
    }
}

impl<F> SyncEventHandler for SyncEventHandlerWrapper<F>
where
    F: Fn(EventData) -> WaeResult<()> + Send + Sync + 'static,
{
    fn handle(&self, event: EventData) -> WaeResult<()> {
        (self.handler)(event)
    }

    fn event_types(&self) -> Vec<EventTypeName> {
        self.event_types.clone()
    }
}

/// 事件过滤器 trait
///
/// 用于过滤事件，决定是否处理某个事件。
pub trait EventFilter: Send + Sync {
    /// 判断是否应该处理该事件
    fn should_handle(&self, event: &EventData) -> bool;
}

/// 类型事件过滤器
///
/// 只处理指定类型的事件。
pub struct TypeEventFilter {
    allowed_types: Vec<EventTypeName>,
}

impl TypeEventFilter {
    /// 创建新的类型过滤器
    pub fn new(allowed_types: Vec<EventTypeName>) -> Self {
        Self { allowed_types }
    }
}

impl EventFilter for TypeEventFilter {
    fn should_handle(&self, event: &EventData) -> bool {
        self.allowed_types.contains(&event.event_type)
    }
}

/// 元数据事件过滤器
///
/// 根据元数据过滤事件。
pub struct MetadataEventFilter {
    key: String,
    value: String,
}

impl MetadataEventFilter {
    /// 创建新的元数据过滤器
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self { key: key.into(), value: value.into() }
    }
}

impl EventFilter for MetadataEventFilter {
    fn should_handle(&self, event: &EventData) -> bool {
        event.metadata.get(&self.key).map(|v| v == &self.value).unwrap_or(false)
    }
}

/// 订阅句柄
///
/// 用于管理订阅生命周期。
#[derive(Debug, Clone)]
pub struct Subscription {
    /// 订阅 ID
    pub id: SubscriptionId,
    /// 订阅的事件类型
    pub event_types: Vec<EventTypeName>,
    /// 订阅时间
    pub created_at: u64,
    /// 是否激活
    pub active: bool,
}

impl Subscription {
    /// 创建新的订阅
    pub fn new(id: SubscriptionId, event_types: Vec<EventTypeName>) -> Self {
        Self {
            id,
            event_types,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            active: true,
        }
    }
}

/// 事件存储 trait
///
/// 定义事件持久化存储的接口。
#[async_trait]
pub trait EventStore: Send + Sync {
    /// 追加事件到存储
    async fn append(&self, event: &EventData) -> WaeResult<()>;

    /// 批量追加事件
    async fn append_batch(&self, events: &[EventData]) -> WaeResult<()>;

    /// 获取指定类型的事件流
    async fn get_events(&self, event_type: &str) -> WaeResult<Vec<EventData>>;

    /// 获取指定时间范围的事件
    async fn get_events_by_time(&self, start: u64, end: u64) -> WaeResult<Vec<EventData>>;

    /// 获取所有事件
    async fn get_all_events(&self) -> WaeResult<Vec<EventData>>;

    /// 获取事件数量
    async fn count(&self) -> WaeResult<u64>;

    /// 清空所有事件
    async fn clear(&self) -> WaeResult<()>;
}

/// 内存事件存储
///
/// 基于内存的事件存储实现，用于开发和测试。
pub struct InMemoryEventStore {
    events: RwLock<Vec<EventData>>,
}

impl InMemoryEventStore {
    /// 创建新的内存事件存储
    pub fn new() -> Self {
        Self { events: RwLock::new(Vec::new()) }
    }

    /// 创建带初始容量的内存事件存储
    pub fn with_capacity(capacity: usize) -> Self {
        Self { events: RwLock::new(Vec::with_capacity(capacity)) }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, event: &EventData) -> WaeResult<()> {
        let mut events = self.events.write();
        events.push(event.clone());
        debug!("Event appended: {} [{}]", event.id, event.event_type);
        Ok(())
    }

    async fn append_batch(&self, new_events: &[EventData]) -> WaeResult<()> {
        let mut events = self.events.write();
        events.extend(new_events.iter().cloned());
        debug!("Batch appended: {} events", new_events.len());
        Ok(())
    }

    async fn get_events(&self, event_type: &str) -> WaeResult<Vec<EventData>> {
        let events = self.events.read();
        let filtered: Vec<EventData> = events.iter().filter(|e| e.event_type == event_type).cloned().collect();
        Ok(filtered)
    }

    async fn get_events_by_time(&self, start: u64, end: u64) -> WaeResult<Vec<EventData>> {
        let events = self.events.read();
        let filtered: Vec<EventData> = events.iter().filter(|e| e.timestamp >= start && e.timestamp <= end).cloned().collect();
        Ok(filtered)
    }

    async fn get_all_events(&self) -> WaeResult<Vec<EventData>> {
        let events = self.events.read();
        Ok(events.clone())
    }

    async fn count(&self) -> WaeResult<u64> {
        let events = self.events.read();
        Ok(events.len() as u64)
    }

    async fn clear(&self) -> WaeResult<()> {
        let mut events = self.events.write();
        events.clear();
        info!("Event store cleared");
        Ok(())
    }
}

/// 事件总线配置
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// 事件队列容量
    pub queue_capacity: usize,
    /// 处理器超时时间
    pub handler_timeout: Duration,
    /// 是否启用事件存储
    pub enable_store: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔
    pub retry_interval: Duration,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 1000,
            handler_timeout: Duration::from_secs(30),
            enable_store: false,
            max_retries: 3,
            retry_interval: Duration::from_millis(100),
        }
    }
}

/// 内部订阅信息
struct SubscriptionInfo {
    subscription: Subscription,
    handler: Arc<dyn EventHandler>,
    filter: Option<Arc<dyn EventFilter>>,
}

/// 事件总线
///
/// 事件驱动的核心组件，负责事件的发布和分发。
pub struct EventBus {
    config: EventBusConfig,
    store: Option<Arc<dyn EventStore>>,
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, SubscriptionInfo>>>,
    event_sender: mpsc::Sender<EventData>,
    shutdown_sender: RwLock<Option<mpsc::Sender<()>>>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(config: EventBusConfig) -> Self {
        Self::with_store(config, None)
    }

    /// 创建带事件存储的事件总线
    pub fn with_store(config: EventBusConfig, store: Option<Arc<dyn EventStore>>) -> Self {
        let (event_sender, event_receiver) = mpsc::channel::<EventData>(config.queue_capacity);
        let (shutdown_sender, shutdown_receiver) = mpsc::channel::<()>(1);

        let subscriptions = Arc::new(RwLock::new(HashMap::new()));

        let bus = Self {
            config,
            store,
            subscriptions: subscriptions.clone(),
            event_sender,
            shutdown_sender: RwLock::new(Some(shutdown_sender)),
        };

        let store_clone = bus.store.clone();
        let config_clone = bus.config.clone();

        tokio::spawn(Self::dispatcher(event_receiver, shutdown_receiver, subscriptions, store_clone, config_clone));

        bus
    }

    async fn dispatcher(
        mut receiver: mpsc::Receiver<EventData>,
        mut shutdown: mpsc::Receiver<()>,
        subscriptions: Arc<RwLock<HashMap<SubscriptionId, SubscriptionInfo>>>,
        store: Option<Arc<dyn EventStore>>,
        config: EventBusConfig,
    ) {
        loop {
            tokio::select! {
                Some(event) = receiver.recv() => {
                    if config.enable_store
                        && let Some(ref event_store) = store
                        && let Err(e) = event_store.append(&event).await
                    {
                        error!("Failed to store event: {}", e);
                    }

                    let subs = subscriptions.read();
                    for (_, info) in subs.iter() {
                        if !info.subscription.active {
                            continue;
                        }

                        if !info.subscription.event_types.contains(&event.event_type) {
                            continue;
                        }

                        if let Some(ref filter) = info.filter
                            && !filter.should_handle(&event)
                        {
                            continue;
                        }

                        let event_clone = event.clone();
                        let handler_clone = info.handler.clone();
                        let event_type = event.event_type.clone();
                        let sub_id = info.subscription.id.clone();
                        let timeout = config.handler_timeout;
                        let max_retries = config.max_retries;
                        let retry_interval = config.retry_interval;

                        tokio::spawn(async move {
                            for attempt in 0..max_retries {
                                match tokio::time::timeout(
                                    timeout,
                                    handler_clone.handle(event_clone.clone()),
                                )
                                .await
                                {
                                    Ok(Ok(())) => {
                                        debug!(
                                            "Event handled successfully: {} [subscription: {}]",
                                            event_type, sub_id
                                        );
                                        return;
                                    }
                                    Ok(Err(e)) => {
                                        warn!(
                                            "Event handler failed (attempt {}/{}): {}",
                                            attempt + 1,
                                            max_retries,
                                            e
                                        );
                                    }
                                    Err(_) => {
                                        warn!(
                                            "Event handler timeout (attempt {}/{})",
                                            attempt + 1,
                                            max_retries
                                        );
                                    }
                                }

                                if attempt + 1 < max_retries {
                                    tokio::time::sleep(retry_interval).await;
                                }
                            }
                            error!(
                                "Event handler failed after {} retries: {} [subscription: {}]",
                                max_retries, event_type, sub_id
                            );
                        });
                    }
                }
                _ = shutdown.recv() => {
                    info!("Event bus dispatcher shutting down");
                    break;
                }
                else => break,
            }
        }
    }

    /// 订阅事件
    ///
    /// 注册一个事件处理器，订阅指定类型的事件。
    #[instrument(skip(self, handler))]
    pub fn subscribe<H: EventHandler + 'static>(&self, event_types: Vec<EventTypeName>, handler: H) -> WaeResult<Subscription> {
        self.subscribe_with_filter(event_types, handler, None::<TypeEventFilter>)
    }

    /// 订阅事件 (带过滤器)
    ///
    /// 注册一个事件处理器，订阅指定类型的事件，并应用过滤器。
    #[instrument(skip(self, handler, filter))]
    pub fn subscribe_with_filter<H: EventHandler + 'static, F: EventFilter + 'static>(
        &self,
        event_types: Vec<EventTypeName>,
        handler: H,
        filter: Option<F>,
    ) -> WaeResult<Subscription> {
        let subscription_id = uuid::Uuid::new_v4().to_string();
        let subscription = Subscription::new(subscription_id.clone(), event_types.clone());

        let info = SubscriptionInfo {
            subscription: subscription.clone(),
            handler: Arc::new(handler),
            filter: filter.map(|f| Arc::new(f) as Arc<dyn EventFilter>),
        };

        {
            let mut subs = self.subscriptions.write();
            subs.insert(subscription_id, info);
        }

        info!("Subscription created: {} for types: {:?}", subscription.id, event_types);
        Ok(subscription)
    }

    /// 取消订阅
    ///
    /// 移除指定 ID 的订阅。
    #[instrument(skip(self))]
    pub fn unsubscribe(&self, subscription_id: &str) -> WaeResult<bool> {
        let mut subs = self.subscriptions.write();
        if subs.remove(subscription_id).is_some() {
            info!("Subscription removed: {}", subscription_id);
            Ok(true)
        }
        else {
            warn!("Subscription not found: {}", subscription_id);
            Ok(false)
        }
    }

    /// 发布事件
    ///
    /// 同步发布事件到事件总线。
    #[instrument(skip(self, event))]
    pub fn publish<E: Event + Serialize>(&self, event: &E) -> WaeResult<()> {
        let event_data = EventData::new(event)?;
        self.publish_data(event_data)
    }

    /// 发布事件数据
    ///
    /// 发布已封装的事件数据。
    #[instrument(skip(self, event_data))]
    pub fn publish_data(&self, event_data: EventData) -> WaeResult<()> {
        match self.event_sender.blocking_send(event_data.clone()) {
            Ok(()) => {
                debug!("Event published: {} [{}]", event_data.id, event_data.event_type);
                Ok(())
            }
            Err(_) => {
                error!("Event bus channel closed");
                Err(WaeError::internal("Event bus channel closed"))
            }
        }
    }

    /// 异步发布事件
    ///
    /// 异步发布事件到事件总线。
    #[instrument(skip(self, event))]
    pub async fn publish_async<E: Event + Serialize>(&self, event: &E) -> WaeResult<()> {
        let event_data = EventData::new(event)?;
        self.publish_data_async(event_data).await
    }

    /// 异步发布事件数据
    ///
    /// 异步发布已封装的事件数据。
    #[instrument(skip(self, event_data))]
    pub async fn publish_data_async(&self, event_data: EventData) -> WaeResult<()> {
        match self.event_sender.send(event_data.clone()).await {
            Ok(()) => {
                debug!("Event published async: {} [{}]", event_data.id, event_data.event_type);
                Ok(())
            }
            Err(_) => {
                error!("Event bus channel closed");
                Err(WaeError::internal("Event bus channel closed"))
            }
        }
    }

    /// 获取订阅数量
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.read().len()
    }

    /// 获取指定事件类型的订阅数量
    pub fn subscription_count_for_type(&self, event_type: &str) -> usize {
        let subs = self.subscriptions.read();
        subs.values().filter(|info| info.subscription.event_types.contains(&event_type.to_string())).count()
    }

    /// 关闭事件总线
    pub async fn shutdown(&self) -> WaeResult<()> {
        let tx = {
            let mut sender = self.shutdown_sender.write();
            sender.take()
        };
        if let Some(tx) = tx {
            let _ = tx.send(()).await;
            info!("Event bus shutdown initiated");
        }
        Ok(())
    }

    /// 事件回放
    ///
    /// 从事件存储中回放事件到指定处理器。
    pub async fn replay_events<H: EventHandler + 'static>(
        &self,
        store: &dyn EventStore,
        handler: &H,
        event_type: Option<&str>,
    ) -> WaeResult<u64> {
        let events = match event_type {
            Some(t) => store.get_events(t).await?,
            None => store.get_all_events().await?,
        };

        let mut count = 0u64;
        for event in events {
            if handler.event_types().contains(&event.event_type) {
                handler.handle(event).await?;
                count += 1;
            }
        }

        info!("Replayed {} events", count);
        Ok(count)
    }
}

/// 基础事件实现
///
/// 提供一个简单的事件基类，方便快速创建事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent<T> {
    /// 事件 ID
    pub id: EventId,
    /// 事件类型
    pub event_type: EventTypeName,
    /// 时间戳
    pub timestamp: u64,
    /// 事件载荷
    pub payload: T,
}

impl<T> BaseEvent<T>
where
    T: Send + Sync + 'static,
{
    /// 创建新的基础事件
    pub fn new(event_type: impl Into<String>, payload: T) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            payload,
        }
    }
}

impl<T: Serialize + Send + Sync + 'static> Event for BaseEvent<T> {
    fn event_type(&self) -> EventTypeName {
        self.event_type.clone()
    }

    fn event_id(&self) -> &EventId {
        &self.id
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

/// 便捷函数：创建内存事件存储
pub fn memory_event_store() -> Arc<InMemoryEventStore> {
    Arc::new(InMemoryEventStore::new())
}

/// 便捷函数：创建事件总线
pub fn event_bus(config: EventBusConfig) -> Arc<EventBus> {
    Arc::new(EventBus::new(config))
}

/// 便捷函数：创建带存储的事件总线
pub fn event_bus_with_store(config: EventBusConfig, store: Arc<dyn EventStore>) -> Arc<EventBus> {
    Arc::new(EventBus::with_store(config, Some(store)))
}
