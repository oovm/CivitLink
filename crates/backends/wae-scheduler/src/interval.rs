use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::{RwLock, broadcast},
    task::JoinHandle,
};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::task::{ScheduledTask, TaskHandle, TaskId, TaskState};
use wae_types::{WaeError, WaeResult};

/// 间隔任务调度器配置
#[derive(Debug, Clone)]
pub struct IntervalSchedulerConfig {
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 任务执行超时时间
    pub task_timeout: Duration,
    /// 错误重试次数
    pub retry_count: u32,
    /// 错误重试间隔
    pub retry_interval: Duration,
}

impl Default for IntervalSchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            task_timeout: Duration::from_secs(300),
            retry_count: 3,
            retry_interval: Duration::from_secs(1),
        }
    }
}

/// 间隔任务调度器
///
/// 管理周期性执行的定时任务，支持固定间隔执行。
pub struct IntervalScheduler {
    /// 配置
    #[allow(dead_code)]
    config: IntervalSchedulerConfig,
    /// 任务句柄映射
    handles: Arc<RwLock<BTreeMap<TaskId, TaskHandle>>>,
    /// 任务映射
    tasks: Arc<RwLock<BTreeMap<TaskId, Arc<dyn ScheduledTask>>>>,
    /// 任务间隔映射
    intervals: Arc<RwLock<BTreeMap<TaskId, Duration>>>,
    /// 任务句柄映射
    task_handles: Arc<RwLock<BTreeMap<TaskId, JoinHandle<()>>>>,
    /// 关闭信号
    shutdown_tx: broadcast::Sender<()>,
    /// 是否已关闭
    is_shutdown: Arc<AtomicBool>,
}

impl IntervalScheduler {
    /// 创建新的间隔任务调度器
    pub fn new(config: IntervalSchedulerConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            handles: Arc::new(RwLock::new(BTreeMap::new())),
            tasks: Arc::new(RwLock::new(BTreeMap::new())),
            intervals: Arc::new(RwLock::new(BTreeMap::new())),
            task_handles: Arc::new(RwLock::new(BTreeMap::new())),
            shutdown_tx,
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 使用默认配置创建调度器
    pub fn default_config() -> Self {
        Self::new(IntervalSchedulerConfig::default())
    }

    /// 注册固定间隔任务
    ///
    /// # 参数
    ///
    /// - `task`: 要执行的任务
    /// - `interval`: 执行间隔
    ///
    /// # 返回值
    ///
    /// 返回任务句柄，可用于查询状态或取消任务。
    pub async fn schedule_interval(&self, task: Arc<dyn ScheduledTask>, interval: Duration) -> WaeResult<TaskHandle> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(WaeError::scheduler_shutdown());
        }

        let task_id = Uuid::new_v4().to_string();
        let handle = TaskHandle::new(task_id.clone(), task.name().to_string());

        {
            let mut handles = self.handles.write().await;
            handles.insert(task_id.clone(), handle.clone());
        }

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task.clone());
        }

        {
            let mut intervals = self.intervals.write().await;
            intervals.insert(task_id.clone(), interval);
        }

        let task_id_clone = task_id.clone();
        let task_clone = task.clone();
        let handles_clone = self.handles.clone();
        let shutdown_rx = self.shutdown_tx.subscribe();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = cancelled.clone();

        let join_handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            let mut shutdown_rx = shutdown_rx;

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Task {} received shutdown signal", task_id_clone);
                        break;
                    }
                    _ = interval_timer.tick() => {
                        if cancelled_clone.load(Ordering::SeqCst) {
                            debug!("Task {} is cancelled, stopping", task_id_clone);
                            break;
                        }

                        if let Some(h) = handles_clone.read().await.get(&task_id_clone) {
                            h.set_state(TaskState::Running).await;
                        }

                        let result = task_clone.execute().await;

                        if let Some(h) = handles_clone.read().await.get(&task_id_clone) {
                            match result {
                                Ok(()) => {
                                    h.record_execution().await;
                                    h.set_state(TaskState::Pending).await;
                                }
                                Err(e) => {
                                    h.record_error(e.to_string()).await;
                                    h.set_state(TaskState::Failed).await;
                                    error!("Task {} execution failed: {}", task_id_clone, e);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(h) = handles_clone.read().await.get(&task_id_clone) {
                h.set_state(TaskState::Completed).await;
            }
        });

        {
            let mut task_handles = self.task_handles.write().await;
            task_handles.insert(task_id, join_handle);
        }

        info!("Scheduled interval task: {} (interval: {:?})", handle.name, interval);
        Ok(handle)
    }

    /// 取消任务
    ///
    /// # 参数
    ///
    /// - `task_id`: 任务 ID
    ///
    /// # 返回值
    ///
    /// 成功返回 `true`，任务不存在返回 `false`。
    pub async fn cancel_task(&self, task_id: &str) -> WaeResult<bool> {
        let handles = self.handles.read().await;
        if let Some(handle) = handles.get(task_id) {
            handle.cancel();
            handle.set_state(TaskState::Cancelled).await;
            info!("Cancelled task: {}", task_id);
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

    /// 获取所有任务句柄
    pub async fn get_all_handles(&self) -> Vec<TaskHandle> {
        self.handles.read().await.values().cloned().collect()
    }

    /// 关闭调度器
    pub async fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        let _ = self.shutdown_tx.send(());
        info!("Interval scheduler shutdown initiated");
    }

    /// 检查是否已关闭
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }
}
