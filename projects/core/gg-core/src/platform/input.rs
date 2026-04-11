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

/// 手柄标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GamepadId(pub u32);

/// 手柄按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    /// 南方按钮（Xbox A / PlayStation Cross）
    South,
    /// 东方按钮（Xbox B / PlayStation Circle）
    East,
    /// 西方按钮（Xbox X / PlayStation Square）
    West,
    /// 北方按钮（Xbox Y / PlayStation Triangle）
    North,
    /// 开始按钮
    Start,
    /// 返回按钮
    Back,
    /// 导航按钮（Xbox / PlayStation Home）
    Guide,
    /// 左摇杆按下
    LeftThumb,
    /// 右摇杆按下
    RightThumb,
    /// 左肩键（LB / L1）
    LeftShoulder,
    /// 右肩键（RB / R1）
    RightShoulder,
    /// 方向键上
    DPadUp,
    /// 方向键下
    DPadDown,
    /// 方向键左
    DPadLeft,
    /// 方向键右
    DPadRight,
}

/// 手柄轴
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    /// 左摇杆 X 轴
    LeftStickX,
    /// 左摇杆 Y 轴
    LeftStickY,
    /// 右摇杆 X 轴
    RightStickX,
    /// 右摇杆 Y 轴
    RightStickY,
    /// 左扳机（LT / L2）
    LeftTrigger,
    /// 右扳机（RT / R2）
    RightTrigger,
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
    /// 手柄按钮事件
    GamepadButton {
        /// 手柄标识符
        id: GamepadId,
        /// 手柄按钮
        button: GamepadButton,
        /// 按钮状态
        state: KeyState,
    },
    /// 手柄轴事件
    GamepadAxis {
        /// 手柄标识符
        id: GamepadId,
        /// 手柄轴
        axis: GamepadAxis,
        /// 轴值
        value: f32,
    },
    /// 手柄连接事件
    GamepadConnected {
        /// 手柄标识符
        id: GamepadId,
        /// 手柄名称
        name: String,
    },
    /// 手柄断开事件
    GamepadDisconnected {
        /// 手柄标识符
        id: GamepadId,
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

    /// 检查指定手柄按钮是否处于按下状态
    fn is_gamepad_button_pressed(&self, _id: GamepadId, _button: GamepadButton) -> bool {
        false
    }

    /// 获取指定手柄轴的当前值
    fn gamepad_axis_value(&self, _id: GamepadId, _axis: GamepadAxis) -> f32 {
        0.0
    }

    /// 获取当前连接的手柄标识符列表
    fn connected_gamepads(&self) -> Vec<GamepadId> {
        Vec::new()
    }
}
