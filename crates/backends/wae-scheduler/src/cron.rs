use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    interval::IntervalScheduler,
    task::{ScheduledTask, TaskHandle, TaskId, TaskState},
};
use wae_types::{WaeError, WaeResult};

/// Cron 字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CronField {
    /// 任意值 (*)
    Any,
    /// 固定值
    Value(u32),
    /// 范围值
    Range(u32, u32),
    /// 步进值
    Step(u32, u32),
    /// 多个值
    List(Vec<u32>),
}

impl CronField {
    /// 检查值是否匹配
    pub fn matches(&self, value: u32) -> bool {
        match self {
            CronField::Any => true,
            CronField::Value(v) => *v == value,
            CronField::Range(start, end) => value >= *start && value <= *end,
            CronField::Step(start, step) => (value - start).is_multiple_of(*step),
            CronField::List(values) => values.contains(&value),
        }
    }
}

/// Cron 表达式
///
/// 解析和计算标准 cron 表达式 (秒 分 时 日 月 周)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExpression {
    /// 秒字段
    pub second: CronField,
    /// 分字段
    pub minute: CronField,
    /// 时字段
    pub hour: CronField,
    /// 日字段
    pub day_of_month: CronField,
    /// 月字段
    pub month: CronField,
    /// 周字段
    pub day_of_week: CronField,
}

impl CronExpression {
    /// 解析 cron 表达式
    ///
    /// 支持标准格式: `秒 分 时 日 月 周`
    ///
    /// # 示例
    ///
    /// - `0 * * * * *` - 每分钟执行
    /// - `0 0 * * * *` - 每小时执行
    /// - `0 0 0 * * *` - 每天执行
    /// - `0 30 14 * * *` - 每天 14:30 执行
    /// - `0 0 9-17 * * 1-5` - 周一到周五 9点到17点每小时执行
    pub fn parse(expression: &str) -> WaeResult<Self> {
        let parts: Vec<&str> = expression.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(WaeError::invalid_cron_expression("Expected 6 fields: second minute hour day month weekday"));
        }

        Ok(Self {
            second: Self::parse_field(parts[0], 0, 59)?,
            minute: Self::parse_field(parts[1], 0, 59)?,
            hour: Self::parse_field(parts[2], 0, 23)?,
            day_of_month: Self::parse_field(parts[3], 1, 31)?,
            month: Self::parse_field(parts[4], 1, 12)?,
            day_of_week: Self::parse_field(parts[5], 0, 6)?,
        })
    }

    fn parse_field(s: &str, min: u32, max: u32) -> WaeResult<CronField> {
        if s == "*" {
            return Ok(CronField::Any);
        }

        if s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() != 2 {
                return Err(WaeError::invalid_cron_expression(format!("Invalid step expression: {}", s)));
            }
            let start = if parts[0] == "*" {
                min
            }
            else {
                parts[0]
                    .parse::<u32>()
                    .map_err(|_| WaeError::invalid_cron_expression(format!("Invalid value: {}", parts[0])))?
            };
            let step = parts[1]
                .parse::<u32>()
                .map_err(|_| WaeError::invalid_cron_expression(format!("Invalid step: {}", parts[1])))?;
            return Ok(CronField::Step(start, step));
        }

        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 {
                return Err(WaeError::invalid_cron_expression(format!("Invalid range expression: {}", s)));
            }
            let start = parts[0]
                .parse::<u32>()
                .map_err(|_| WaeError::invalid_cron_expression(format!("Invalid start: {}", parts[0])))?;
            let end =
                parts[1].parse::<u32>().map_err(|_| WaeError::invalid_cron_expression(format!("Invalid end: {}", parts[1])))?;
            return Ok(CronField::Range(start, end));
        }

        if s.contains(',') {
            let values: Result<Vec<u32>, _> = s.split(',').map(|v| v.parse::<u32>()).collect();
            let values = values.map_err(|_| WaeError::invalid_cron_expression(format!("Invalid list: {}", s)))?;
            return Ok(CronField::List(values));
        }

        let value = s.parse::<u32>().map_err(|_| WaeError::invalid_cron_expression(format!("Invalid value: {}", s)))?;

        if value < min || value > max {
            return Err(WaeError::invalid_cron_expression(format!("Value {} out of range [{}, {}]", value, min, max)));
        }

        Ok(CronField::Value(value))
    }

    /// 计算下一次执行时间
    pub fn next_execution(&self, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let mut current = from + chrono::Duration::seconds(1);

        for _ in 0..366 * 24 * 60 * 60 {
            let second = current.second();
            let minute = current.minute();
            let hour = current.hour();
            let day = current.day();
            let month = current.month();
            let weekday = current.weekday().num_days_from_sunday();

            if !self.second.matches(second) {
                current += chrono::Duration::seconds(1);
                continue;
            }
            if !self.minute.matches(minute) {
                current += chrono::Duration::minutes(1);
                current = current.with_second(0).unwrap();
                continue;
            }
            if !self.hour.matches(hour) {
                current += chrono::Duration::hours(1);
                current = current.with_second(0).unwrap().with_minute(0).unwrap();
                continue;
            }
            if !self.day_of_month.matches(day) {
                current += chrono::Duration::days(1);
                current = current.with_second(0).unwrap().with_minute(0).unwrap().with_hour(0).unwrap();
                continue;
            }
            if !self.month.matches(month) {
                current += chrono::Duration::days(28);
                continue;
            }
            if !self.day_of_week.matches(weekday) {
                current += chrono::Duration::days(1);
                current = current.with_second(0).unwrap().with_minute(0).unwrap().with_hour(0).unwrap();
                continue;
            }

            return Some(current);
        }

        None
    }
}

/// Cron 任务
pub struct CronTask {
    /// 任务 ID
    pub id: TaskId,
    /// 任务名称
    pub name: String,
    /// Cron 表达式
    pub expression: CronExpression,
    /// 任务
    pub task: Arc<dyn ScheduledTask>,
}

impl std::fmt::Debug for CronTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CronTask")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("expression", &self.expression)
            .finish()
    }
}

/// Cron 调度器配置
#[derive(Debug, Clone)]
pub struct CronSchedulerConfig {
    /// 轮询间隔
    pub poll_interval: Duration,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
}

impl Default for CronSchedulerConfig {
    fn default() -> Self {
        Self { poll_interval: Duration::from_secs(1), max_concurrent_tasks: 100 }
    }
}

/// Cron 调度器
///
/// 基于 Cron 表达式的任务调度器。
pub struct CronScheduler {
    /// 配置
    #[allow(dead_code)]
    config: CronSchedulerConfig,
    /// Cron 任务映射
    cron_tasks: Arc<RwLock<BTreeMap<TaskId, CronTask>>>,
    /// 任务句柄映射
    handles: Arc<RwLock<BTreeMap<TaskId, TaskHandle>>>,
    /// 下次执行时间映射
    next_executions: Arc<RwLock<BTreeMap<TaskId, DateTime<Utc>>>>,
    /// 关闭信号
    shutdown_tx: broadcast::Sender<()>,
    /// 是否已关闭
    is_shutdown: Arc<AtomicBool>,
}

impl CronScheduler {
    /// 创建新的 Cron 调度器
    pub fn new(config: CronSchedulerConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let cron_tasks: Arc<RwLock<BTreeMap<TaskId, CronTask>>> = Arc::new(RwLock::new(BTreeMap::new()));
        let handles: Arc<RwLock<BTreeMap<TaskId, TaskHandle>>> = Arc::new(RwLock::new(BTreeMap::new()));
        let next_executions = Arc::new(RwLock::new(BTreeMap::new()));
        let is_shutdown = Arc::new(AtomicBool::new(false));

        let cron_tasks_clone = cron_tasks.clone();
        let handles_clone = handles.clone();
        let next_executions_clone = next_executions.clone();
        let is_shutdown_clone = is_shutdown.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();
        let poll_interval = config.poll_interval;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Cron scheduler received shutdown signal");
                        break;
                    }
                    _ = tokio::time::sleep(poll_interval) => {
                        if is_shutdown_clone.load(Ordering::SeqCst) {
                            break;
                        }

                        let now = Utc::now();
                        let tasks = cron_tasks_clone.read().await;

                        for (task_id, cron_task) in tasks.iter() {
                            let next_exec = next_executions_clone.read().await.get(task_id).cloned();

                            if let Some(next) = next_exec
                                && now >= next
                            {
                                let task = cron_task.task.clone();
                                let handle = handles_clone.read().await.get(task_id).cloned();
                                let expression = cron_task.expression.clone();
                                let next_executions_ref = next_executions_clone.clone();
                                let task_id_clone = task_id.clone();

                                tokio::spawn(async move {
                                    if let Some(h) = &handle {
                                        h.set_state(TaskState::Running).await;
                                    }

                                    let result = task.execute().await;

                                    if let Some(h) = &handle {
                                        match result {
                                            Ok(()) => {
                                                h.record_execution().await;
                                                h.set_state(TaskState::Pending).await;
                                            }
                                            Err(e) => {
                                                h.record_error(e.to_string()).await;
                                                h.set_state(TaskState::Failed).await;
                                                error!("Cron task {} execution failed: {}", h.name, e);
                                            }
                                        }
                                    }

                                    if let Some(next_time) = expression.next_execution(now) {
                                        next_executions_ref.write().await.insert(task_id_clone, next_time);
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });

        Self { config, cron_tasks, handles, next_executions, shutdown_tx, is_shutdown }
    }

    /// 使用默认配置创建调度器
    pub fn default_config() -> Self {
        Self::new(CronSchedulerConfig::default())
    }

    /// 注册 Cron 任务
    ///
    /// # 参数
    ///
    /// - `task`: 要执行的任务
    /// - `expression`: Cron 表达式
    ///
    /// # 返回值
    ///
    /// 返回任务句柄。
    pub async fn schedule_cron(&self, task: Arc<dyn ScheduledTask>, expression: &str) -> WaeResult<TaskHandle> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(WaeError::scheduler_shutdown());
        }

        let cron_expr = CronExpression::parse(expression)?;
        let task_id = Uuid::new_v4().to_string();
        let handle = TaskHandle::new(task_id.clone(), task.name().to_string());

        let next_execution = cron_expr
            .next_execution(Utc::now())
            .ok_or_else(|| WaeError::invalid_cron_expression("Cannot determine next execution time"))?;

        {
            let mut handles = self.handles.write().await;
            handles.insert(task_id.clone(), handle.clone());
        }

        {
            let cron_task = CronTask { id: task_id.clone(), name: task.name().to_string(), expression: cron_expr, task };
            let mut tasks = self.cron_tasks.write().await;
            tasks.insert(task_id.clone(), cron_task);
        }

        {
            let mut next_execs = self.next_executions.write().await;
            next_execs.insert(task_id.clone(), next_execution);
        }

        info!("Scheduled cron task: {} (expression: {})", handle.name, expression);
        Ok(handle)
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> WaeResult<bool> {
        {
            let mut tasks = self.cron_tasks.write().await;
            tasks.remove(task_id);
        }

        {
            let mut next_execs = self.next_executions.write().await;
            next_execs.remove(task_id);
        }

        let mut handles = self.handles.write().await;
        if let Some(handle) = handles.remove(task_id) {
            handle.cancel();
            handle.set_state(TaskState::Cancelled).await;
            info!("Cancelled cron task: {}", task_id);
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

    /// 获取下次执行时间
    pub async fn get_next_execution(&self, task_id: &str) -> Option<DateTime<Utc>> {
        self.next_executions.read().await.get(task_id).cloned()
    }

    /// 关闭调度器
    pub fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
        let _ = self.shutdown_tx.send(());
        info!("Cron scheduler shutdown initiated");
    }

    /// 检查是否已关闭
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }
}

/// 便捷函数：创建间隔任务调度器
pub fn interval_scheduler() -> IntervalScheduler {
    IntervalScheduler::default_config()
}

/// 便捷函数：创建 Cron 调度器
pub fn cron_scheduler() -> CronScheduler {
    CronScheduler::default_config()
}
