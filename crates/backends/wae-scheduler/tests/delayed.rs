use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::{sync::Arc, time::Duration as StdDuration};
use wae_scheduler::{DelayedQueue, DelayedQueueConfig, DelayedTask, DelayedTaskExecutor, TaskState};
use wae_types::WaeResult;

struct TestExecutor;

#[async_trait]
impl DelayedTaskExecutor<String> for TestExecutor {
    async fn execute(&self, task: DelayedTask<String>) -> WaeResult<()> {
        println!("Executing task: {} with data: {}", task.name, task.data);
        Ok(())
    }
}

#[test]
fn test_delayed_task_new() {
    let name = "Test Task".to_string();
    let execute_at = Utc::now() + Duration::hours(1);
    let data = "Test Data".to_string();

    let task = DelayedTask::new(name.clone(), execute_at, data.clone());

    assert_eq!(task.name, name);
    assert_eq!(task.execute_at, execute_at);
    assert_eq!(task.data, data);
    assert_eq!(task.priority, 0);
}

#[test]
fn test_delayed_task_with_priority() {
    let execute_at = Utc::now() + Duration::hours(1);
    let task = DelayedTask::new("Test Task".to_string(), execute_at, "Data".to_string()).with_priority(5);

    assert_eq!(task.priority, 5);
}

#[test]
fn test_delayed_task_is_due() {
    let past_time = Utc::now() - Duration::hours(1);
    let past_task = DelayedTask::new("Past Task".to_string(), past_time, "Data".to_string());
    assert!(past_task.is_due());

    let future_time = Utc::now() + Duration::hours(1);
    let future_task = DelayedTask::new("Future Task".to_string(), future_time, "Data".to_string());
    assert!(!future_task.is_due());
}

#[test]
fn test_delayed_task_remaining() {
    let past_time = Utc::now() - Duration::hours(1);
    let past_task = DelayedTask::new("Past Task".to_string(), past_time, "Data".to_string());
    assert_eq!(past_task.remaining(), StdDuration::ZERO);

    let future_duration = Duration::hours(1);
    let future_time = Utc::now() + future_duration;
    let future_task = DelayedTask::new("Future Task".to_string(), future_time, "Data".to_string());
    let remaining = future_task.remaining();
    assert!(remaining <= future_duration.to_std().unwrap());
    assert!(remaining > StdDuration::ZERO);
}

#[test]
fn test_delayed_queue_config_default() {
    let config = DelayedQueueConfig::default();
    assert_eq!(config.max_queue_size, 10000);
    assert_eq!(config.poll_interval, StdDuration::from_millis(100));
    assert_eq!(config.max_concurrent_executions, 10);
}

#[tokio::test]
async fn test_delayed_queue_new() {
    let config =
        DelayedQueueConfig { max_queue_size: 1000, poll_interval: StdDuration::from_millis(50), max_concurrent_executions: 5 };
    let executor = Arc::new(TestExecutor);
    let queue: DelayedQueue<String> = DelayedQueue::new(config, executor);
    assert!(!queue.is_shutdown());
    queue.shutdown();
}

#[tokio::test]
async fn test_delayed_queue_schedule_task() {
    let config = DelayedQueueConfig::default();
    let executor = Arc::new(TestExecutor);
    let queue: DelayedQueue<String> = DelayedQueue::new(config, executor);

    let execute_at = Utc::now() + Duration::seconds(1);
    let task = DelayedTask::new("Test Task".to_string(), execute_at, "Data".to_string());

    let result = queue.schedule_delayed(task).await;
    assert!(result.is_ok());

    let handle = result.unwrap();
    assert_eq!(handle.name, "Test Task");
    assert_eq!(handle.state().await, TaskState::Pending);
}

#[tokio::test]
async fn test_delayed_queue_queue_size() {
    let config = DelayedQueueConfig::default();
    let executor = Arc::new(TestExecutor);
    let queue: DelayedQueue<String> = DelayedQueue::new(config, executor);

    assert_eq!(queue.queue_size().await, 0);

    let execute_at = Utc::now() + Duration::hours(1);
    let task = DelayedTask::new("Test Task".to_string(), execute_at, "Data".to_string());
    queue.schedule_delayed(task).await.unwrap();

    tokio::time::sleep(StdDuration::from_millis(100)).await;

    assert_eq!(queue.queue_size().await, 1);
}

#[tokio::test]
async fn test_delayed_queue_get_handle() {
    let config = DelayedQueueConfig::default();
    let executor = Arc::new(TestExecutor);
    let queue: DelayedQueue<String> = DelayedQueue::new(config, executor);

    let execute_at = Utc::now() + Duration::hours(1);
    let task = DelayedTask::new("Test Task".to_string(), execute_at, "Data".to_string());

    let handle = queue.schedule_delayed(task).await.unwrap();
    let task_id = handle.id.clone();

    let retrieved = queue.get_handle(&task_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, task_id);
}

#[tokio::test]
async fn test_delayed_queue_cancel_task() {
    let config = DelayedQueueConfig::default();
    let executor = Arc::new(TestExecutor);
    let queue: DelayedQueue<String> = DelayedQueue::new(config, executor);

    let execute_at = Utc::now() + Duration::hours(1);
    let task = DelayedTask::new("Test Task".to_string(), execute_at, "Data".to_string());

    let handle = queue.schedule_delayed(task).await.unwrap();
    let task_id = handle.id.clone();

    tokio::time::sleep(StdDuration::from_millis(100)).await;

    let result = queue.cancel_task(&task_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    let handle = queue.get_handle(&task_id).await;
    assert!(handle.is_some());
    assert!(handle.unwrap().is_cancelled());
}

#[tokio::test]
async fn test_delayed_queue_shutdown() {
    let config = DelayedQueueConfig::default();
    let executor = Arc::new(TestExecutor);
    let queue: DelayedQueue<String> = DelayedQueue::new(config, executor);

    assert!(!queue.is_shutdown());

    queue.shutdown();

    assert!(queue.is_shutdown());

    let execute_at = Utc::now() + Duration::hours(1);
    let task = DelayedTask::new("Test Task".to_string(), execute_at, "Data".to_string());

    let result = queue.schedule_delayed(task).await;
    assert!(result.is_err());
}
