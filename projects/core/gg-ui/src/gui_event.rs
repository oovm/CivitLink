use std::any::Any;

/// 事件传播阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// 事件从根节点向目标节点传播
    Capturing,
    /// 事件到达目标组件
    AtTarget,
    /// 事件从目标节点向根节点传播
    Bubbling,
}

/// 事件上下文，提供给事件处理器以支持传播控制
pub struct EventContext {
    /// 当前传播阶段
    pub phase: EventPhase,
    /// 是否应停止传播
    pub stop_propagation: bool,
}

impl EventContext {
    /// 为指定阶段创建新的事件上下文
    pub fn new(phase: EventPhase) -> Self {
        Self { phase, stop_propagation: false }
    }

    /// 停止此事件的进一步传播
    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }
}

/// GUI 事件
#[derive(Debug)]
pub enum GuiEvent {
    /// 鼠标点击事件
    MouseClick {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
        /// 鼠标按钮
        button: MouseButton,
    },
    /// 鼠标移动事件
    MouseMove {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
    /// 键盘按键事件
    KeyPress {
        /// 按键
        key: Key,
        /// 修饰键
        modifiers: KeyModifiers,
    },
    /// 文本输入事件
    TextInput {
        /// 输入的文本
        text: String,
    },
    /// 组件特定事件
    Custom {
        /// 事件名称
        name: String,
        /// 事件数据
        data: Box<dyn Any>,
    },
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 其他按钮
    Other(u32),
}

/// 键盘按键
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
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
    /// F 功能键
    FKey(u32),
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
    /// 数字键
    Number(u32),
    /// 空格键
    Space,
    /// 回车键
    Enter,
    /// ESC 键
    Escape,
    /// 退格键
    Backspace,
    /// Tab 键
    Tab,
    /// Shift 键
    Shift,
    /// Control 键
    Control,
    /// Alt 键
    Alt,
    /// 其他键
    Other(String),
}

/// 键盘修饰键
#[derive(Debug, Clone, Default)]
pub struct KeyModifiers {
    /// Shift 是否按下
    pub shift: bool,
    /// Control 是否按下
    pub control: bool,
    /// Alt 是否按下
    pub alt: bool,
    /// Meta 是否按下
    pub meta: bool,
}
