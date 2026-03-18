use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use tokio::sync::RwLock;
use wae_types::WaeResult;

/// 任务 ID 类型
pub type TaskId = String;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// 待执行
    Pending,
    /// 运行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 执行失败
    Failed,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::Pending => write!(f, "pending"),
            TaskState::Running => write!(f, "running"),
            TaskState::Paused => write!(f, "paused"),
            TaskState::Completed => write!(f, "completed"),
            TaskState::Cancelled => write!(f, "cancelled"),
            TaskState::Failed => write!(f, "failed"),
        }
    }
}

/// 定时任务 trait
///
/// 定义定时任务的核心接口，所有需要被调度的任务都需要实现此 trait。
#[async_trait]
pub trait ScheduledTask: Send + Sync {
    /// 执行任务
    ///
    /// # 返回值
    ///
    /// 返回 `Ok(())` 表示任务执行成功，返回 `Err` 表示任务执行失败。
    async fn execute(&self) -> WaeResult<()>;

    /// 获取任务名称
    fn name(&self) -> &str;
}

/// 任务句柄
///
/// 用于管理和查询任务状态。
#[derive(Debug, Clone)]
pub struct TaskHandle {
    /// 任务 ID
    pub id: TaskId,
    /// 任务名称
    pub name: String,
    /// 任务状态
    state: Arc<RwLock<TaskState>>,
    /// 执行次数计数器
    execution_count: Arc<AtomicU64>,
    /// 最后执行时间
    last_execution: Arc<RwLock<Option<DateTime<Utc>>>>,
    /// 最后错误信息
    last_error: Arc<RwLock<Option<String>>>,
    /// 取消标志
    cancelled: Arc<AtomicBool>,
}

impl TaskHandle {
    /// 创建新的任务句柄
    pub fn new(id: TaskId, name: String) -> Self {
        Self {
            id,
            name,
            state: Arc::new(RwLock::new(TaskState::Pending)),
            execution_count: Arc::new(AtomicU64::new(0)),
            last_execution: Arc::new(RwLock::new(None)),
            last_error: Arc::new(RwLock::new(None)),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取任务状态
    pub async fn state(&self) -> TaskState {
        *self.state.read().await
    }

    /// 获取执行次数
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// 获取最后执行时间
    pub async fn last_execution(&self) -> Option<DateTime<Utc>> {
        *self.last_execution.read().await
    }

    /// 获取最后错误信息
    pub async fn last_error(&self) -> Option<String> {
        self.last_error.read().await.clone()
    }

    /// 检查任务是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// 更新状态
    pub async fn set_state(&self, state: TaskState) {
        *self.state.write().await = state;
    }

    /// 记录执行
    pub async fn record_execution(&self) {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        *self.last_execution.write().await = Some(Utc::now());
    }

    /// 记录错误
    pub async fn record_error(&self, error: String) {
        *self.last_error.write().await = Some(error);
    }

    /// 取消任务
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}
