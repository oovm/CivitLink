use gg_core::platform::RuntimePlatform;
use gg_platform_web::runtime::WebRuntimePlatform;

#[test]
fn test_runtime_platform_id() {
    let platform = WebRuntimePlatform::new("/assets");
    assert_eq!(platform.id(), "web");
}

#[test]
fn test_runtime_platform_display_name() {
    let platform = WebRuntimePlatform::new("/assets");
    assert_eq!(platform.display_name(), "Web (WebAssembly)");
}

#[test]
fn test_runtime_platform_create_services() {
    let platform = WebRuntimePlatform::new("/assets");
    let services = platform.create_services();
    assert_eq!(services.window.size(), (1280, 720));
}
