use gg_platform_mobile::{lifecycle_manager::MobileLifecycleManager, lifecycle::{LifecycleState, MobileLifecycle}};

#[test]
fn test_initial_state_is_running() {
    let manager = MobileLifecycleManager::new();
    assert!(manager.is_running());
    assert!(!manager.is_paused());
    assert!(!manager.is_destroyed());
}

#[test]
fn test_on_pause() {
    let mut manager = MobileLifecycleManager::new();
    manager.on_pause().unwrap();
    assert!(!manager.is_running());
    assert!(manager.is_paused());
    assert!(!manager.is_destroyed());
    assert_eq!(manager.state(), LifecycleState::Paused);
}

#[test]
fn test_on_resume() {
    let mut manager = MobileLifecycleManager::new();
    manager.on_pause().unwrap();
    assert!(manager.is_paused());
    manager.on_resume().unwrap();
    assert!(manager.is_running());
    assert!(!manager.is_paused());
}

#[test]
fn test_on_destroy() {
    let mut manager = MobileLifecycleManager::new();
    manager.on_destroy().unwrap();
    assert!(manager.is_destroyed());
    assert!(!manager.is_running());
    assert_eq!(manager.state(), LifecycleState::Destroyed);
}

#[test]
fn test_lifecycle_transitions() {
    let mut manager = MobileLifecycleManager::new();
    assert!(manager.is_running());
    manager.on_pause().unwrap();
    assert!(manager.is_paused());
    manager.on_resume().unwrap();
    assert!(manager.is_running());
    manager.on_destroy().unwrap();
    assert!(manager.is_destroyed());
}
