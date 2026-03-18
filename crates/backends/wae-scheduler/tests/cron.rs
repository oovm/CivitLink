use async_trait::async_trait;
use chrono::Utc;
use std::{sync::Arc, time::Duration as StdDuration};
use wae_scheduler::{
    CronExpression, CronField, CronScheduler, CronSchedulerConfig, ScheduledTask, TaskState, cron_scheduler, interval_scheduler,
};
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
fn test_cron_field_matches_any() {
    let field = CronField::Any;
    assert!(field.matches(0));
    assert!(field.matches(59));
    assert!(field.matches(100));
}

#[test]
fn test_cron_field_matches_value() {
    let field = CronField::Value(30);
    assert!(field.matches(30));
    assert!(!field.matches(0));
    assert!(!field.matches(59));
}

#[test]
fn test_cron_field_matches_range() {
    let field = CronField::Range(10, 20);
    assert!(field.matches(10));
    assert!(field.matches(15));
    assert!(field.matches(20));
    assert!(!field.matches(9));
    assert!(!field.matches(21));
}

#[test]
fn test_cron_field_matches_step() {
    let field = CronField::Step(0, 5);
    assert!(field.matches(0));
    assert!(field.matches(5));
    assert!(field.matches(10));
    assert!(!field.matches(1));
    assert!(!field.matches(6));
}

#[test]
fn test_cron_field_matches_list() {
    let field = CronField::List(vec![5, 10, 15]);
    assert!(field.matches(5));
    assert!(field.matches(10));
    assert!(field.matches(15));
    assert!(!field.matches(0));
    assert!(!field.matches(20));
}

#[test]
fn test_cron_expression_parse_valid() {
    let expr = CronExpression::parse("0 * * * * *");
    assert!(expr.is_ok());

    let expr = CronExpression::parse("0 0 * * * *");
    assert!(expr.is_ok());

    let expr = CronExpression::parse("0 30 14 * * *");
    assert!(expr.is_ok());
}

#[test]
fn test_cron_expression_parse_invalid() {
    let expr = CronExpression::parse("invalid");
    assert!(expr.is_err());

    let expr = CronExpression::parse("0 * * * *");
    assert!(expr.is_err());

    let expr = CronExpression::parse("0 0 0 32 * *");
    assert!(expr.is_err());

    let expr = CronExpression::parse("0 0 0 * 13 *");
    assert!(expr.is_err());
}

#[test]
fn test_cron_expression_next_execution() {
    let expr = CronExpression::parse("0 * * * * *").unwrap();
    let now = Utc::now();
    let next = expr.next_execution(now);
    assert!(next.is_some());
    assert!(next.unwrap() > now);
}

#[test]
fn test_cron_scheduler_config_default() {
    let config = CronSchedulerConfig::default();
    assert_eq!(config.poll_interval, StdDuration::from_secs(1));
    assert_eq!(config.max_concurrent_tasks, 100);
}

#[tokio::test]
async fn test_cron_scheduler_new() {
    let config = CronSchedulerConfig { poll_interval: StdDuration::from_secs(2), max_concurrent_tasks: 50 };
    let scheduler = CronScheduler::new(config);
    assert!(!scheduler.is_shutdown());
    scheduler.shutdown();
}

#[tokio::test]
async fn test_cron_scheduler_default_config() {
    let scheduler = CronScheduler::default_config();
    assert!(!scheduler.is_shutdown());
    scheduler.shutdown();
}

#[tokio::test]
async fn test_cron_scheduler_convenience_functions() {
    let scheduler = cron_scheduler();
    assert!(!scheduler.is_shutdown());
    scheduler.shutdown();

    let interval = interval_scheduler();
    assert!(!interval.is_shutdown());
    interval.shutdown().await;
}

#[tokio::test]
async fn test_cron_scheduler_schedule_task() {
    let scheduler = CronScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Cron Task".to_string() });

    let result = scheduler.schedule_cron(task, "0 * * * * *").await;
    assert!(result.is_ok());

    let handle = result.unwrap();
    assert_eq!(handle.name, "Test Cron Task");
    assert_eq!(handle.state().await, TaskState::Pending);
}

#[tokio::test]
async fn test_cron_scheduler_get_handle() {
    let scheduler = CronScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let handle = scheduler.schedule_cron(task, "0 * * * * *").await.unwrap();
    let task_id = handle.id.clone();

    let retrieved = scheduler.get_handle(&task_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, task_id);
}

#[tokio::test]
async fn test_cron_scheduler_get_all_handles() {
    let scheduler = CronScheduler::default_config();

    let task1 = Arc::new(TestTask { name: "Task 1".to_string() });

    let task2 = Arc::new(TestTask { name: "Task 2".to_string() });

    scheduler.schedule_cron(task1, "0 * * * * *").await.unwrap();
    scheduler.schedule_cron(task2, "0 * * * * *").await.unwrap();

    let handles = scheduler.get_all_handles().await;
    assert_eq!(handles.len(), 2);
}

#[tokio::test]
async fn test_cron_scheduler_get_next_execution() {
    let scheduler = CronScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let handle = scheduler.schedule_cron(task, "0 * * * * *").await.unwrap();
    let next_exec = scheduler.get_next_execution(&handle.id).await;
    assert!(next_exec.is_some());
}

#[tokio::test]
async fn test_cron_scheduler_cancel_task() {
    let scheduler = CronScheduler::default_config();

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let handle = scheduler.schedule_cron(task, "0 * * * * *").await.unwrap();
    let task_id = handle.id.clone();

    let result = scheduler.cancel_task(&task_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    let handle = scheduler.get_handle(&task_id).await;
    assert!(handle.is_none());
}

#[tokio::test]
async fn test_cron_scheduler_shutdown() {
    let scheduler = CronScheduler::default_config();

    assert!(!scheduler.is_shutdown());

    scheduler.shutdown();

    assert!(scheduler.is_shutdown());

    let task = Arc::new(TestTask { name: "Test Task".to_string() });

    let result = scheduler.schedule_cron(task, "0 * * * * *").await;
    assert!(result.is_err());
}
