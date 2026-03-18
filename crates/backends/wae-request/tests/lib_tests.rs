use wae_request::*;

#[test]
fn test_config_default() {
    let config = RequestConfig::default();
    assert_eq!(config.timeout_secs, 30);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn test_client_creation() {
    let result = RequestClient::with_defaults();
    assert!(result.is_ok());
}
