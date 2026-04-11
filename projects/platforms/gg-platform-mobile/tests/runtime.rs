use gg_platform_mobile::runtime::{MobileRuntimePlatform, MobilePlatformServices};
use gg_core::platform::RuntimePlatform;

#[test]
fn test_runtime_platform_id() {
    let platform = MobileRuntimePlatform;
    assert_eq!(platform.id(), "mobile");
}

#[test]
fn test_runtime_platform_display_name() {
    let platform = MobileRuntimePlatform;
    assert_eq!(platform.display_name(), "Mobile (iOS/Android)");
}

#[test]
fn test_runtime_platform_create_services() {
    let platform = MobileRuntimePlatform;
    let services = platform.create_services();
    assert_eq!(services.window.size(), (1280, 720));
}
