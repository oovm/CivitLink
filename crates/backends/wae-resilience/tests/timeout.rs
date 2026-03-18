use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_timeout_config_new() {
    let config = TimeoutConfig::new(Duration::from_secs(60), Duration::from_secs(30));
    assert_eq!(config.global_timeout, Duration::from_secs(60));
    assert_eq!(config.operation_timeout, Duration::from_secs(30));
}

#[tokio::test]
async fn test_timeout_config_builder() {
    let config = TimeoutConfig::default().global_timeout(Duration::from_secs(120)).operation_timeout(Duration::from_secs(60));

    assert_eq!(config.global_timeout, Duration::from_secs(120));
    assert_eq!(config.operation_timeout, Duration::from_secs(60));
}

#[tokio::test]
async fn test_timeout_config_default() {
    let config = TimeoutConfig::default();
    assert_eq!(config.global_timeout, Duration::from_secs(30));
    assert_eq!(config.operation_timeout, Duration::from_secs(10));
}

#[tokio::test]
async fn test_with_timeout_success() {
    let result = with_timeout(Duration::from_millis(100), async { Ok::<_, &str>("success") }).await;
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_with_timeout_operation_error() {
    let result = with_timeout(Duration::from_millis(100), async { Err::<(), &str>("operation failed") }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_with_timeout_exceeds_timeout() {
    let result = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<_, &str>("done")
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_with_timeout_raw_success() {
    let result = with_timeout_raw(Duration::from_millis(100), async { Ok::<_, &str>("success") }).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().unwrap(), "success");
}

#[tokio::test]
async fn test_with_timeout_raw_operation_error() {
    let result = with_timeout_raw(Duration::from_millis(100), async { Err::<(), &str>("operation failed") }).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_err());
}

#[tokio::test]
async fn test_with_timeout_raw_exceeds_timeout() {
    let result = with_timeout_raw(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<_, &str>("done")
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_with_timeout_long_operation() {
    let result = with_timeout(Duration::from_millis(200), async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, &str>("completed")
    })
    .await;
    assert_eq!(result.unwrap(), "completed");
}

#[tokio::test]
async fn test_with_timeout_raw_long_operation() {
    let result = with_timeout_raw(Duration::from_millis(200), async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, &str>("completed")
    })
    .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().unwrap(), "completed");
}
