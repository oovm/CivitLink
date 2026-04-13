use gg_core::platform::{WindowConfig, WindowId, WindowManager};
use gg_platform_desktop::{window::DesktopWindow, window_manager::DesktopWindowManager};

#[test]
fn test_create_window_manager() {
    let manager = DesktopWindowManager::new();
    assert_eq!(manager.window_count(), 0);
}

#[test]
fn test_create_and_destroy_window() {
    let mut manager = DesktopWindowManager::new();
    let id = manager.create_window(WindowConfig::default());
    assert_eq!(manager.window_count(), 1);
    assert!(manager.get_window(id).is_some());
    manager.destroy_window(id).unwrap();
    assert_eq!(manager.window_count(), 0);
    assert!(manager.get_window(id).is_none());
}

#[test]
fn test_destroy_nonexistent_window() {
    let mut manager = DesktopWindowManager::new();
    let result = manager.destroy_window(WindowId(999));
    assert!(result.is_err());
}

#[test]
fn test_with_main_window() {
    let main_window = DesktopWindow::new(WindowConfig::default());
    let manager = DesktopWindowManager::with_main_window(main_window);
    assert_eq!(manager.window_count(), 1);
}

#[test]
fn test_multiple_windows() {
    let mut manager = DesktopWindowManager::new();
    let id1 = manager.create_window(WindowConfig::new("Window 1", 800, 600));
    let id2 = manager.create_window(WindowConfig::new("Window 2", 1024, 768));
    assert_eq!(manager.window_count(), 2);
    assert_ne!(id1, id2);
    assert!(manager.get_window(id1).is_some());
    assert!(manager.get_window(id2).is_some());
}

#[test]
fn test_poll_events_empty() {
    let mut manager = DesktopWindowManager::new();
    let events = manager.poll_events();
    assert!(events.is_empty());
}

#[test]
fn test_window_event_routing() {
    let mut manager = DesktopWindowManager::new();
    let id1 = manager.create_window(WindowConfig::new("Window 1", 800, 600));
    let id2 = manager.create_window(WindowConfig::new("Window 2", 1024, 768));

    let events = manager.poll_events();
    assert!(events.is_empty());

    assert!(manager.get_window(id1).is_some());
    assert!(manager.get_window(id2).is_some());

    {
        let w1 = manager.get_window(id1).unwrap();
        assert_eq!(w1.size(), (800, 600));
    }
    {
        let w2 = manager.get_window(id2).unwrap();
        assert_eq!(w2.size(), (1024, 768));
    }
}
