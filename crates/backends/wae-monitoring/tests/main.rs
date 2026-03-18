use wae_monitoring::*;

#[test]
fn test_alert_creation() {
    let alert = Alert::new("test_alert".to_string(), AlertSeverity::Critical);
    assert_eq!(alert.name, "test_alert");
    assert_eq!(alert.severity, AlertSeverity::Critical);
    assert_eq!(alert.status, AlertStatus::Firing);
}

#[test]
fn test_alert_with_label() {
    let alert = Alert::new("test_alert".to_string(), AlertSeverity::Warning).with_label("service", "api");
    assert_eq!(alert.labels.get("service"), Some(&"api".to_string()));
}

#[test]
fn test_alert_with_annotation() {
    let alert =
        Alert::new("test_alert".to_string(), AlertSeverity::Error).with_annotation("description", "Something went wrong");
    assert_eq!(alert.annotations.get("description"), Some(&"Something went wrong".to_string()));
}

#[test]
fn test_alert_config_default() {
    let config = AlertConfig::default();
    assert_eq!(config.endpoint, "");
    assert_eq!(config.timeout, std::time::Duration::from_secs(30));
    assert!(config.headers.is_empty());
}

#[test]
fn test_alert_severity_default() {
    let severity = AlertSeverity::default();
    assert_eq!(severity, AlertSeverity::Warning);
}
