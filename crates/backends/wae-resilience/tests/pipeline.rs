use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_resilience_pipeline_config_default() {
    let config = ResiliencePipelineConfig::default();
    assert!(config.circuit_breaker.is_none());
    assert!(config.rate_limiter.is_none());
    assert!(config.retry.is_none());
    assert!(config.timeout.is_none());
    assert!(config.bulkhead.is_none());
}

#[tokio::test]
async fn test_resilience_pipeline_builder() {
    let pipeline = ResiliencePipelineBuilder::new("test-builder")
        .circuit_breaker(CircuitBreakerConfig::default())
        .rate_limiter(TokenBucketConfig::default())
        .retry(RetryConfig::default())
        .timeout(TimeoutConfig::default())
        .bulkhead(BulkheadConfig::default())
        .build();

    assert_eq!(pipeline.name(), "test-builder");
    assert!(pipeline.circuit_breaker().is_some());
    assert!(pipeline.bulkhead().is_some());
}

#[tokio::test]
async fn test_resilience_pipeline_new() {
    let config = ResiliencePipelineConfig {
        circuit_breaker: Some(CircuitBreakerConfig::default()),
        rate_limiter: Some(TokenBucketConfig::default()),
        retry: Some(RetryConfig::default()),
        timeout: Some(TimeoutConfig::default()),
        bulkhead: Some(BulkheadConfig::default()),
    };

    let pipeline = ResiliencePipeline::new("test-pipeline", config);
    assert_eq!(pipeline.name(), "test-pipeline");
    assert!(pipeline.circuit_breaker().is_some());
    assert!(pipeline.bulkhead().is_some());
}

#[tokio::test]
async fn test_resilience_pipeline_empty_execute_success() {
    let config = ResiliencePipelineConfig::default();
    let pipeline = ResiliencePipeline::new("test-empty", config);

    let result = pipeline.execute(|| async { Ok::<_, &str>("success") }).await;
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_resilience_pipeline_empty_execute_error() {
    let config = ResiliencePipelineConfig::default();
    let pipeline = ResiliencePipeline::new("test-empty-error", config);

    let result = pipeline.execute(|| async { Err::<(), &str>("error") }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resilience_pipeline_with_retry() {
    let config = ResiliencePipelineConfig {
        retry: Some(RetryConfig::new(2, RetryPolicy::fixed(Duration::from_millis(10)))),
        ..Default::default()
    };

    let pipeline = ResiliencePipeline::new("test-retry", config);
    let attempts = std::sync::Arc::new(std::sync::Mutex::new(0));

    let result = pipeline
        .execute(|| {
            let attempts_clone = attempts.clone();
            async move {
                let mut lock = attempts_clone.lock().unwrap();
                *lock += 1;
                let current = *lock;
                if current < 2 { Err("temporary error") } else { Ok("success") }
            }
        })
        .await;

    assert_eq!(result.unwrap(), "success");
    assert_eq!(*attempts.lock().unwrap(), 2);
}

#[tokio::test]
async fn test_resilience_pipeline_with_timeout() {
    let config = ResiliencePipelineConfig {
        timeout: Some(TimeoutConfig::new(Duration::from_secs(30), Duration::from_millis(10))),
        ..Default::default()
    };

    let pipeline = ResiliencePipeline::new("test-timeout", config);

    let result = pipeline
        .execute(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<_, &str>("done")
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_resilience_pipeline_with_circuit_breaker() {
    let config = ResiliencePipelineConfig {
        circuit_breaker: Some(CircuitBreakerConfig::new().failure_threshold(2).time_window(Duration::from_secs(60))),
        ..Default::default()
    };

    let pipeline = ResiliencePipeline::new("test-cb", config);

    pipeline.execute(|| async { Err::<(), &str>("error") }).await.unwrap_err();
    assert_eq!(pipeline.circuit_breaker().unwrap().state(), CircuitState::Closed);

    pipeline.execute(|| async { Err::<(), &str>("error") }).await.unwrap_err();
    assert_eq!(pipeline.circuit_breaker().unwrap().state(), CircuitState::Open);

    let result = pipeline.execute(|| async { Ok::<_, &str>("success") }).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_resilience_pipeline_with_bulkhead() {
    let config = ResiliencePipelineConfig { bulkhead: Some(BulkheadConfig::new(1, Duration::ZERO)), ..Default::default() };

    let pipeline = ResiliencePipeline::new("test-bulkhead", config);

    let result = pipeline.execute(|| async { Ok::<_, &str>("success") }).await;
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_resilience_pipeline_with_rate_limiter() {
    let config = ResiliencePipelineConfig { rate_limiter: Some(TokenBucketConfig::new(3, 100)), ..Default::default() };

    let pipeline = ResiliencePipeline::new("test-rate-limiter", config);

    for _ in 0..3 {
        let result = pipeline.execute(|| async { Ok::<_, &str>("success") }).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_resilience_pipeline_accessors() {
    let config = ResiliencePipelineConfig {
        circuit_breaker: Some(CircuitBreakerConfig::default()),
        bulkhead: Some(BulkheadConfig::default()),
        ..Default::default()
    };

    let pipeline = ResiliencePipeline::new("test-accessors", config);

    assert_eq!(pipeline.name(), "test-accessors");
    assert!(pipeline.circuit_breaker().is_some());
    assert!(pipeline.bulkhead().is_some());
}

#[tokio::test]
async fn test_resilience_pipeline_record_success() {
    let config = ResiliencePipelineConfig {
        circuit_breaker: Some(CircuitBreakerConfig::new().failure_threshold(2).time_window(Duration::from_secs(60))),
        ..Default::default()
    };

    let pipeline = ResiliencePipeline::new("test-record-success", config);

    pipeline.execute(|| async { Err::<(), &str>("error") }).await.unwrap_err();

    pipeline.execute(|| async { Ok::<_, &str>("success") }).await.unwrap();

    assert_eq!(pipeline.circuit_breaker().unwrap().failure_count(), 0);
}
