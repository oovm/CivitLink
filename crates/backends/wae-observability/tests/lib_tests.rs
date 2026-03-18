use wae_observability::{HealthCheck, HealthStatus, ObservabilityConfig};

#[test]
fn test_health_check() {
    let check = HealthCheck::new("database".to_string(), HealthStatus::Passing);
    assert_eq!(check.name, "database");
}

#[test]
fn test_observability_config_default() {
    let config = ObservabilityConfig::default();
    assert_eq!(config.service_name, "wae-service");
}
