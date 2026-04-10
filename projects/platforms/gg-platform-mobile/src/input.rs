use std::collections::HashSet;

use gg_core::platform::{Input, InputEvent, KeyCode, KeyState, PointerAction, PointerButton};

use crate::mobile_input::{MobileInput, TouchPoint};

/// 移动平台输入实现
///
/// 组合了通用 `Input` trait 和移动平台特有的 `MobileInput` trait，
/// 提供触摸、加速度计和陀螺仪等移动设备输入支持。
pub struct MobileInputImpl {
    /// 待处理的事件缓冲
    event_buffer: Vec<InputEvent>,
    /// 当前按下的按键集合
    pressed_keys: HashSet<KeyCode>,
    /// 指针是否按下
    pointer_down: bool,
    /// 指针当前位置
    pointer_position: (f32, f32),
    /// 活跃的触摸点列表
    active_touches: Vec<TouchPoint>,
    /// 加速度计数据
    accelerometer: Option<(f32, f32, f32)>,
    /// 陀螺仪数据
    gyroscope: Option<(f32, f32, f32)>,
}

impl MobileInputImpl {
    /// 创建新的移动输入实例
    pub fn new() -> Self {
        Self {
            event_buffer: Vec::new(),
            pressed_keys: HashSet::new(),
            pointer_down: false,
            pointer_position: (0.0, 0.0),
            active_touches: Vec::new(),
            accelerometer: None,
            gyroscope: None,
        }
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
            InputEvent::Gamepad { .. } => {}
        }
        self.event_buffer.push(event);
    }

    /// 更新加速度计数据
    pub fn set_accelerometer(&mut self, data: (f32, f32, f32)) {
        self.accelerometer = Some(data);
    }

    /// 更新陀螺仪数据
    pub fn set_gyroscope(&mut self, data: (f32, f32, f32)) {
        self.gyroscope = Some(data);
    }

    /// 更新活跃触摸点
    pub fn set_active_touches(&mut self, touches: Vec<TouchPoint>) {
        self.active_touches = touches;
    }
}

impl Input for MobileInputImpl {
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

impl MobileInput for MobileInputImpl {
    fn active_touches(&self) -> Vec<TouchPoint> {
        self.active_touches.clone()
    }

    fn accelerometer(&self) -> Option<(f32, f32, f32)> {
        self.accelerometer
    }

    fn gyroscope(&self) -> Option<(f32, f32, f32)> {
        self.gyroscope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
