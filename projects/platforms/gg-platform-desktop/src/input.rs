use std::collections::{HashMap, HashSet};

use gg_core::platform::{
    GamepadAxis, GamepadButton, GamepadId, Input, InputEvent, KeyCode, KeyState, PointerAction,
    PointerButton,
};
use gilrs::Gilrs;

/// 桌面平台输入实现
///
/// 维护内部按键状态映射和指针状态，通过 `push_event` 接收来自窗口系统的事件，
/// 同时提供 `push_winit_event` 方法直接接收 winit 事件。
/// 通过 gilrs 库提供手柄输入支持。
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
    /// gilrs 实例，用于手柄轮询
    gilrs: Option<Gilrs>,
    /// 每个手柄当前按下的按钮集合
    gamepad_buttons: HashMap<GamepadId, HashSet<GamepadButton>>,
    /// 每个手柄的轴值映射
    gamepad_axes: HashMap<GamepadId, HashMap<GamepadAxis, f32>>,
    /// 当前连接的手柄标识符列表
    connected_gamepads: Vec<GamepadId>,
}

impl DesktopInput {
    /// 创建新的桌面输入实例
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("警告: 无法初始化 gilrs: {e}，手柄输入将不可用");
                None
            }
        };
        Self {
            event_buffer: Vec::new(),
            pressed_keys: HashSet::new(),
            pointer_down: false,
            pointer_position: (0.0, 0.0),
            pointer_button: None,
            gilrs,
            gamepad_buttons: HashMap::new(),
            gamepad_axes: HashMap::new(),
            connected_gamepads: Vec::new(),
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
            InputEvent::GamepadButton { id, button, state } => match state {
                KeyState::Pressed => {
                    self.gamepad_buttons.entry(*id).or_default().insert(*button);
                }
                KeyState::Released => {
                    self.gamepad_buttons.entry(*id).or_default().remove(button);
                }
            },
            InputEvent::GamepadAxis { id, axis, value } => {
                self.gamepad_axes.entry(*id).or_default().insert(*axis, *value);
            }
            InputEvent::GamepadConnected { id, .. } => {
                if !self.connected_gamepads.contains(id) {
                    self.connected_gamepads.push(*id);
                }
            }
            InputEvent::GamepadDisconnected { id } => {
                self.connected_gamepads.retain(|gid| gid != id);
                self.gamepad_buttons.remove(id);
                self.gamepad_axes.remove(id);
            }
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

    /// 轮询 gilrs 事件并转换为引擎输入事件
    ///
    /// 从 gilrs 库获取手柄事件，更新内部状态并返回转换后的事件列表。
    fn poll_gilrs_events(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();
        let Some(gilrs) = self.gilrs.as_mut() else {
            return events;
        };
        while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
            let gamepad_id = GamepadId(id as u32);
            match event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(gp_button) = Self::gilrs_button_to_gamepad(button) {
                        self.gamepad_buttons.entry(gamepad_id).or_default().insert(gp_button);
                        events.push(InputEvent::GamepadButton {
                            id: gamepad_id,
                            button: gp_button,
                            state: KeyState::Pressed,
                        });
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(gp_button) = Self::gilrs_button_to_gamepad(button) {
                        self.gamepad_buttons.entry(gamepad_id).or_default().remove(&gp_button);
                        events.push(InputEvent::GamepadButton {
                            id: gamepad_id,
                            button: gp_button,
                            state: KeyState::Released,
                        });
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    if let Some(gp_axis) = Self::gilrs_axis_to_gamepad(axis) {
                        self.gamepad_axes.entry(gamepad_id).or_default().insert(gp_axis, value);
                        events.push(InputEvent::GamepadAxis {
                            id: gamepad_id,
                            axis: gp_axis,
                            value,
                        });
                    }
                }
                gilrs::EventType::Connected => {
                    if !self.connected_gamepads.contains(&gamepad_id) {
                        self.connected_gamepads.push(gamepad_id);
                    }
                    let name = gilrs.gamepad(id).name().to_string();
                    events.push(InputEvent::GamepadConnected {
                        id: gamepad_id,
                        name,
                    });
                }
                gilrs::EventType::Disconnected => {
                    self.connected_gamepads.retain(|gid| gid != &gamepad_id);
                    self.gamepad_buttons.remove(&gamepad_id);
                    self.gamepad_axes.remove(&gamepad_id);
                    events.push(InputEvent::GamepadDisconnected {
                        id: gamepad_id,
                    });
                }
                _ => {}
            }
        }
        events
    }

    /// 将 gilrs 按钮映射为引擎手柄按钮
    fn gilrs_button_to_gamepad(btn: gilrs::Button) -> Option<GamepadButton> {
        match btn {
            gilrs::Button::South => Some(GamepadButton::South),
            gilrs::Button::East => Some(GamepadButton::East),
            gilrs::Button::West => Some(GamepadButton::West),
            gilrs::Button::North => Some(GamepadButton::North),
            gilrs::Button::Start => Some(GamepadButton::Start),
            gilrs::Button::Select => Some(GamepadButton::Back),
            gilrs::Button::Mode => Some(GamepadButton::Guide),
            gilrs::Button::LeftThumb => Some(GamepadButton::LeftThumb),
            gilrs::Button::RightThumb => Some(GamepadButton::RightThumb),
            gilrs::Button::LeftTrigger => Some(GamepadButton::LeftShoulder),
            gilrs::Button::RightTrigger => Some(GamepadButton::RightShoulder),
            gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
            gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
            gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
            gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
            _ => None,
        }
    }

    /// 将 gilrs 轴映射为引擎手柄轴
    fn gilrs_axis_to_gamepad(axis: gilrs::Axis) -> Option<GamepadAxis> {
        match axis {
            gilrs::Axis::LeftStickX => Some(GamepadAxis::LeftStickX),
            gilrs::Axis::LeftStickY => Some(GamepadAxis::LeftStickY),
            gilrs::Axis::RightStickX => Some(GamepadAxis::RightStickX),
            gilrs::Axis::RightStickY => Some(GamepadAxis::RightStickY),
            gilrs::Axis::LeftZ => Some(GamepadAxis::LeftTrigger),
            gilrs::Axis::RightZ => Some(GamepadAxis::RightTrigger),
            _ => None,
        }
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
        let mut gamepad_events = self.poll_gilrs_events();
        let mut other_events = std::mem::take(&mut self.event_buffer);
        gamepad_events.append(&mut other_events);
        gamepad_events
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

    /// 检查指定手柄按钮是否处于按下状态
    fn is_gamepad_button_pressed(&self, id: GamepadId, button: GamepadButton) -> bool {
        self.gamepad_buttons.get(&id).map_or(false, |buttons| buttons.contains(&button))
    }

    /// 获取指定手柄轴的当前值
    fn gamepad_axis_value(&self, id: GamepadId, axis: GamepadAxis) -> f32 {
        self.gamepad_axes.get(&id).and_then(|axes| axes.get(&axis)).copied().unwrap_or(0.0)
    }

    /// 获取当前连接的手柄标识符列表
    fn connected_gamepads(&self) -> Vec<GamepadId> {
        self.connected_gamepads.clone()
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
        input.push_event(InputEvent::GamepadButton {
            id: gp_id,
            button: GamepadButton::South,
            state: KeyState::Pressed,
        });
        assert!(input.is_gamepad_button_pressed(gp_id, GamepadButton::South));
        input.push_event(InputEvent::GamepadButton {
            id: gp_id,
            button: GamepadButton::South,
            state: KeyState::Released,
        });
        assert!(!input.is_gamepad_button_pressed(gp_id, GamepadButton::South));
    }

    #[test]
    fn test_gamepad_axis_state_tracking() {
        let mut input = DesktopInput::new();
        let gp_id = GamepadId(0);
        input.push_event(InputEvent::GamepadAxis {
            id: gp_id,
            axis: GamepadAxis::LeftStickX,
            value: 0.75,
        });
        assert!((input.gamepad_axis_value(gp_id, GamepadAxis::LeftStickX) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gamepad_connected_disconnected() {
        let mut input = DesktopInput::new();
        let gp_id = GamepadId(0);
        input.push_event(InputEvent::GamepadConnected {
            id: gp_id,
            name: "Test Gamepad".to_string(),
        });
        assert!(input.connected_gamepads().contains(&gp_id));
        input.push_event(InputEvent::GamepadDisconnected {
            id: gp_id,
        });
        assert!(!input.connected_gamepads().contains(&gp_id));
    }
}
