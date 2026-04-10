use std::collections::HashSet;

use gg_core::platform::{Input, InputEvent, KeyCode, KeyState, PointerAction, PointerButton};

/// 桌面平台输入实现
///
/// 维护内部按键状态映射和指针状态，通过 `push_event` 接收来自窗口系统的事件，
/// 同时提供 `push_winit_event` 方法直接接收 winit 事件。
pub struct DesktopInput {
    /// 待处理的事件缓冲
    event_buffer: Vec<InputEvent>,
    /// 当前按下的按键集合
    pressed_keys: HashSet<KeyCode>,
    /// 指针是否按下
    pointer_down: bool,
    /// 指针当前位置
    pointer_position: (f32, f32),
    /// 指针按下的按钮
    pointer_button: Option<PointerButton>,
}

impl DesktopInput {
    /// 创建新的桌面输入实例
    pub fn new() -> Self {
        Self {
            event_buffer: Vec::new(),
            pressed_keys: HashSet::new(),
            pointer_down: false,
            pointer_position: (0.0, 0.0),
            pointer_button: None,
        }
    }

    /// 推送一个输入事件到缓冲区
    ///
    /// 由窗口系统在接收到事件时调用。
    pub fn push_event(&mut self, event: InputEvent) {
        match &event {
            InputEvent::Keyboard { key, state } => match state {
                KeyState::Pressed => {
                    self.pressed_keys.insert(*key);
                }
                KeyState::Released => {
                    self.pressed_keys.remove(key);
                }
            },
            InputEvent::Pointer { position, action, button } => {
                self.pointer_position = *position;
                match action {
                    PointerAction::Down => {
                        self.pointer_down = true;
                        self.pointer_button = *button;
                    }
                    PointerAction::Up => {
                        self.pointer_down = false;
                        self.pointer_button = None;
                    }
                    PointerAction::Move => {}
                    PointerAction::Scroll(_) => {}
                }
            }
            InputEvent::Gamepad { .. } => {}
        }
        self.event_buffer.push(event);
    }

    /// 推送 winit 键盘事件到缓冲区
    ///
    /// 将 winit 的 `KeyEvent` 转换为引擎的 `InputEvent::Keyboard` 事件。
    pub fn push_winit_keyboard_event(&mut self, event: &winit::event::KeyEvent) {
        if let Some(key) = Self::winit_key_to_keycode(event.physical_key) {
            let state = match event.state {
                winit::event::ElementState::Pressed => KeyState::Pressed,
                winit::event::ElementState::Released => KeyState::Released,
            };
            self.push_event(InputEvent::Keyboard { key, state });
        }
    }

    /// 推送 winit 鼠标输入事件到缓冲区
    ///
    /// 将 winit 的鼠标按钮事件转换为引擎的 `InputEvent::Pointer` 事件。
    pub fn push_winit_mouse_input(&mut self, state: winit::event::ElementState, button: winit::event::MouseButton) {
        let pointer_button = Self::winit_mouse_button_to_pointer(button);
        let action = match state {
            winit::event::ElementState::Pressed => PointerAction::Down,
            winit::event::ElementState::Released => PointerAction::Up,
        };
        self.push_event(InputEvent::Pointer {
            position: self.pointer_position,
            action,
            button: Some(pointer_button),
        });
    }

    /// 推送 winit 鼠标移动事件到缓冲区
    ///
    /// 将 winit 的鼠标位置更新转换为引擎的 `InputEvent::Pointer` 事件。
    pub fn push_winit_cursor_moved(&mut self, position: (f32, f32)) {
        self.pointer_position = position;
        self.push_event(InputEvent::Pointer {
            position,
            action: PointerAction::Move,
            button: self.pointer_button,
        });
    }

    /// 推送 winit 滚轮事件到缓冲区
    pub fn push_winit_scroll(&mut self, delta: f32) {
        self.push_event(InputEvent::Pointer {
            position: self.pointer_position,
            action: PointerAction::Scroll(delta),
            button: None,
        });
    }

    /// 将 winit PhysicalKey 转换为引擎 KeyCode
    fn winit_key_to_keycode(key: winit::keyboard::PhysicalKey) -> Option<KeyCode> {
        match key {
            winit::keyboard::PhysicalKey::Code(code) => Self::winit_keycode_to_keycode(code),
            _ => None,
        }
    }

    /// 将 winit KeyCode 枚举映射为引擎 KeyCode 枚举
    fn winit_keycode_to_keycode(code: winit::keyboard::KeyCode) -> Option<KeyCode> {
        use winit::keyboard::KeyCode as Wk;
        match code {
            Wk::KeyA => Some(KeyCode::A),
            Wk::KeyB => Some(KeyCode::B),
            Wk::KeyC => Some(KeyCode::C),
            Wk::KeyD => Some(KeyCode::D),
            Wk::KeyE => Some(KeyCode::E),
            Wk::KeyF => Some(KeyCode::F),
            Wk::KeyG => Some(KeyCode::G),
            Wk::KeyH => Some(KeyCode::H),
            Wk::KeyI => Some(KeyCode::I),
            Wk::KeyJ => Some(KeyCode::J),
            Wk::KeyK => Some(KeyCode::K),
            Wk::KeyL => Some(KeyCode::L),
            Wk::KeyM => Some(KeyCode::M),
            Wk::KeyN => Some(KeyCode::N),
            Wk::KeyO => Some(KeyCode::O),
            Wk::KeyP => Some(KeyCode::P),
            Wk::KeyQ => Some(KeyCode::Q),
            Wk::KeyR => Some(KeyCode::R),
            Wk::KeyS => Some(KeyCode::S),
            Wk::KeyT => Some(KeyCode::T),
            Wk::KeyU => Some(KeyCode::U),
            Wk::KeyV => Some(KeyCode::V),
            Wk::KeyW => Some(KeyCode::W),
            Wk::KeyX => Some(KeyCode::X),
            Wk::KeyY => Some(KeyCode::Y),
            Wk::KeyZ => Some(KeyCode::Z),
            Wk::Digit0 => Some(KeyCode::Key0),
            Wk::Digit1 => Some(KeyCode::Key1),
            Wk::Digit2 => Some(KeyCode::Key2),
            Wk::Digit3 => Some(KeyCode::Key3),
            Wk::Digit4 => Some(KeyCode::Key4),
            Wk::Digit5 => Some(KeyCode::Key5),
            Wk::Digit6 => Some(KeyCode::Key6),
            Wk::Digit7 => Some(KeyCode::Key7),
            Wk::Digit8 => Some(KeyCode::Key8),
            Wk::Digit9 => Some(KeyCode::Key9),
            Wk::Space => Some(KeyCode::Space),
            Wk::Enter => Some(KeyCode::Enter),
            Wk::Escape => Some(KeyCode::Escape),
            Wk::Tab => Some(KeyCode::Tab),
            Wk::ShiftLeft | Wk::ShiftRight => Some(KeyCode::Shift),
            Wk::ControlLeft | Wk::ControlRight => Some(KeyCode::Control),
            Wk::AltLeft | Wk::AltRight => Some(KeyCode::Alt),
            Wk::ArrowUp => Some(KeyCode::Up),
            Wk::ArrowDown => Some(KeyCode::Down),
            Wk::ArrowLeft => Some(KeyCode::Left),
            Wk::ArrowRight => Some(KeyCode::Right),
            Wk::F1 => Some(KeyCode::F1),
            Wk::F2 => Some(KeyCode::F2),
            Wk::F3 => Some(KeyCode::F3),
            Wk::F4 => Some(KeyCode::F4),
            Wk::F5 => Some(KeyCode::F5),
            Wk::F6 => Some(KeyCode::F6),
            Wk::F7 => Some(KeyCode::F7),
            Wk::F8 => Some(KeyCode::F8),
            Wk::F9 => Some(KeyCode::F9),
            Wk::F10 => Some(KeyCode::F10),
            Wk::F11 => Some(KeyCode::F11),
            Wk::F12 => Some(KeyCode::F12),
            Wk::Home => Some(KeyCode::Home),
            Wk::End => Some(KeyCode::End),
            Wk::PageUp => Some(KeyCode::PageUp),
            Wk::PageDown => Some(KeyCode::PageDown),
            Wk::Insert => Some(KeyCode::Insert),
            Wk::Delete => Some(KeyCode::Delete),
            Wk::Backspace => Some(KeyCode::Backspace),
            _ => None,
        }
    }

    /// 将 winit MouseButton 转换为引擎 PointerButton
    fn winit_mouse_button_to_pointer(button: winit::event::MouseButton) -> PointerButton {
        match button {
            winit::event::MouseButton::Left => PointerButton::Left,
            winit::event::MouseButton::Right => PointerButton::Right,
            winit::event::MouseButton::Middle => PointerButton::Middle,
            winit::event::MouseButton::Back => PointerButton::Other(1),
            winit::event::MouseButton::Forward => PointerButton::Other(2),
            winit::event::MouseButton::Other(n) => PointerButton::Other(n as u8),
        }
    }
}

impl Input for DesktopInput {
    fn poll_events(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.event_buffer)
    }

    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    fn is_pointer_down(&self) -> bool {
        self.pointer_down
    }

    fn pointer_position(&self) -> (f32, f32) {
        self.pointer_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_core::platform::Input;

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
}
