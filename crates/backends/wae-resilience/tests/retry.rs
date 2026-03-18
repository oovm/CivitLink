use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_retry_policy_exponential() {
    let policy = RetryPolicy::exponential(Duration::from_millis(100), Duration::from_secs(30), 2.0);
    assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
    assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(200));
    assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(400));
    assert_eq!(policy.delay_for_attempt(10), Duration::from_secs(30));
}

#[tokio::test]
async fn test_retry_policy_fixed() {
    let policy = RetryPolicy::fixed(Duration::from_millis(500));
    assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(500));
    assert_eq!(policy.delay_for_attempt(5), Duration::from_millis(500));
    assert_eq!(policy.delay_for_attempt(10), Duration::from_millis(500));
}

#[tokio::test]
async fn test_retry_policy_linear() {
    let policy = RetryPolicy::linear(Duration::from_millis(100), Duration::from_millis(50), Duration::from_secs(1));
    assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
    assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(150));
    assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(200));
    assert_eq!(policy.delay_for_attempt(20), Duration::from_secs(1));
}

#[tokio::test]
async fn test_retry_policy_default() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.delay_for_attempt(0), Duration::from_millis(100));
}

#[tokio::test]
async fn test_retry_config_builder() {
    let config = RetryConfig::new(5, RetryPolicy::fixed(Duration::from_millis(100)));
    assert_eq!(config.max_retries, 5);

    let config = config.max_retries(10).policy(RetryPolicy::linear(
        Duration::from_millis(50),
        Duration::from_millis(10),
        Duration::from_secs(5),
    ));
    assert_eq!(config.max_retries, 10);
}

#[tokio::test]
async fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
}

#[tokio::test]
async fn test_retry_context() {
    let mut ctx = RetryContext::new(3);
    assert_eq!(ctx.attempt, 0);
    assert_eq!(ctx.max_retries, 3);
    assert!(ctx.can_retry());

    ctx.attempt = 3;
    assert!(!ctx.can_retry());
}

#[tokio::test]
async fn test_retry_async_success_first_attempt() {
    let config = RetryConfig::new(3, RetryPolicy::fixed(Duration::from_millis(10)));
    let result = retry_async(&config, |_ctx| async { Ok::<_, &str>("success") }).await;
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_retry_async_success_after_retries() {
    let config = RetryConfig::new(3, RetryPolicy::fixed(Duration::from_millis(10)));
    let mut attempts = 0;
    let result = retry_async(&config, |_ctx| {
        attempts += 1;
        async move { if attempts < 3 { Err("temporary error") } else { Ok(42) } }
    })
    .await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempts, 3);
}

#[tokio::test]
async fn test_retry_async_max_retries_exceeded() {
    let config = RetryConfig::new(2, RetryPolicy::fixed(Duration::from_millis(10)));
    let mut attempts = 0;
    let result: Result<i32, _> = retry_async(&config, |_ctx| {
        attempts += 1;
        async { Err::<i32, &str>("persistent error") }
    })
    .await;
    assert!(result.is_err());
    assert_eq!(attempts, 2);
}

#[tokio::test]
async fn test_retry_async_if_should_retry_true() {
    let config = RetryConfig::new(3, RetryPolicy::fixed(Duration::from_millis(10)));
    let mut attempts = 0;
    let result = retry_async_if(
        &config,
        |_ctx| {
            attempts += 1;
            async move { if attempts < 2 { Err("temporary error") } else { Ok("success") } }
        },
        |_e| true,
    )
    .await;
    assert_eq!(result.unwrap(), "success");
    assert_eq!(attempts, 2);
}

#[tokio::test]
async fn test_retry_async_if_should_retry_false() {
    let config = RetryConfig::new(3, RetryPolicy::fixed(Duration::from_millis(10)));
    let mut attempts = 0;
    let result: Result<i32, _> = retry_async_if(
        &config,
        |_ctx| {
            attempts += 1;
            async { Err::<i32, &str>("fatal error") }
        },
        |_e| false,
    )
    .await;
    assert!(result.is_err());
    assert_eq!(attempts, 1);
}

#[tokio::test]
async fn test_retry_async_if_max_retries_exceeded() {
    let config = RetryConfig::new(2, RetryPolicy::fixed(Duration::from_millis(10)));
    let mut attempts = 0;
    let result: Result<i32, _> = retry_async_if(
        &config,
        |_ctx| {
            attempts += 1;
            async { Err::<i32, &str>("persistent error") }
        },
        |_e| true,
    )
    .await;
    assert!(result.is_err());
    assert_eq!(attempts, 2);
}
