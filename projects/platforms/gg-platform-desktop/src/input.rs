use std::collections::HashSet;

use gg_core::platform::{Input, InputEvent, KeyCode, KeyState, PointerAction, PointerButton};

/// 桌面平台输入实现
///
/// 维护内部按键状态映射和指针状态，通过 `push_event` 接收来自窗口系统的事件。
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
