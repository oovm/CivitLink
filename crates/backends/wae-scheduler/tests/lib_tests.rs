use chrono::{Timelike, Utc};
use std::{sync::Arc, time::Duration};
use wae_scheduler::*;

struct TestTask {
    name: String,
}

#[async_trait::async_trait]
impl ScheduledTask for TestTask {
    async fn execute(&self) -> WaeResult<()> {
        println!("Executing task: {}", self.name);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[tokio::test]
async fn test_interval_scheduler() {
    let scheduler = IntervalScheduler::default_config();
    let task = Arc::new(TestTask { name: "test".to_string() });

    let handle = scheduler.schedule_interval(task, Duration::from_millis(100)).await.unwrap();
    assert_eq!(handle.name, "test");

    tokio::time::sleep(Duration::from_millis(350)).await;

    assert!(handle.execution_count() >= 3);

    scheduler.shutdown().await;
}

#[tokio::test]
async fn test_cron_expression() {
    let expr = CronExpression::parse("0 * * * * *").unwrap();
    let next = expr.next_execution(Utc::now()).unwrap();
    assert!(next > Utc::now());
}

#[tokio::test]
async fn test_cron_expression_daily() {
    let expr = CronExpression::parse("0 30 14 * * *").unwrap();
    let next = expr.next_execution(Utc::now()).unwrap();
    assert_eq!(next.minute(), 30);
    assert_eq!(next.hour(), 14);
}

#[tokio::test]
async fn test_delayed_task() {
    let task = DelayedTask::new("test".to_string(), Utc::now() + chrono::Duration::seconds(1), "data".to_string());
    assert!(!task.is_due());
    assert!(task.remaining() > Duration::ZERO);
}
