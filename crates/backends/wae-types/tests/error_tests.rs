use serde_json::json;
use wae_types::*;

#[test]
fn test_error_category_http_status() {
    assert_eq!(ErrorCategory::Validation.http_status(), 400);
    assert_eq!(ErrorCategory::Auth.http_status(), 401);
    assert_eq!(ErrorCategory::Permission.http_status(), 403);
    assert_eq!(ErrorCategory::NotFound.http_status(), 404);
    assert_eq!(ErrorCategory::Conflict.http_status(), 409);
    assert_eq!(ErrorCategory::RateLimited.http_status(), 429);
    assert_eq!(ErrorCategory::Network.http_status(), 502);
    assert_eq!(ErrorCategory::Storage.http_status(), 500);
    assert_eq!(ErrorCategory::Database.http_status(), 500);
    assert_eq!(ErrorCategory::Cache.http_status(), 500);
    assert_eq!(ErrorCategory::Config.http_status(), 500);
    assert_eq!(ErrorCategory::Timeout.http_status(), 408);
    assert_eq!(ErrorCategory::Internal.http_status(), 500);
}

#[test]
fn test_wae_error_kind_category() {
    let validation_error = WaeErrorKind::InvalidFormat { field: "email".to_string(), expected: "email format".to_string() };
    assert_eq!(validation_error.category(), ErrorCategory::Validation);

    let auth_error = WaeErrorKind::InvalidCredentials;
    assert_eq!(auth_error.category(), ErrorCategory::Auth);

    let permission_error = WaeErrorKind::PermissionDenied { action: "delete".to_string() };
    assert_eq!(permission_error.category(), ErrorCategory::Permission);

    let not_found_error = WaeErrorKind::ResourceNotFound { resource_type: "user".to_string(), identifier: "123".to_string() };
    assert_eq!(not_found_error.category(), ErrorCategory::NotFound);

    let conflict_error = WaeErrorKind::ResourceConflict { resource: "file".to_string(), reason: "already exists".to_string() };
    assert_eq!(conflict_error.category(), ErrorCategory::Conflict);

    let network_error = WaeErrorKind::ConnectionFailed { target: "api.example.com".to_string() };
    assert_eq!(network_error.category(), ErrorCategory::Network);
}

#[test]
fn test_wae_error_kind_i18n_key() {
    let error = WaeErrorKind::InvalidFormat { field: "email".to_string(), expected: "email format".to_string() };
    assert_eq!(error.i18n_key(), "wae.error.validation.invalid_format");

    let error = WaeErrorKind::InvalidCredentials;
    assert_eq!(error.i18n_key(), "wae.error.auth.invalid_credentials");

    let error = WaeErrorKind::PermissionDenied { action: "delete".to_string() };
    assert_eq!(error.i18n_key(), "wae.error.permission.denied");
}

#[test]
fn test_wae_error_kind_i18n_data() {
    let error = WaeErrorKind::InvalidFormat { field: "email".to_string(), expected: "email format".to_string() };
    assert_eq!(error.i18n_data(), json!({ "field": "email", "expected": "email format" }));

    let error = WaeErrorKind::UserNotFound { identifier: "test@example.com".to_string() };
    assert_eq!(error.i18n_data(), json!({ "identifier": "test@example.com" }));

    let error = WaeErrorKind::InvalidCredentials;
    assert_eq!(error.i18n_data(), json!({}));
}

#[test]
fn test_wae_error_new() {
    let kind = WaeErrorKind::InvalidFormat { field: "email".to_string(), expected: "email format".to_string() };
    let error = WaeError::new(kind);
    assert_eq!(error.i18n_key(), "wae.error.validation.invalid_format");
}

#[test]
fn test_wae_error_convenience_methods() {
    let error = WaeError::invalid_format("email", "email format");
    assert_eq!(error.category(), ErrorCategory::Validation);
    assert_eq!(error.i18n_key(), "wae.error.validation.invalid_format");

    let error = WaeError::invalid_credentials();
    assert_eq!(error.category(), ErrorCategory::Auth);
    assert_eq!(error.i18n_key(), "wae.error.auth.invalid_credentials");

    let error = WaeError::permission_denied("delete");
    assert_eq!(error.category(), ErrorCategory::Permission);
    assert_eq!(error.i18n_key(), "wae.error.permission.denied");

    let error = WaeError::not_found("user", "123");
    assert_eq!(error.category(), ErrorCategory::NotFound);
    assert_eq!(error.i18n_key(), "wae.error.not_found.resource");

    let error = WaeError::internal("something went wrong");
    assert_eq!(error.category(), ErrorCategory::Internal);
    assert_eq!(error.i18n_key(), "wae.error.internal.error");
}

#[test]
fn test_wae_error_http_status() {
    let error = WaeError::invalid_format("email", "email format");
    assert_eq!(error.http_status(), 400);

    let error = WaeError::invalid_credentials();
    assert_eq!(error.http_status(), 401);

    let error = WaeError::not_found("user", "123");
    assert_eq!(error.http_status(), 404);

    let error = WaeError::internal("something went wrong");
    assert_eq!(error.http_status(), 500);
}

#[test]
fn test_from_std_io_error() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let error: WaeError = io_error.into();
    assert_eq!(error.category(), ErrorCategory::Internal);
    assert_eq!(error.i18n_key(), "wae.error.internal.io_error");
}

#[test]
fn test_from_serde_json_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let error: WaeError = json_error.into();
    assert_eq!(error.category(), ErrorCategory::Internal);
    assert_eq!(error.i18n_key(), "wae.error.internal.json_error");
}

#[test]
fn test_wae_error_display() {
    let error = WaeError::invalid_format("email", "email format");
    let display_str = format!("{}", error);
    assert!(display_str.contains("Validation"));
    assert!(display_str.contains("wae.error.validation.invalid_format"));
}

#[test]
fn test_wae_error_kind_display() {
    let kind = WaeErrorKind::InvalidFormat { field: "email".to_string(), expected: "email format".to_string() };
    let display_str = format!("{}", kind);
    assert_eq!(display_str, "wae.error.validation.invalid_format");
}
