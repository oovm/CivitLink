use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_bulkhead_config_new() {
    let config = BulkheadConfig::new(20, Duration::from_secs(10));
    assert_eq!(config.max_concurrent_calls, 20);
    assert_eq!(config.max_wait_duration, Duration::from_secs(10));
}

#[tokio::test]
async fn test_bulkhead_config_builder() {
    let config = BulkheadConfig::default().max_concurrent_calls(50).max_wait_duration(Duration::from_secs(30));

    assert_eq!(config.max_concurrent_calls, 50);
    assert_eq!(config.max_wait_duration, Duration::from_secs(30));
}

#[tokio::test]
async fn test_bulkhead_config_default() {
    let config = BulkheadConfig::default();
    assert_eq!(config.max_concurrent_calls, 10);
    assert_eq!(config.max_wait_duration, Duration::from_secs(5));
}

#[tokio::test]
async fn test_bulkhead_new() {
    let config = BulkheadConfig::new(3, Duration::ZERO);
    let bulkhead = Bulkhead::new(config);
    assert_eq!(bulkhead.max_concurrent_calls(), 3);
    assert_eq!(bulkhead.concurrent_count(), 0);
    assert_eq!(bulkhead.available_permits(), 3);
}

#[tokio::test]
async fn test_bulkhead_with_defaults() {
    let bulkhead = Bulkhead::with_defaults();
    assert_eq!(bulkhead.max_concurrent_calls(), 10);
}

#[tokio::test]
async fn test_bulkhead_try_acquire_success() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(2, Duration::ZERO));

    let permit1 = bulkhead.try_acquire().unwrap();
    assert_eq!(bulkhead.concurrent_count(), 1);
    assert_eq!(bulkhead.available_permits(), 1);

    let permit2 = bulkhead.try_acquire().unwrap();
    assert_eq!(bulkhead.concurrent_count(), 2);
    assert_eq!(bulkhead.available_permits(), 0);

    drop(permit1);
    assert_eq!(bulkhead.concurrent_count(), 1);
    assert_eq!(bulkhead.available_permits(), 1);

    drop(permit2);
    assert_eq!(bulkhead.concurrent_count(), 0);
    assert_eq!(bulkhead.available_permits(), 2);
}

#[tokio::test]
async fn test_bulkhead_try_acquire_full() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(2, Duration::ZERO));

    let _permit1 = bulkhead.try_acquire().unwrap();
    let _permit2 = bulkhead.try_acquire().unwrap();

    assert!(bulkhead.try_acquire().is_err());
}

#[tokio::test]
async fn test_bulkhead_acquire_success() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(2, Duration::ZERO));

    let _permit1 = bulkhead.acquire().await.unwrap();
    let _permit2 = bulkhead.acquire().await.unwrap();

    assert_eq!(bulkhead.concurrent_count(), 2);
}

#[tokio::test]
async fn test_bulkhead_permit_drop() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(3, Duration::ZERO));

    {
        let _permit1 = bulkhead.try_acquire().unwrap();
        let _permit2 = bulkhead.try_acquire().unwrap();
        assert_eq!(bulkhead.concurrent_count(), 2);
    }

    assert_eq!(bulkhead.concurrent_count(), 0);
    assert_eq!(bulkhead.available_permits(), 3);
}

#[tokio::test]
async fn test_bulkhead_execute_success() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(2, Duration::ZERO));
    let result = bulkhead.execute(|| async { Ok::<_, &str>("success") }).await;
    assert_eq!(result.unwrap(), "success");
    assert_eq!(bulkhead.concurrent_count(), 0);
}

#[tokio::test]
async fn test_bulkhead_execute_error() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(2, Duration::ZERO));
    let result = bulkhead.execute(|| async { Err::<(), &str>("error") }).await;
    assert!(result.is_err());
    assert_eq!(bulkhead.concurrent_count(), 0);
}

#[tokio::test]
async fn test_bulkhead_config_accessor() {
    let config = BulkheadConfig::new(15, Duration::from_secs(15));
    let bulkhead = Bulkhead::new(config.clone());
    assert_eq!(bulkhead.config().max_concurrent_calls, 15);
    assert_eq!(bulkhead.config().max_wait_duration, Duration::from_secs(15));
}

#[tokio::test]
async fn test_bulkhead_concurrent_count() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(5, Duration::ZERO));

    assert_eq!(bulkhead.concurrent_count(), 0);

    let permit1 = bulkhead.try_acquire().unwrap();
    assert_eq!(bulkhead.concurrent_count(), 1);

    let permit2 = bulkhead.try_acquire().unwrap();
    assert_eq!(bulkhead.concurrent_count(), 2);

    drop(permit1);
    assert_eq!(bulkhead.concurrent_count(), 1);

    drop(permit2);
    assert_eq!(bulkhead.concurrent_count(), 0);
}
