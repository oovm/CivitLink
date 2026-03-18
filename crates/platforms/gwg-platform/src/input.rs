//! 输入抽象
//!
//! 提供跨平台的输入接口。

use super::{PlatformError, PlatformResult};

/// 按键状态
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    /// 释放
    Released,
    /// 按下
    Pressed,
    /// 重复
    Repeated,
}

/// 虚拟按键码
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Unknown,
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
}

/// 鼠标按钮
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// 输入事件
#[derive(Clone, Debug)]
pub enum InputEvent {
    /// 按键事件
    Key {
        key_code: KeyCode,
        state: KeyState,
    },
    /// 字符输入
    Char(char),
    /// 鼠标移动
    MouseMove {
        x: f64,
        y: f64,
    },
    /// 鼠标按钮事件
    MouseButton {
        button: MouseButton,
        state: KeyState,
    },
    /// 鼠标滚轮
    MouseWheel {
        dx: f64,
        dy: f64,
    },
}

/// 输入 trait
pub trait Input {
    /// 检查按键是否按下
    fn is_key_pressed(&self, key_code: KeyCode) -> bool;

    /// 检查按键是否刚被按下
    fn is_key_just_pressed(&self, key_code: KeyCode) -> bool;

    /// 检查按键是否刚被释放
    fn is_key_just_released(&self, key_code: KeyCode) -> bool;

    /// 检查鼠标按钮是否按下
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;

    /// 检查鼠标按钮是否刚被按下
    fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool;

    /// 检查鼠标按钮是否刚被释放
    fn is_mouse_button_just_released(&self, button: MouseButton) -> bool;

    /// 获取鼠标位置
    fn mouse_position(&self) -> (f64, f64);

    /// 获取鼠标移动差值
    fn mouse_delta(&self) -> (f64, f64);

    /// 获取输入事件迭代器
    fn events(&self) -> Box<dyn Iterator<Item = InputEvent> + '_>;

    /// 清空输入状态（每帧结束调用）
    fn clear_state(&mut self);
}

/// 空输入（用于平台不支持输入的情况）
pub struct NullInput;

impl Input for NullInput {
    fn is_key_pressed(&self, _key_code: KeyCode) -> bool {
        false
    }

    fn is_key_just_pressed(&self, _key_code: KeyCode) -> bool {
        false
    }

    fn is_key_just_released(&self, _key_code: KeyCode) -> bool {
        false
    }

    fn is_mouse_button_pressed(&self, _button: MouseButton) -> bool {
        false
    }

    fn is_mouse_button_just_pressed(&self, _button: MouseButton) -> bool {
        false
    }

    fn is_mouse_button_just_released(&self, _button: MouseButton) -> bool {
        false
    }

    fn mouse_position(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn mouse_delta(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn events(&self) -> Box<dyn Iterator<Item = InputEvent> + '_> {
        Box::new(std::iter::empty())
    }

    fn clear_state(&mut self) {}
}
