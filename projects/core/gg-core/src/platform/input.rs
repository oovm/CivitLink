#![warn(missing_docs)]

//! 输入抽象层
//! 提供跨平台的键盘、鼠标和手柄输入接口

/// 按键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// 按下
    Pressed,
    /// 释放
    Released,
}

/// 键码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// A 键
    A,
    /// B 键
    B,
    /// C 键
    C,
    /// D 键
    D,
    /// E 键
    E,
    /// F 键
    F,
    /// G 键
    G,
    /// H 键
    H,
    /// I 键
    I,
    /// J 键
    J,
    /// K 键
    K,
    /// L 键
    L,
    /// M 键
    M,
    /// N 键
    N,
    /// O 键
    O,
    /// P 键
    P,
    /// Q 键
    Q,
    /// R 键
    R,
    /// S 键
    S,
    /// T 键
    T,
    /// U 键
    U,
    /// V 键
    V,
    /// W 键
    W,
    /// X 键
    X,
    /// Y 键
    Y,
    /// Z 键
    Z,
    /// 数字 0 键
    Key0,
    /// 数字 1 键
    Key1,
    /// 数字 2 键
    Key2,
    /// 数字 3 键
    Key3,
    /// 数字 4 键
    Key4,
    /// 数字 5 键
    Key5,
    /// 数字 6 键
    Key6,
    /// 数字 7 键
    Key7,
    /// 数字 8 键
    Key8,
    /// 数字 9 键
    Key9,
    /// 空格键
    Space,
    /// 回车键
    Enter,
    /// Esc 键
    Escape,
    /// Tab 键
    Tab,
    /// Shift 键
    Shift,
    /// Control 键
    Control,
    /// Alt 键
    Alt,
    /// 上方向键
    Up,
    /// 下方向键
    Down,
    /// 左方向键
    Left,
    /// 右方向键
    Right,
    /// F1 键
    F1,
    /// F2 键
    F2,
    /// F3 键
    F3,
    /// F4 键
    F4,
    /// F5 键
    F5,
    /// F6 键
    F6,
    /// F7 键
    F7,
    /// F8 键
    F8,
    /// F9 键
    F9,
    /// F10 键
    F10,
    /// F11 键
    F11,
    /// F12 键
    F12,
    /// Home 键
    Home,
    /// End 键
    End,
    /// PageUp 键
    PageUp,
    /// PageDown 键
    PageDown,
    /// Insert 键
    Insert,
    /// Delete 键
    Delete,
    /// Backspace 键
    Backspace,
}

/// 指针动作
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerAction {
    /// 按下
    Down,
    /// 释放
    Up,
    /// 移动
    Move,
    /// 滚动，参数为滚动量
    Scroll(f32),
}

/// 指针按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 其他按钮
    Other(u8),
}

/// 输入事件
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// 键盘事件
    Keyboard {
        /// 按键码
        key: KeyCode,
        /// 按键状态
        state: KeyState,
    },
    /// 指针事件
    Pointer {
        /// 指针位置
        position: (f32, f32),
        /// 指针动作
        action: PointerAction,
        /// 按下的按钮
        button: Option<PointerButton>,
    },
    /// 手柄事件
    Gamepad {
        /// 手柄 ID
        id: u32,
        /// 按钮编号
        button: u32,
        /// 按钮状态
        state: KeyState,
    },
}

/// 输入抽象 trait
pub trait Input {
    /// 轮询所有待处理的输入事件
    fn poll_events(&mut self) -> Vec<InputEvent>;

    /// 检查指定按键是否处于按下状态
    fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// 检查指针（鼠标）是否有按钮按下
    fn is_pointer_down(&self) -> bool;

    /// 获取指针当前位置
    fn pointer_position(&self) -> (f32, f32);
}
