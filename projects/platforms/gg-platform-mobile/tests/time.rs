use gg_platform_mobile::time::MobileTime;
use std::time::Duration;
use gg_core::platform::Time;

#[test]
fn test_initial_delta_is_zero() {
    let time = MobileTime::new();
    assert_eq!(time.delta(), Duration::ZERO);
}

#[test]
fn test_elapsed_increases() {
    let time = MobileTime::new();
    std::thread::sleep(Duration::from_millis(10));
    assert!(time.elapsed() > Duration::ZERO);
}

#[test]
fn test_update_advances_delta() {
    let mut time = MobileTime::new();
    std::thread::sleep(Duration::from_millis(10));
    time.update();
    assert!(time.delta() >= Duration::from_millis(5));
}
