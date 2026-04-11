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
            InputEvent::GamepadButton { .. } => {}
            InputEvent::GamepadAxis { .. } => {}
            InputEvent::GamepadConnected { .. } => {}
            InputEvent::GamepadDisconnected { .. } => {}
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
