use wae_observability::*;

#[test]
fn test_health_check() {
    let check = HealthCheck::new("database".to_string(), HealthStatus::Passing);
    assert_eq!(check.name, "database");
    assert_eq!(check.status, HealthStatus::Passing);
}

#[test]
fn test_observability_config_default() {
    let config = ObservabilityConfig::default();
    assert_eq!(config.service_name, "wae-service");
    assert_eq!(config.service_version, "0.1.0");
}

#[test]
fn test_observability_config_new() {
    let config = ObservabilityConfig::new("test-service".to_string(), "1.0.0".to_string());
    assert_eq!(config.service_name, "test-service");
    assert_eq!(config.service_version, "1.0.0");
}

#[test]
fn test_health_status_default() {
    let status = HealthStatus::default();
    assert_eq!(status, HealthStatus::Passing);
}

#[test]
fn test_health_report() {
    let checks = vec![
        HealthCheck::new("check1".to_string(), HealthStatus::Passing),
        HealthCheck::new("check2".to_string(), HealthStatus::Passing),
    ];
    let report = HealthReport::new(checks);
    assert_eq!(report.overall, HealthStatus::Passing);
    assert_eq!(report.checks.len(), 2);
}

#[test]
fn test_health_check_with_details() {
    let check = HealthCheck::new("test".to_string(), HealthStatus::Warning).with_details("warning details".to_string());
    assert_eq!(check.details, Some("warning details".to_string()));
}

#[test]
fn test_metric_labels() {
    let mut labels = MetricLabels::new();
    labels.insert("key".to_string(), "value".to_string());
    assert_eq!(labels.get("key"), Some(&"value".to_string()));
}

#[test]
fn test_metric_labels_default() {
    let labels = MetricLabels::default();
    assert!(labels.get("key").is_none());
}

fn main() {}
