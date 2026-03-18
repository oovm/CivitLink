use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_token_bucket_config_new() {
    let config = TokenBucketConfig::new(100, 50);
    assert_eq!(config.capacity, 100);
    assert_eq!(config.refill_rate, 50);
}

#[tokio::test]
async fn test_token_bucket_config_default() {
    let config = TokenBucketConfig::default();
    assert_eq!(config.capacity, 100);
    assert_eq!(config.refill_rate, 10);
}

#[tokio::test]
async fn test_token_bucket_new() {
    let config = TokenBucketConfig::new(10, 5);
    let bucket = TokenBucket::new(config);
    assert_eq!(bucket.available_permits(), 10);
}

#[tokio::test]
async fn test_token_bucket_with_defaults() {
    let bucket = TokenBucket::with_defaults();
    assert_eq!(bucket.available_permits(), 100);
}

#[tokio::test]
async fn test_token_bucket_try_acquire_success() {
    let bucket = TokenBucket::new(TokenBucketConfig::new(5, 1));
    for _ in 0..5 {
        assert!(bucket.try_acquire().is_ok());
    }
}

#[tokio::test]
async fn test_token_bucket_try_acquire_empty() {
    let bucket = TokenBucket::new(TokenBucketConfig::new(5, 1));
    for _ in 0..5 {
        bucket.try_acquire().unwrap();
    }
    assert!(bucket.try_acquire().is_err());
}

#[tokio::test]
async fn test_token_bucket_available_permits() {
    let bucket = TokenBucket::new(TokenBucketConfig::new(10, 1));
    assert_eq!(bucket.available_permits(), 10);

    bucket.try_acquire().unwrap();
    bucket.try_acquire().unwrap();
    assert_eq!(bucket.available_permits(), 8);
}

#[tokio::test]
async fn test_token_bucket_acquire() {
    let bucket = TokenBucket::new(TokenBucketConfig::new(3, 100));
    for _ in 0..3 {
        assert!(bucket.acquire().await.is_ok());
    }
}

#[tokio::test]
async fn test_sliding_window_config_new() {
    let config = SlidingWindowConfig::new(Duration::from_secs(2), 50);
    assert_eq!(config.window_size, Duration::from_secs(2));
    assert_eq!(config.max_requests, 50);
}

#[tokio::test]
async fn test_sliding_window_config_default() {
    let config = SlidingWindowConfig::default();
    assert_eq!(config.window_size, Duration::from_secs(1));
    assert_eq!(config.max_requests, 100);
}

#[tokio::test]
async fn test_sliding_window_new() {
    let config = SlidingWindowConfig::new(Duration::from_secs(1), 5);
    let window = SlidingWindow::new(config);
    assert_eq!(window.available_permits(), 5);
}

#[tokio::test]
async fn test_sliding_window_with_defaults() {
    let window = SlidingWindow::with_defaults();
    assert_eq!(window.available_permits(), 100);
}

#[tokio::test]
async fn test_sliding_window_try_acquire_success() {
    let window = SlidingWindow::new(SlidingWindowConfig::new(Duration::from_millis(100), 3));
    for _ in 0..3 {
        assert!(window.try_acquire().is_ok());
    }
}

#[tokio::test]
async fn test_sliding_window_try_acquire_full() {
    let window = SlidingWindow::new(SlidingWindowConfig::new(Duration::from_millis(100), 3));
    for _ in 0..3 {
        window.try_acquire().unwrap();
    }
    assert!(window.try_acquire().is_err());
}

#[tokio::test]
async fn test_sliding_window_available_permits() {
    let window = SlidingWindow::new(SlidingWindowConfig::new(Duration::from_millis(100), 5));
    assert_eq!(window.available_permits(), 5);

    window.try_acquire().unwrap();
    window.try_acquire().unwrap();
    assert_eq!(window.available_permits(), 3);
}

#[tokio::test]
async fn test_sliding_window_acquire() {
    let window = SlidingWindow::new(SlidingWindowConfig::new(Duration::from_millis(100), 3));
    for _ in 0..3 {
        assert!(window.acquire().await.is_ok());
    }
}

#[tokio::test]
async fn test_sliding_window_cleanup() {
    let window = SlidingWindow::new(SlidingWindowConfig::new(Duration::from_millis(50), 3));

    window.try_acquire().unwrap();
    window.try_acquire().unwrap();
    assert_eq!(window.available_permits(), 1);

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(window.available_permits(), 3);
}
