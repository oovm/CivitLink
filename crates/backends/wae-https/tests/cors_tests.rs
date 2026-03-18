use wae_https::middleware::{CorsConfig, cors_permissive, cors_strict};

#[test]
fn test_cors_config_default() {
    let config = CorsConfig::new();
    assert!(config.allowed_origins.is_empty());
    assert!(config.allowed_methods.is_empty());
    assert!(config.allowed_headers.is_empty());
    assert!(!config.allow_credentials);
    assert_eq!(config.max_age, 600);
}

#[test]
fn test_cors_config_builders() {
    let config = CorsConfig::new()
        .allow_origin("https://example.com")
        .allow_method("GET")
        .allow_header("Content-Type")
        .allow_credentials(true)
        .max_age(3600);

    assert_eq!(config.allowed_origins.len(), 1);
    assert_eq!(config.allowed_methods.len(), 1);
    assert_eq!(config.allowed_headers.len(), 1);
    assert!(config.allow_credentials);
    assert_eq!(config.max_age, 3600);
}

#[test]
fn test_cors_config_into_layer() {
    let config = CorsConfig::new()
        .allow_origins(["https://example.com", "https://api.example.com"])
        .allow_methods(["GET", "POST"])
        .allow_headers(["Content-Type", "Authorization"]);

    let _layer = config.into_layer();
}
