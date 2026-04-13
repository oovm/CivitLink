use gg_core::GResult;

/// 输入事件枚举
pub enum InputEvent {
    /// 键盘事件
    Key { key: KeyCode, pressed: bool },
    /// 鼠标事件
    Mouse { button: MouseButton, pressed: bool },
    /// 鼠标移动事件
    MouseMove { x: f64, y: f64 },
    /// 鼠标滚轮事件
    MouseWheel { delta_x: f32, delta_y: f32 },
}

/// 键盘按键码枚举
pub enum KeyCode {
    /// 未知按键
    Unknown,
    /// 字母 A
    A,
    /// 字母 B
    B,
    /// 字母 C
    C,
    /// 字母 D
    D,
    /// 字母 E
    E,
    /// 字母 F
    F,
    /// 字母 G
    G,
    /// 字母 H
    H,
    /// 字母 I
    I,
    /// 字母 J
    J,
    /// 字母 K
    K,
    /// 字母 L
    L,
    /// 字母 M
    M,
    /// 字母 N
    N,
    /// 字母 O
    O,
    /// 字母 P
    P,
    /// 字母 Q
    Q,
    /// 字母 R
    R,
    /// 字母 S
    S,
    /// 字母 T
    T,
    /// 字母 U
    U,
    /// 字母 V
    V,
    /// 字母 W
    W,
    /// 字母 X
    X,
    /// 字母 Y
    Y,
    /// 字母 Z
    Z,
    /// 数字 0
    Num0,
    /// 数字 1
    Num1,
    /// 数字 2
    Num2,
    /// 数字 3
    Num3,
    /// 数字 4
    Num4,
    /// 数字 5
    Num5,
    /// 数字 6
    Num6,
    /// 数字 7
    Num7,
    /// 数字 8
    Num8,
    /// 数字 9
    Num9,
    /// 功能键 F1
    F1,
    /// 功能键 F2
    F2,
    /// 功能键 F3
    F3,
    /// 功能键 F4
    F4,
    /// 功能键 F5
    F5,
    /// 功能键 F6
    F6,
    /// 功能键 F7
    F7,
    /// 功能键 F8
    F8,
    /// 功能键 F9
    F9,
    /// 功能键 F10
    F10,
    /// 功能键 F11
    F11,
    /// 功能键 F12
    F12,
    /// 左 Shift
    ShiftLeft,
    /// 右 Shift
    ShiftRight,
    /// 左 Control
    ControlLeft,
    /// 右 Control
    ControlRight,
    /// 左 Alt
    AltLeft,
    /// 右 Alt
    AltRight,
    /// 空格
    Space,
    /// 回车
    Enter,
    /// 退格
    Backspace,
    /// Tab
    Tab,
    /// 左箭头
    ArrowLeft,
    /// 右箭头
    ArrowRight,
    /// 上箭头
    ArrowUp,
    /// 下箭头
    ArrowDown,
    /// 转义
    Escape,
    /// 主页
    Home,
    /// 结束
    End,
    /// 页面上
    PageUp,
    /// 页面下
    PageDown,
    /// 插入
    Insert,
    /// 删除
    Delete,
}

/// 鼠标按键枚举
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 鼠标侧键 1
    Side1,
    /// 鼠标侧键 2
    Side2,
}

/// 平台通用输入接口
///
/// 为各平台提供输入事件处理的统一接口。
pub trait PlatformInput {
    /// 处理输入事件
    fn process_event(&mut self, event: InputEvent);

    /// 获取键盘按键状态
    fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// 获取鼠标按键状态
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;

    /// 获取鼠标位置
    fn get_mouse_position(&self) -> (f64, f64);

    /// 设置鼠标位置
    fn set_mouse_position(&mut self, x: f64, y: f64);

    /// 获取鼠标滚轮增量
    fn get_mouse_wheel_delta(&self) -> (f32, f32);

    /// 清除输入状态
    fn clear(&mut self);
}
