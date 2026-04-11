use std::collections::HashSet;

use gg_core::platform::{Input, InputEvent, KeyCode, KeyState, PointerAction, PointerButton};

/// Web 平台输入实现
///
/// 通过 JavaScript 互操作接收 DOM 事件，转换为统一的输入事件。
/// 提供两种事件推送方式：
/// - 直接推送已转换的 `InputEvent`（通过 `push_event`）
/// - 推送原始 DOM 事件参数（通过 `push_keyboard_event`、`push_raw_mouse_event`、`push_raw_touch_event`）
pub struct WebInput {
    /// 待处理的事件缓冲
    event_buffer: Vec<InputEvent>,
    /// 当前按下的按键集合
    pressed_keys: HashSet<KeyCode>,
    /// 指针是否按下
    pointer_down: bool,
    /// 指针当前位置
    pointer_position: (f32, f32),
}

impl WebInput {
    /// 创建新的 Web 输入实例
    pub fn new() -> Self {
        Self { event_buffer: Vec::new(), pressed_keys: HashSet::new(), pointer_down: false, pointer_position: (0.0, 0.0) }
    }

    /// 推送一个输入事件到缓冲区
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
            InputEvent::Pointer { position, action, .. } => {
                self.pointer_position = *position;
                match action {
                    PointerAction::Down => {
                        self.pointer_down = true;
                    }
                    PointerAction::Up => {
                        self.pointer_down = false;
                    }
                    _ => {}
                }
            }
            InputEvent::GamepadButton { .. } => {}
            InputEvent::GamepadAxis { .. } => {}
            InputEvent::GamepadConnected { .. } => {}
            InputEvent::GamepadDisconnected { .. } => {}
        }
        self.event_buffer.push(event);
    }

    /// 从 DOM KeyboardEvent 推送键盘事件
    ///
    /// # 参数
    ///
    /// - `code` - 键盘事件的 code 字符串（如 "KeyA", "ArrowUp"）
    /// - `pressed` - 是否为按下事件
    pub fn push_keyboard_event(&mut self, code: &str, pressed: bool) {
        if let Some(key) = Self::parse_key_code(code) {
            let state = if pressed { KeyState::Pressed } else { KeyState::Released };
            match state {
                KeyState::Pressed => {
                    self.pressed_keys.insert(key);
                }
                KeyState::Released => {
                    self.pressed_keys.remove(&key);
                }
            }
            self.event_buffer.push(InputEvent::Keyboard { key, state });
        }
    }

    /// 从 DOM MouseEvent 推送鼠标事件
    ///
    /// # 参数
    ///
    /// - `x` - 鼠标 X 坐标
    /// - `y` - 鼠标 Y 坐标
    /// - `action` - 指针动作
    /// - `button` - 鼠标按钮
    pub fn push_mouse_event(&mut self, x: f32, y: f32, action: PointerAction, button: PointerButton) {
        self.pointer_position = (x, y);
        match &action {
            PointerAction::Down => {
                self.pointer_down = true;
            }
            PointerAction::Up => {
                self.pointer_down = false;
            }
            _ => {}
        }
        self.event_buffer.push(InputEvent::Pointer { position: (x, y), action, button: Some(button) });
    }

    /// 从原始 DOM MouseEvent 参数推送鼠标事件
    ///
    /// # 参数
    ///
    /// - `x` - 鼠标 X 坐标
    /// - `y` - 鼠标 Y 坐标
    /// - `event_type` - DOM 事件类型字符串（"mousedown", "mouseup", "mousemove", "wheel"）
    /// - `button` - DOM MouseEvent.button 数值（0=左键, 1=中键, 2=右键）
    /// - `scroll_delta` - 滚轮滚动量（仅 "wheel" 事件时使用）
    pub fn push_raw_mouse_event(&mut self, x: f32, y: f32, event_type: &str, button: u16, scroll_delta: f32) {
        let action = Self::parse_pointer_action(event_type, scroll_delta);
        let pointer_button = Self::parse_mouse_button(button);
        self.push_mouse_event(x, y, action, pointer_button);
    }

    /// 从 DOM TouchEvent 推送触摸事件
    ///
    /// # 参数
    ///
    /// - `x` - 触摸 X 坐标
    /// - `y` - 触摸 Y 坐标
    /// - `action` - 指针动作
    pub fn push_touch_event(&mut self, x: f32, y: f32, action: PointerAction) {
        self.pointer_position = (x, y);
        match &action {
            PointerAction::Down => {
                self.pointer_down = true;
            }
            PointerAction::Up => {
                self.pointer_down = false;
            }
            _ => {}
        }
        self.event_buffer.push(InputEvent::Pointer { position: (x, y), action, button: Some(PointerButton::Left) });
    }

    /// 从原始 DOM TouchEvent 参数推送触摸事件
    ///
    /// # 参数
    ///
    /// - `x` - 触摸 X 坐标
    /// - `y` - 触摸 Y 坐标
    /// - `event_type` - DOM 事件类型字符串（"touchstart", "touchend", "touchmove"）
    pub fn push_raw_touch_event(&mut self, x: f32, y: f32, event_type: &str) {
        let action = Self::parse_pointer_action(event_type, 0.0);
        self.push_touch_event(x, y, action);
    }

    /// 将 DOM 事件类型字符串解析为 PointerAction
    fn parse_pointer_action(event_type: &str, scroll_delta: f32) -> PointerAction {
        match event_type {
            "mousedown" | "touchstart" => PointerAction::Down,
            "mouseup" | "touchend" => PointerAction::Up,
            "mousemove" | "touchmove" => PointerAction::Move,
            "wheel" => PointerAction::Scroll(scroll_delta),
            _ => PointerAction::Move,
        }
    }

    /// 将 DOM MouseEvent.button 数值解析为 PointerButton
    fn parse_mouse_button(button: u16) -> PointerButton {
        match button {
            0 => PointerButton::Left,
            1 => PointerButton::Middle,
            2 => PointerButton::Right,
            n => PointerButton::Other(n as u8),
        }
    }

    /// 将 DOM KeyboardEvent.code 字符串解析为 KeyCode
    fn parse_key_code(code: &str) -> Option<KeyCode> {
        match code {
            "KeyA" => Some(KeyCode::A),
            "KeyB" => Some(KeyCode::B),
            "KeyC" => Some(KeyCode::C),
            "KeyD" => Some(KeyCode::D),
            "KeyE" => Some(KeyCode::E),
            "KeyF" => Some(KeyCode::F),
            "KeyG" => Some(KeyCode::G),
            "KeyH" => Some(KeyCode::H),
            "KeyI" => Some(KeyCode::I),
            "KeyJ" => Some(KeyCode::J),
            "KeyK" => Some(KeyCode::K),
            "KeyL" => Some(KeyCode::L),
            "KeyM" => Some(KeyCode::M),
            "KeyN" => Some(KeyCode::N),
            "KeyO" => Some(KeyCode::O),
            "KeyP" => Some(KeyCode::P),
            "KeyQ" => Some(KeyCode::Q),
            "KeyR" => Some(KeyCode::R),
            "KeyS" => Some(KeyCode::S),
            "KeyT" => Some(KeyCode::T),
            "KeyU" => Some(KeyCode::U),
            "KeyV" => Some(KeyCode::V),
            "KeyW" => Some(KeyCode::W),
            "KeyX" => Some(KeyCode::X),
            "KeyY" => Some(KeyCode::Y),
            "KeyZ" => Some(KeyCode::Z),
            "Digit0" => Some(KeyCode::Key0),
            "Digit1" => Some(KeyCode::Key1),
            "Digit2" => Some(KeyCode::Key2),
            "Digit3" => Some(KeyCode::Key3),
            "Digit4" => Some(KeyCode::Key4),
            "Digit5" => Some(KeyCode::Key5),
            "Digit6" => Some(KeyCode::Key6),
            "Digit7" => Some(KeyCode::Key7),
            "Digit8" => Some(KeyCode::Key8),
            "Digit9" => Some(KeyCode::Key9),
            "Space" => Some(KeyCode::Space),
            "Enter" => Some(KeyCode::Enter),
            "Escape" => Some(KeyCode::Escape),
            "Tab" => Some(KeyCode::Tab),
            "ShiftLeft" | "ShiftRight" => Some(KeyCode::Shift),
            "ControlLeft" | "ControlRight" => Some(KeyCode::Control),
            "AltLeft" | "AltRight" => Some(KeyCode::Alt),
            "ArrowUp" => Some(KeyCode::Up),
            "ArrowDown" => Some(KeyCode::Down),
            "ArrowLeft" => Some(KeyCode::Left),
            "ArrowRight" => Some(KeyCode::Right),
            "Backspace" => Some(KeyCode::Backspace),
            "Insert" => Some(KeyCode::Insert),
            "Delete" => Some(KeyCode::Delete),
            "Home" => Some(KeyCode::Home),
            "End" => Some(KeyCode::End),
            "PageUp" => Some(KeyCode::PageUp),
            "PageDown" => Some(KeyCode::PageDown),
            "F1" => Some(KeyCode::F1),
            "F2" => Some(KeyCode::F2),
            "F3" => Some(KeyCode::F3),
            "F4" => Some(KeyCode::F4),
            "F5" => Some(KeyCode::F5),
            "F6" => Some(KeyCode::F6),
            "F7" => Some(KeyCode::F7),
            "F8" => Some(KeyCode::F8),
            "F9" => Some(KeyCode::F9),
            "F10" => Some(KeyCode::F10),
            "F11" => Some(KeyCode::F11),
            "F12" => Some(KeyCode::F12),
            _ => None,
        }
    }
}

impl Input for WebInput {
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


