use std::time::Duration;
use wae_resilience::*;

#[tokio::test]
async fn test_circuit_state_variants() {
    let closed = CircuitState::Closed;
    let open = CircuitState::Open;
    let half_open = CircuitState::HalfOpen;

    assert_ne!(closed, open);
    assert_ne!(closed, half_open);
    assert_ne!(open, half_open);
}

#[tokio::test]
async fn test_circuit_breaker_config_builder() {
    let config = CircuitBreakerConfig::new()
        .failure_threshold(10)
        .success_threshold(5)
        .timeout(Duration::from_secs(60))
        .time_window(Duration::from_secs(120));

    assert_eq!(config.failure_threshold, 10);
    assert_eq!(config.success_threshold, 5);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.time_window, Duration::from_secs(120));
}

#[tokio::test]
async fn test_circuit_breaker_config_default() {
    let config = CircuitBreakerConfig::default();
    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.success_threshold, 3);
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.time_window, Duration::from_secs(60));
}

#[tokio::test]
async fn test_circuit_breaker_new() {
    let config = CircuitBreakerConfig::default();
    let cb = CircuitBreaker::new("test-breaker", config);
    assert_eq!(cb.name(), "test-breaker");
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_with_name() {
    let cb = CircuitBreaker::with_name("test-with-name");
    assert_eq!(cb.name(), "test-with-name");
}

#[tokio::test]
async fn test_circuit_breaker_closed_state() {
    let cb = CircuitBreaker::with_name("test-closed");
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.is_call_allowed());
    assert_eq!(cb.failure_count(), 0);
}

#[tokio::test]
async fn test_circuit_breaker_transition_to_open() {
    let config = CircuitBreakerConfig::new().failure_threshold(3).time_window(Duration::from_secs(60));
    let cb = CircuitBreaker::new("test-open", config);

    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.is_call_allowed());
}

#[tokio::test]
async fn test_circuit_breaker_success_resets_failure_count() {
    let config = CircuitBreakerConfig::new().failure_threshold(3).time_window(Duration::from_secs(60));
    let cb = CircuitBreaker::new("test-reset", config);

    cb.record_failure();
    cb.record_failure();
    assert_eq!(cb.failure_count(), 2);

    cb.record_success();
    assert_eq!(cb.failure_count(), 0);
}

#[tokio::test]
async fn test_circuit_breaker_force_open() {
    let cb = CircuitBreaker::with_name("test-force-open");
    assert_eq!(cb.state(), CircuitState::Closed);

    cb.force_open();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.is_call_allowed());
}

#[tokio::test]
async fn test_circuit_breaker_force_close() {
    let cb = CircuitBreaker::with_name("test-force-close");
    cb.force_open();
    assert_eq!(cb.state(), CircuitState::Open);

    cb.force_close();
    assert_eq!(cb.state(), CircuitState::Closed);
    assert_eq!(cb.failure_count(), 0);
}

#[tokio::test]
async fn test_circuit_breaker_execute_success() {
    let cb = CircuitBreaker::with_name("test-execute-success");
    let result = cb.execute(|| async { Ok::<_, &str>("success") }).await;
    assert_eq!(result.unwrap(), "success");
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_execute_failure() {
    let cb = CircuitBreaker::with_name("test-execute-failure");
    let result = cb.execute(|| async { Err::<(), &str>("failure") }).await;
    assert!(result.is_err());
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_success() {
    let config = CircuitBreakerConfig::new()
        .failure_threshold(1)
        .success_threshold(1)
        .timeout(Duration::from_millis(10))
        .time_window(Duration::from_secs(60));
    let cb = CircuitBreaker::new("test-half-open", config);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);

    tokio::time::sleep(Duration::from_millis(20)).await;

    assert!(cb.is_call_allowed());
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_half_open_failure() {
    let config = CircuitBreakerConfig::new()
        .failure_threshold(1)
        .success_threshold(1)
        .timeout(Duration::from_millis(10))
        .time_window(Duration::from_secs(60));
    let cb = CircuitBreaker::new("test-half-open-failure", config);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);

    tokio::time::sleep(Duration::from_millis(20)).await;

    assert!(cb.is_call_allowed());
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
}

#[tokio::test]
async fn test_circuit_breaker_config_accessor() {
    let config = CircuitBreakerConfig::new().failure_threshold(10);
    let cb = CircuitBreaker::new("test-config", config.clone());
    assert_eq!(cb.config().failure_threshold, 10);
}
