use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use wae_scheduler::{IntervalScheduler, IntervalSchedulerConfig, ScheduledTask, TaskState};
use wae_types::WaeResult;

struct TestTask {
    name: String,
}

#[async_trait]
impl ScheduledTask for TestTask {
    async fn execute(&self) -> WaeResult<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[test]
fn test_interval_scheduler_config_default() {
    let config = IntervalSchedulerConfig::default();
    assert_eq!(config.max_concurrent_tasks, 100);
    assert_eq!(config.task_timeout, Duration::from_secs(300));
    assert_eq!(config.retry_count, 3);
    assert_eq!(config.retry_interval, Duration::from_secs(1));
}

#[tokio::test]
async fn test_interval_scheduler_new() {
    let config = IntervalSchedulerConfig {
        max_concurrent_tasks: 50,
        task_timeout: Duration::from_secs(60),
        retry_count: 5,
        retry_interval: Duration::from_secs(2),
    };
    let scheduler = IntervalScheduler::new(config);
    assert!(!scheduler.is_shutdown());
    scheduler.shutdown().await;
}

#[tokio::test]
async fn test_interval_scheduler_default_config() {
    let scheduler = IntervalScheduler::default_config();
    assert!(!scheduler.is_shutdown());
    scheduler.shutdown().await;
}

#[tokio::test]
async fn test_interval_scheduler_schedule_task() {
    let scheduler = IntervalScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Interval Task".to_string() });

    let interval = Duration::from_secs(1);
    let result = scheduler.schedule_interval(task, interval).await;
    assert!(result.is_ok());

    let handle = result.unwrap();
    assert_eq!(handle.name, "Test Interval Task");
    assert_eq!(handle.state().await, TaskState::Pending);
}

#[tokio::test]
async fn test_interval_scheduler_get_handle() {
    let scheduler = IntervalScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let handle = scheduler.schedule_interval(task, Duration::from_secs(1)).await.unwrap();
    let task_id = handle.id.clone();

    let retrieved = scheduler.get_handle(&task_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, task_id);
}

#[tokio::test]
async fn test_interval_scheduler_get_all_handles() {
    let scheduler = IntervalScheduler::default_config();

    let task1 = Arc::new(TestTask { name: "Task 1".to_string() });

    let task2 = Arc::new(TestTask { name: "Task 2".to_string() });

    scheduler.schedule_interval(task1, Duration::from_secs(1)).await.unwrap();
    scheduler.schedule_interval(task2, Duration::from_secs(1)).await.unwrap();

    let handles = scheduler.get_all_handles().await;
    assert_eq!(handles.len(), 2);
}

#[tokio::test]
async fn test_interval_scheduler_cancel_task() {
    let scheduler = IntervalScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let handle = scheduler.schedule_interval(task, Duration::from_secs(1)).await.unwrap();
    let task_id = handle.id.clone();

    let result = scheduler.cancel_task(&task_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    let handle = scheduler.get_handle(&task_id).await;
    assert!(handle.is_some());
    assert!(handle.unwrap().is_cancelled());
}

#[tokio::test]
async fn test_interval_scheduler_shutdown() {
    let scheduler = IntervalScheduler::default_config();

    assert!(!scheduler.is_shutdown());

    scheduler.shutdown().await;

    assert!(scheduler.is_shutdown());

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let result = scheduler.schedule_interval(task, Duration::from_secs(1)).await;
    assert!(result.is_err());
}
