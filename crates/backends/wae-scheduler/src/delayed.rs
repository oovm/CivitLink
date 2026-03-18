use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, info};
use uuid::Uuid;

use crate::task::{TaskHandle, TaskId, TaskState};
use wae_types::{WaeError, WaeResult};

/// 延迟任务
#[derive(Debug, Clone)]
pub struct DelayedTask<T: Send + Sync + Clone + 'static> {
    /// 任务 ID
    pub id: TaskId,
    /// 任务名称
    pub name: String,
    /// 执行时间
    pub execute_at: DateTime<Utc>,
    /// 优先级 (数值越小优先级越高)
    pub priority: u32,
    /// 任务数据
    pub data: T,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl<T: Send + Sync + Clone + 'static> DelayedTask<T> {
    /// 创建新的延迟任务
    pub fn new(name: String, execute_at: DateTime<Utc>, data: T) -> Self {
        Self { id: Uuid::new_v4().to_string(), name, execute_at, priority: 0, data, created_at: Utc::now() }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// 检查是否到期
    pub fn is_due(&self) -> bool {
        Utc::now() >= self.execute_at
    }

    /// 获取剩余时间
    pub fn remaining(&self) -> Duration {
        let now = Utc::now();
        if now >= self.execute_at { Duration::ZERO } else { (self.execute_at - now).to_std().unwrap_or(Duration::ZERO) }
    }
}

/// 延迟任务执行器 trait
#[async_trait]
pub trait DelayedTaskExecutor<T: Send + Sync + Clone + 'static>: Send + Sync {
    /// 执行任务
    async fn execute(&self, task: DelayedTask<T>) -> WaeResult<()>;
}

/// 延迟任务队列配置
#[derive(Debug, Clone)]
pub struct DelayedQueueConfig {
    /// 最大队列大小
    pub max_queue_size: usize,
    /// 轮询间隔
    pub poll_interval: Duration,
    /// 最大并发执行数
    pub max_concurrent_executions: usize,
}

impl Default for DelayedQueueConfig {
    fn default() -> Self {
        Self { max_queue_size: 10000, poll_interval: Duration::from_millis(100), max_concurrent_executions: 10 }
    }
}

/// 延迟任务队列
///
/// 管理延迟执行的任务，支持优先级排序。
pub struct DelayedQueue<T: Send + Sync + Clone + 'static> {
    /// 配置
    #[allow(dead_code)]
    config: DelayedQueueConfig,
    /// 任务队列 (按执行时间和优先级排序)
    queue: Arc<RwLock<Vec<DelayedTask<T>>>>,
    /// 任务句柄映射
    handles: Arc<RwLock<BTreeMap<TaskId, TaskHandle>>>,
    /// 任务发送器
    task_tx: mpsc::Sender<DelayedTask<T>>,
    /// 关闭信号
    shutdown_tx: broadcast::Sender<()>,
    /// 是否已关闭
    is_shutdown: Arc<AtomicBool>,
}

impl<T: Send + Sync + Clone + 'static> DelayedQueue<T> {
    /// 创建新的延迟任务队列
    pub fn new<E: DelayedTaskExecutor<T> + 'static>(config: DelayedQueueConfig, executor: Arc<E>) -> Self {
        let (task_tx, mut task_rx) = mpsc::channel(config.max_queue_size);
        let (shutdown_tx, _) = broadcast::channel(1);
        let queue: Arc<RwLock<Vec<DelayedTask<T>>>> = Arc::new(RwLock::new(Vec::new()));
        let handles: Arc<RwLock<BTreeMap<TaskId, TaskHandle>>> = Arc::new(RwLock::new(BTreeMap::new()));
        let is_shutdown = Arc::new(AtomicBool::new(false));

        let queue_clone = queue.clone();
        let handles_clone = handles.clone();
        let is_shutdown_clone = is_shutdown.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Delayed queue received shutdown signal");
                        break;
                    }
                    _ = tokio::time::sleep(config.poll_interval) => {
                        if is_shutdown_clone.load(Ordering::SeqCst) {
                            break;
                        }

                        let now = Utc::now();
                        let mut queue_guard = queue_clone.write().await;

                        let due_tasks: Vec<_> = queue_guard
                            .iter()
                            .filter(|t| t.execute_at <= now)
                            .cloned()
                            .collect();

                        queue_guard.retain(|t| t.execute_at > now);

                        drop(queue_guard);

                        for task in due_tasks {
                            let executor_clone = executor.clone();
                            let handles_ref = handles_clone.clone();

                            tokio::spawn(async move {
                                if let Some(handle) = handles_ref.read().await.get(&task.id) {
                                    handle.set_state(TaskState::Running).await;
                                }

                                let result = executor_clone.execute(task.clone()).await;

                                if let Some(handle) = handles_ref.read().await.get(&task.id) {
                                    match result {
                                        Ok(()) => {
                                            handle.record_execution().await;
                                            handle.set_state(TaskState::Completed).await;
                                        }
                                        Err(e) => {
                                            handle.record_error(e.to_string()).await;
                                            handle.set_state(TaskState::Failed).await;
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });

        let queue_clone = queue.clone();
        tokio::spawn(async move {
            while let Some(task) = task_rx.recv().await {
                let mut queue_guard = queue_clone.write().await;
                queue_guard.push(task);
                queue_guard.sort_by(|a, b| a.execute_at.cmp(&b.execute_at).then_with(|| a.priority.cmp(&b.priority)));
            }
        });

        Self { config, queue, handles, task_tx, shutdown_tx, is_shutdown }
    }

    /// 注册延迟任务
    ///
    /// # 参数
    ///
    /// - `task`: 延迟任务
    ///
    /// # 返回值
    ///
    /// 返回任务句柄。
    pub async fn schedule_delayed(&self, task: DelayedTask<T>) -> WaeResult<TaskHandle> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(WaeError::scheduler_shutdown());
        }

        let handle = TaskHandle::new(task.id.clone(), task.name.clone());

        {
            let mut handles = self.handles.write().await;
            handles.insert(task.id.clone(), handle.clone());
        }

        self.task_tx.send(task).await.map_err(|e| WaeError::internal(format!("Failed to send task: {}", e)))?;

        info!("Scheduled delayed task: {}", handle.name);
        Ok(handle)
    }

    /// 获取队列大小
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> WaeResult<bool> {
        let mut queue = self.queue.write().await;
        let initial_len = queue.len();
        queue.retain(|t| t.id != task_id);

        if queue.len() < initial_len {
            if let Some(handle) = self.handles.read().await.get(task_id) {
                handle.cancel();
                handle.set_state(TaskState::Cancelled).await;
            }
            info!("Cancelled delayed task: {}", task_id);
            Ok(true)
        }
        else {
            Err(WaeError::task_not_found(task_id))
        }
    }

    /// 获取任务句柄
    pub async fn get_handle(&self, task_id: &str) -> Option<TaskHandle> {
        self.handles.read().await.get(task_id).cloned()
    }

    /// 关闭队列
    pub fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        let _ = self.shutdown_tx.send(());
        info!("Delayed queue shutdown initiated");
    }

    /// 检查是否已关闭
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }
}
