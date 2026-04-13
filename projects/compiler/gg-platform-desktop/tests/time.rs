use gg_core::platform::Time;
use gg_platform_desktop::time::DesktopTime;
use std::time::Duration;

#[test]
fn test_initial_delta_is_zero() {
    let time = DesktopTime::new();
    assert_eq!(time.delta(), Duration::ZERO);
}

#[test]
fn test_elapsed_increases() {
    let time = DesktopTime::new();
    std::thread::sleep(Duration::from_millis(10));
    assert!(time.elapsed() > Duration::ZERO);
}

#[test]
fn test_update_advances_delta() {
    let mut time = DesktopTime::new();
    std::thread::sleep(Duration::from_millis(10));
    time.update();
    assert!(time.delta() >= Duration::from_millis(10));
}
