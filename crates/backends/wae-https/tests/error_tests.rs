use hyper::StatusCode;
use wae_https::error::*;
use wae_types::ErrorCategory;

#[test]
fn test_http_error_category() {
    let error = HttpError::invalid_params("test", "test reason");
    assert_eq!(error.category(), ErrorCategory::Validation);

    let error = HttpError::invalid_token("test reason");
    assert_eq!(error.category(), ErrorCategory::Auth);

    let error = HttpError::not_found("resource", "id");
    assert_eq!(error.category(), ErrorCategory::NotFound);

    let error = HttpError::internal("test reason");
    assert_eq!(error.category(), ErrorCategory::Internal);
}

#[test]
fn test_error_response() {
    let error = HttpError::not_found("资源", "123");
    let response = ErrorResponse::from_error(&error);

    assert!(!response.success);
    assert_eq!(response.code, "wae.error.not_found.resource");
    assert!(response.details.is_some());
    assert!(response.trace_id.is_none());
}

#[test]
fn test_error_ext() {
    let result: Result<(), &str> = Err("test error");
    let http_result = result.bad_request();

    match http_result {
        Err(e) => assert_eq!(e.category(), ErrorCategory::Validation),
        _ => panic!("Expected Validation error"),
    }
}

#[test]
fn test_from_wae_error() {
    let wae_error = wae_types::WaeError::not_found("资源", "123");
    let http_error: HttpError = wae_error.into();

    assert_eq!(http_error.category(), ErrorCategory::NotFound);
}

#[test]
fn test_http_error_from_wae_error() {
    let wae_error = wae_types::WaeError::invalid_token("test");
    let http_error = HttpError::new(wae_error);

    assert_eq!(http_error.category(), ErrorCategory::Auth);
    assert_eq!(http_error.i18n_key(), "wae.error.auth.invalid_token");
}
