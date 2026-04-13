use std::collections::HashSet;

use gg_core::platform::{Input, InputEvent, KeyCode, KeyState, PointerAction};

/// Android 平台输入实现
///
/// 为 Android 平台提供输入事件处理的具体实现。
pub struct AndroidInputImpl {
    /// 待处理的事件缓冲
    event_buffer: Vec<InputEvent>,
    /// 当前按下的按键集合
    pressed_keys: HashSet<KeyCode>,
    /// 指针是否按下
    pointer_down: bool,
    /// 指针当前位置
    pointer_position: (f32, f32),
}

impl AndroidInputImpl {
    /// 创建 Android 输入实例
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
                if let PointerAction::Down = action {
                    self.pointer_down = true;
                }
                else if let PointerAction::Up = action {
                    self.pointer_down = false;
                }
            }
            _ => {}
        }
        self.event_buffer.push(event);
    }
}

impl Input for AndroidInputImpl {
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
