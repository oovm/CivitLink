use gg_platform_mobile::input::{MobileInputImpl, KeyCode, TouchPoint};
use gg_core::platform::{Input, InputEvent, KeyState, PointerAction, PointerButton};

#[test]
fn test_push_keyboard_event() {
    let mut input = MobileInputImpl::new();
    input.push_event(InputEvent::Keyboard { key: KeyCode::A, state: KeyState::Pressed });
    assert!(input.is_key_pressed(KeyCode::A));
    input.push_event(InputEvent::Keyboard { key: KeyCode::A, state: KeyState::Released });
    assert!(!input.is_key_pressed(KeyCode::A));
}

#[test]
fn test_push_pointer_event() {
    let mut input = MobileInputImpl::new();
    input.push_event(InputEvent::Pointer {
        position: (100.0, 200.0),
        action: PointerAction::Down,
        button: Some(PointerButton::Left),
    });
    assert!(input.is_pointer_down());
    assert_eq!(input.pointer_position(), (100.0, 200.0));
    input.push_event(InputEvent::Pointer {
        position: (150.0, 250.0),
        action: PointerAction::Up,
        button: Some(PointerButton::Left),
    });
    assert!(!input.is_pointer_down());
}

#[test]
fn test_poll_events_empties_buffer() {
    let mut input = MobileInputImpl::new();
    input.push_event(InputEvent::Keyboard { key: KeyCode::Space, state: KeyState::Pressed });
    let events = input.poll_events();
    assert_eq!(events.len(), 1);
    let events2 = input.poll_events();
    assert!(events2.is_empty());
}

#[test]
fn test_set_accelerometer() {
    let mut input = MobileInputImpl::new();
    assert!(input.accelerometer().is_none());
    input.set_accelerometer((1.0, 2.0, 3.0));
    assert_eq!(input.accelerometer(), Some((1.0, 2.0, 3.0)));
}

#[test]
fn test_set_gyroscope() {
    let mut input = MobileInputImpl::new();
    assert!(input.gyroscope().is_none());
    input.set_gyroscope((0.1, 0.2, 0.3));
    assert_eq!(input.gyroscope(), Some((0.1, 0.2, 0.3)));
}

#[test]
fn test_set_active_touches() {
    let mut input = MobileInputImpl::new();
    assert!(input.active_touches().is_empty());
    input.set_active_touches(vec![TouchPoint { id: 1, x: 100.0, y: 200.0, pressure: 1.0 }]);
    let touches = input.active_touches();
    assert_eq!(touches.len(), 1);
    assert_eq!(touches[0].id, 1);
}
