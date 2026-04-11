use gg_platform_web::render::{detect_best_backend, is_webgpu_available, is_webgl2_available};

#[test]
fn test_detect_best_backend_non_wasm() {
    let result = detect_best_backend();
    assert!(result.is_none());
}

#[test]
fn test_is_webgpu_available_non_wasm() {
    assert!(!is_webgpu_available());
}

#[test]
fn test_is_webgl2_available_non_wasm() {
    assert!(!is_webgl2_available());
}
