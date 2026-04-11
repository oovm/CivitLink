use gg_platform_web::input::{WebInput, KeyCode, PointerButton, PointerAction};
use gg_core::platform::Input;

#[test]
fn test_parse_key_code_letters() {
    assert_eq!(WebInput::parse_key_code("KeyA"), Some(KeyCode::A));
    assert_eq!(WebInput::parse_key_code("KeyZ"), Some(KeyCode::Z));
    assert_eq!(WebInput::parse_key_code("Unknown"), None);
}

#[test]
fn test_parse_key_code_digits() {
    assert_eq!(WebInput::parse_key_code("Digit0"), Some(KeyCode::Key0));
    assert_eq!(WebInput::parse_key_code("Digit9"), Some(KeyCode::Key9));
}

#[test]
fn test_parse_key_code_special() {
    assert_eq!(WebInput::parse_key_code("Space"), Some(KeyCode::Space));
    assert_eq!(WebInput::parse_key_code("Enter"), Some(KeyCode::Enter));
    assert_eq!(WebInput::parse_key_code("Escape"), Some(KeyCode::Escape));
    assert_eq!(WebInput::parse_key_code("ArrowUp"), Some(KeyCode::Up));
    assert_eq!(WebInput::parse_key_code("ShiftLeft"), Some(KeyCode::Shift));
    assert_eq!(WebInput::parse_key_code("ShiftRight"), Some(KeyCode::Shift));
}

#[test]
fn test_parse_mouse_button() {
    assert_eq!(WebInput::parse_mouse_button(0), PointerButton::Left);
    assert_eq!(WebInput::parse_mouse_button(1), PointerButton::Middle);
    assert_eq!(WebInput::parse_mouse_button(2), PointerButton::Right);
    assert_eq!(WebInput::parse_mouse_button(3), PointerButton::Other(3));
}

#[test]
fn test_parse_pointer_action() {
    assert_eq!(WebInput::parse_pointer_action("mousedown", 0.0), PointerAction::Down);
    assert_eq!(WebInput::parse_pointer_action("mouseup", 0.0), PointerAction::Up);
    assert_eq!(WebInput::parse_pointer_action("mousemove", 0.0), PointerAction::Move);
    assert_eq!(WebInput::parse_pointer_action("wheel", 3.0), PointerAction::Scroll(3.0));
    assert_eq!(WebInput::parse_pointer_action("touchstart", 0.0), PointerAction::Down);
    assert_eq!(WebInput::parse_pointer_action("touchend", 0.0), PointerAction::Up);
    assert_eq!(WebInput::parse_pointer_action("touchmove", 0.0), PointerAction::Move);
}

#[test]
fn test_push_keyboard_event() {
    let mut input = WebInput::new();
    input.push_keyboard_event("KeyA", true);
    assert!(input.is_key_pressed(KeyCode::A));
    input.push_keyboard_event("KeyA", false);
    assert!(!input.is_key_pressed(KeyCode::A));
}

#[test]
fn test_push_mouse_event() {
    let mut input = WebInput::new();
    input.push_mouse_event(100.0, 200.0, PointerAction::Down, PointerButton::Left);
    assert!(input.is_pointer_down());
    assert_eq!(input.pointer_position(), (100.0, 200.0));
    input.push_mouse_event(150.0, 250.0, PointerAction::Up, PointerButton::Left);
    assert!(!input.is_pointer_down());
}

#[test]
fn test_push_raw_mouse_event() {
    let mut input = WebInput::new();
    input.push_raw_mouse_event(100.0, 200.0, "mousedown", 0, 0.0);
    assert!(input.is_pointer_down());
    let events = input.poll_events();
    assert_eq!(events.len(), 1);
}

#[test]
fn test_push_raw_touch_event() {
    let mut input = WebInput::new();
    input.push_raw_touch_event(50.0, 75.0, "touchstart");
    assert!(input.is_pointer_down());
    input.push_raw_touch_event(50.0, 75.0, "touchend");
    assert!(!input.is_pointer_down());
}

#[test]
fn test_poll_events_empties_buffer() {
    let mut input = WebInput::new();
    input.push_keyboard_event("Space", true);
    input.push_keyboard_event("Enter", true);
    let events = input.poll_events();
    assert_eq!(events.len(), 2);
    let events2 = input.poll_events();
    assert!(events2.is_empty());
}
