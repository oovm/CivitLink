use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_circuit_breaker() {
    let cb = CircuitBreaker::with_name("test");

    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.is_call_allowed());

    for _ in 0..5 {
        cb.record_failure();
    }

    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.is_call_allowed());
}

#[tokio::test]
async fn test_token_bucket() {
    let bucket = TokenBucket::new(TokenBucketConfig::new(5, 1));

    for _ in 0..5 {
        assert!(bucket.try_acquire().is_ok());
    }

    assert!(bucket.try_acquire().is_err());
}

#[tokio::test]
async fn test_sliding_window() {
    let window = SlidingWindow::new(SlidingWindowConfig::new(Duration::from_millis(100), 3));

    for _ in 0..3 {
        assert!(window.try_acquire().is_ok());
    }

    assert!(window.try_acquire().is_err());
}

#[tokio::test]
async fn test_retry() {
    let config = RetryConfig::new(3, RetryPolicy::fixed(Duration::from_millis(10)));

    let result = retry_async(&config, |ctx| async move { if ctx.attempt < 2 { Err("temporary error") } else { Ok(42) } }).await;

    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_timeout() {
    let result = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<_, &str>("done")
    })
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bulkhead() {
    let bulkhead = Bulkhead::new(BulkheadConfig::new(2, Duration::ZERO));

    let permit1 = bulkhead.try_acquire().unwrap();
    let _permit2 = bulkhead.try_acquire().unwrap();

    assert!(bulkhead.try_acquire().is_err());

    drop(permit1);
    assert!(bulkhead.try_acquire().is_ok());
}
