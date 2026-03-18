use async_trait::async_trait;
use wae_scheduler::{ScheduledTask, TaskHandle, TaskId, TaskState};
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
fn test_task_state_display() {
    assert_eq!(format!("{}", TaskState::Pending), "pending");
    assert_eq!(format!("{}", TaskState::Running), "running");
    assert_eq!(format!("{}", TaskState::Paused), "paused");
    assert_eq!(format!("{}", TaskState::Completed), "completed");
    assert_eq!(format!("{}", TaskState::Cancelled), "cancelled");
    assert_eq!(format!("{}", TaskState::Failed), "failed");
}

#[test]
fn test_task_state_equality() {
    assert_eq!(TaskState::Pending, TaskState::Pending);
    assert_eq!(TaskState::Running, TaskState::Running);
    assert_ne!(TaskState::Pending, TaskState::Running);
}

#[test]
fn test_task_handle_creation() {
    let id: TaskId = "test-id-123".to_string();
    let name = "Test Task".to_string();
    let handle = TaskHandle::new(id.clone(), name.clone());

    assert_eq!(handle.id, id);
    assert_eq!(handle.name, name);
}

#[tokio::test]
async fn test_task_handle_state() {
    let handle = TaskHandle::new("test-id".to_string(), "Test".to_string());

    assert_eq!(handle.state().await, TaskState::Pending);

    handle.set_state(TaskState::Running).await;
    assert_eq!(handle.state().await, TaskState::Running);

    handle.set_state(TaskState::Completed).await;
    assert_eq!(handle.state().await, TaskState::Completed);
}

#[tokio::test]
async fn test_task_handle_execution_count() {
    let handle = TaskHandle::new("test-id".to_string(), "Test".to_string());

    assert_eq!(handle.execution_count(), 0);

    handle.record_execution().await;
    assert_eq!(handle.execution_count(), 1);

    handle.record_execution().await;
    assert_eq!(handle.execution_count(), 2);
}

#[tokio::test]
async fn test_task_handle_last_execution() {
    let handle = TaskHandle::new("test-id".to_string(), "Test".to_string());

    assert!(handle.last_execution().await.is_none());

    handle.record_execution().await;
    assert!(handle.last_execution().await.is_some());
}

#[tokio::test]
async fn test_task_handle_error() {
    let handle = TaskHandle::new("test-id".to_string(), "Test".to_string());

    assert!(handle.last_error().await.is_none());

    let error_msg = "Something went wrong".to_string();
    handle.record_error(error_msg.clone()).await;

    assert_eq!(handle.last_error().await, Some(error_msg));
}

#[test]
fn test_task_handle_cancel() {
    let handle = TaskHandle::new("test-id".to_string(), "Test".to_string());

    assert!(!handle.is_cancelled());

    handle.cancel();

    assert!(handle.is_cancelled());
}

#[tokio::test]
async fn test_scheduled_task_trait() {
    let task = TestTask { name: "My Task".to_string() };

    assert_eq!(task.name(), "My Task");
    assert!(task.execute().await.is_ok());
}
