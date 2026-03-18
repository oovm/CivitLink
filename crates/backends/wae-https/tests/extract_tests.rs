use wae_https::extract::*;

#[test]
fn test_extractor_error_display() {
    let err = ExtractorError::Custom("test error".to_string());
    assert_eq!(format!("{}", err), "Extractor error: test error");
}
