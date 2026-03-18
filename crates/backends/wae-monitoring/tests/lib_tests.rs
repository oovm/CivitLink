use wae_monitoring::{Alert, AlertConfig, AlertSeverity};

#[test]
fn test_alert_creation() {
    let alert = Alert::new("test_alert".to_string(), AlertSeverity::Warning);
    assert_eq!(alert.name, "test_alert");
}

#[test]
fn test_alert_config_default() {
    let config = AlertConfig::default();
    assert!(config.endpoint.is_empty());
}
