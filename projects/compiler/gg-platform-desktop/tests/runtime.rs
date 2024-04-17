use gg_core::platform::{KeyCode, RuntimePlatform, WindowConfig, WindowManager};
use gg_platform_desktop::{
    DesktopWindowManager,
    runtime::{DesktopPlatformServices, DesktopRuntimePlatform},
};

#[test]
fn test_runtime_platform_id() {
    let platform = DesktopRuntimePlatform;
    assert_eq!(platform.id(), "desktop");
}

#[test]
fn test_runtime_platform_display_name() {
    let platform = DesktopRuntimePlatform;
    assert_eq!(platform.display_name(), "Desktop (Windows/macOS/Linux)");
}

#[test]
fn test_runtime_platform_create_services() {
    let platform = DesktopRuntimePlatform;
    let services = platform.create_services();
    assert_eq!(services.window.size(), (1280, 720));
}

#[test]
fn test_runtime_platform_services_have_all_components() {
    let platform = DesktopRuntimePlatform;
    let services = platform.create_services();
    assert_eq!(services.window.size(), (1280, 720));
    assert!(!services.input.is_pointer_down());
}

#[test]
fn test_runtime_platform_services_with_window_manager() {
    let mut wm = DesktopWindowManager::new();
    let _id = wm.create_window(WindowConfig::new("Test", 800, 600));
    assert_eq!(wm.window_count(), 1);

    let services = DesktopPlatformServices::create_with_window_manager(WindowConfig::default(), Box::new(wm));
    assert!(services.window_manager.is_some());
    assert_eq!(services.window.size(), (1280, 720));
}

#[test]
fn test_editor_startup_simulation() {
    let platform = DesktopRuntimePlatform;
    let services = platform.create_services();

    let (w, h) = services.window.size();
    assert!(w > 0);
    assert!(h > 0);

    assert!(!services.input.is_key_pressed(KeyCode::Escape));
    assert!(!services.input.is_pointer_down());
    assert_eq!(services.input.pointer_position(), (0.0, 0.0));
    assert!(services.input.connected_gamepads().is_empty());
}
