use gg_core::platform::Input;
use gg_platform_desktop::input::{
    DesktopInput, GamepadAxis, GamepadButton, GamepadId, InputEvent, KeyCode, KeyState, PointerAction, PointerButton,
};

#[test]
fn test_push_keyboard_event() {
    let mut input = DesktopInput::new();
    input.push_event(InputEvent::Keyboard { key: KeyCode::A, state: KeyState::Pressed });
    assert!(input.is_key_pressed(KeyCode::A));
    input.push_event(InputEvent::Keyboard { key: KeyCode::A, state: KeyState::Released });
    assert!(!input.is_key_pressed(KeyCode::A));
}

#[test]
fn test_push_pointer_event() {
    let mut input = DesktopInput::new();
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
    assert_eq!(input.pointer_position(), (150.0, 250.0));
}

#[test]
fn test_poll_events_returns_buffered() {
    let mut input = DesktopInput::new();
    input.push_event(InputEvent::Keyboard { key: KeyCode::Space, state: KeyState::Pressed });
    input.push_event(InputEvent::Keyboard { key: KeyCode::Enter, state: KeyState::Pressed });
    let events = input.poll_events();
    assert_eq!(events.len(), 2);
    let events2 = input.poll_events();
    assert!(events2.is_empty());
}

#[test]
fn test_winit_keycode_mapping() {
    assert_eq!(
        DesktopInput::winit_key_to_keycode(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyA)),
        Some(KeyCode::A)
    );
    assert_eq!(
        DesktopInput::winit_key_to_keycode(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space)),
        Some(KeyCode::Space)
    );
    assert_eq!(
        DesktopInput::winit_key_to_keycode(winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp)),
        Some(KeyCode::Up)
    );
}

#[test]
fn test_winit_mouse_button_mapping() {
    assert_eq!(DesktopInput::winit_mouse_button_to_pointer(winit::event::MouseButton::Left), PointerButton::Left);
    assert_eq!(DesktopInput::winit_mouse_button_to_pointer(winit::event::MouseButton::Right), PointerButton::Right);
    assert_eq!(DesktopInput::winit_mouse_button_to_pointer(winit::event::MouseButton::Middle), PointerButton::Middle);
}

#[test]
fn test_gilrs_button_mapping() {
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::South), Some(GamepadButton::South));
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::East), Some(GamepadButton::East));
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::West), Some(GamepadButton::West));
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::North), Some(GamepadButton::North));
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::Start), Some(GamepadButton::Start));
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::Select), Some(GamepadButton::Back));
    assert_eq!(DesktopInput::gilrs_button_to_gamepad(gilrs::Button::DPadUp), Some(GamepadButton::DPadUp));
}

#[test]
fn test_gilrs_axis_mapping() {
    assert_eq!(DesktopInput::gilrs_axis_to_gamepad(gilrs::Axis::LeftStickX), Some(GamepadAxis::LeftStickX));
    assert_eq!(DesktopInput::gilrs_axis_to_gamepad(gilrs::Axis::LeftStickY), Some(GamepadAxis::LeftStickY));
    assert_eq!(DesktopInput::gilrs_axis_to_gamepad(gilrs::Axis::RightStickX), Some(GamepadAxis::RightStickX));
    assert_eq!(DesktopInput::gilrs_axis_to_gamepad(gilrs::Axis::RightStickY), Some(GamepadAxis::RightStickY));
    assert_eq!(DesktopInput::gilrs_axis_to_gamepad(gilrs::Axis::LeftZ), Some(GamepadAxis::LeftTrigger));
    assert_eq!(DesktopInput::gilrs_axis_to_gamepad(gilrs::Axis::RightZ), Some(GamepadAxis::RightTrigger));
}

#[test]
fn test_gamepad_button_state_tracking() {
    let mut input = DesktopInput::new();
    let gp_id = GamepadId(0);
    input.push_event(InputEvent::GamepadButton { id: gp_id, button: GamepadButton::South, state: KeyState::Pressed });
    assert!(input.is_gamepad_button_pressed(gp_id, GamepadButton::South));
}
