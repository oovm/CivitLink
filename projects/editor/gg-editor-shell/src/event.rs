//! 编辑器事件系统

use std::any::Any;

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
}

/// 键盘按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    Num0,
    /// 数字 1 键
    Num1,
    /// 数字 2 键
    Num2,
    /// 数字 3 键
    Num3,
    /// 数字 4 键
    Num4,
    /// 数字 5 键
    Num5,
    /// 数字 6 键
    Num6,
    /// 数字 7 键
    Num7,
    /// 数字 8 键
    Num8,
    /// 数字 9 键
    Num9,
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
    /// Escape 键
    Escape,
    /// Enter 键
    Enter,
    /// Space 键
    Space,
    /// Tab 键
    Tab,
    /// Backspace 键
    Backspace,
    /// Delete 键
    Delete,
    /// Insert 键
    Insert,
    /// Home 键
    Home,
    /// End 键
    End,
    /// PageUp 键
    PageUp,
    /// PageDown 键
    PageDown,
    /// 上方向键
    ArrowUp,
    /// 下方向键
    ArrowDown,
    /// 左方向键
    ArrowLeft,
    /// 右方向键
    ArrowRight,
    /// Shift 键
    Shift,
    /// Control 键
    Control,
    /// Alt 键
    Alt,
}

/// 订阅标识
///
/// 由 `EventBus::subscribe` 返回，用于唯一标识一个事件订阅。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscriptionId(u64);

/// 拖拽数据枚举
///
/// 定义面板间拖拽操作携带的数据类型。
#[derive(Debug, Clone)]
pub enum DragData {
    /// 资源路径拖拽
    AssetPath(String),
    /// 多资源路径批量拖拽
    MultiAsset(Vec<String>),
    /// 实体拖拽
    Entity(u64),
    /// 自定义拖拽数据
    Custom {
        /// 数据类型标识
        kind: String,
        /// 数据内容
        data: String,
    },
}

/// 拖拽状态
///
/// 管理当前拖拽操作的状态信息。
#[derive(Debug, Clone)]
pub struct DragState {
    /// 拖拽数据
    pub data: DragData,
    /// 拖拽起始位置
    pub start_position: (f32, f32),
    /// 是否正在拖拽中
    pub is_dragging: bool,
}

impl DragState {
    /// 创建新的拖拽状态
    pub fn new(data: DragData, start_position: (f32, f32)) -> Self {
        Self { data, start_position, is_dragging: true }
    }
}

/// 拖拽视觉反馈状态
///
/// 管理拖拽操作过程中的视觉反馈信息，包括目标面板的高亮状态。
/// 有效放置目标显示蓝色边框，无效放置目标显示红色边框。
#[derive(Debug, Clone)]
pub struct DragVisualFeedback {
    /// 当前是否正在拖拽
    pub is_dragging: bool,
    /// 拖拽目标的合法面板名称列表
    pub valid_targets: Vec<String>,
    /// 当前鼠标悬停的面板名称
    pub hover_target: Option<String>,
}

impl DragVisualFeedback {
    /// 创建新的拖拽视觉反馈状态
    pub fn new() -> Self {
        Self { is_dragging: false, valid_targets: Vec::new(), hover_target: None }
    }

    /// 开始拖拽，设置有效目标面板列表
    pub fn start_drag(&mut self, valid_targets: Vec<String>) {
        self.is_dragging = true;
        self.valid_targets = valid_targets;
        self.hover_target = None;
    }

    /// 结束拖拽，清除所有状态
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.valid_targets.clear();
        self.hover_target = None;
    }

    /// 设置当前悬停目标面板
    pub fn set_hover_target(&mut self, target: Option<String>) {
        self.hover_target = target;
    }

    /// 判断指定面板是否为有效放置目标
    pub fn is_valid_target(&self, panel_name: &str) -> bool {
        self.valid_targets.iter().any(|t| t == panel_name)
    }

    /// 获取指定面板的拖拽高亮颜色
    ///
    /// 返回 (r, g, b, a) 颜色值：
    /// - 正在拖拽且为有效目标且正在悬停：蓝色 (0.2, 0.5, 1.0, 0.5)
    /// - 正在拖拽且为无效目标且正在悬停：红色 (1.0, 0.2, 0.2, 0.5)
    /// - 其他情况：透明 (0.0, 0.0, 0.0, 0.0)
    pub fn highlight_color(&self, panel_name: &str) -> (f32, f32, f32, f32) {
        if !self.is_dragging {
            return (0.0, 0.0, 0.0, 0.0);
        }
        if self.hover_target.as_deref() == Some(panel_name) {
            if self.is_valid_target(panel_name) { (0.2, 0.5, 1.0, 0.5) } else { (1.0, 0.2, 0.2, 0.5) }
        }
        else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Default for DragVisualFeedback {
    fn default() -> Self {
        Self::new()
    }
}

/// 编辑器事件枚举
///
/// 定义了编辑器中所有内置事件类型，以及支持携带任意数据的自定义事件。
pub enum EditorEvent {
    /// 实体被选中
    EntitySelected {
        /// 选中的实体 ID
        entity: u64,
    },
    /// 实体取消选中
    EntityDeselected,
    /// 属性变更
    PropertyChanged {
        /// 实体 ID
        entity: u64,
        /// 组件名称
        component: String,
        /// 属性名称
        property: String,
    },
    /// 文件变更
    FileChanged {
        /// 文件路径
        path: String,
    },
    /// 场景加载完成
    SceneLoaded {
        /// 场景名称
        scene_name: String,
    },
    /// 场景卸载完成
    SceneUnloaded {
        /// 场景名称
        scene_name: String,
    },
    /// 窗口获得焦点
    WindowFocused {
        /// 窗口 ID
        window_id: u64,
    },
    /// 窗口已创建
    WindowCreated {
        /// 窗口 ID
        window_id: u64,
    },
    /// 窗口已销毁
    WindowDestroyed {
        /// 窗口 ID
        window_id: u64,
    },
    /// 面板已注册
    PanelRegistered {
        /// 面板名称
        panel_name: String,
    },
    /// 面板已注销
    PanelUnregistered {
        /// 面板名称
        panel_name: String,
    },
    /// 鼠标按下
    MouseDown {
        /// 按下的鼠标按钮
        button: MouseButton,
        /// 鼠标位置 (x, y)
        position: (f32, f32),
    },
    /// 鼠标释放
    MouseUp {
        /// 释放的鼠标按钮
        button: MouseButton,
        /// 鼠标位置 (x, y)
        position: (f32, f32),
    },
    /// 鼠标移动
    MouseMove {
        /// 鼠标位置 (x, y)
        position: (f32, f32),
    },
    /// 鼠标滚轮
    MouseWheel {
        /// 滚轮增量 (水平, 垂直)
        delta: (f32, f32),
        /// 鼠标位置 (x, y)
        position: (f32, f32),
    },
    /// 键盘按下
    KeyDown {
        /// 按下的键
        key: Key,
    },
    /// 键盘释放
    KeyUp {
        /// 释放的键
        key: Key,
    },
    /// 插件已加载
    PluginLoaded {
        /// 插件名称
        plugin_name: String,
    },
    /// 插件已卸载
    PluginUnloaded {
        /// 插件名称
        plugin_name: String,
    },
    /// 插件已重载
    PluginReloaded {
        /// 插件名称
        plugin_name: String,
    },
    /// 布局已保存
    LayoutSaved {
        /// 布局文件路径
        path: String,
    },
    /// 布局已加载
    LayoutLoaded {
        /// 布局文件路径
        path: String,
    },
    /// 布局已重置为默认
    LayoutReset,
    /// 窗口大小变更
    WindowResized {
        /// 窗口 ID
        window_id: u64,
        /// 新宽度
        width: u32,
        /// 新高度
        height: u32,
    },
    /// 预览已启动
    PreviewStarted,
    /// 预览已停止
    PreviewStopped,
    /// 预览已暂停
    PreviewPaused,
    /// 预览已恢复
    PreviewResumed,
    /// HMR 重载已触发
    HmrReloadTriggered {
        /// 变更文件列表
        changed_files: Vec<String>,
    },
    /// HMR 重载已完成
    HmrReloadCompleted {
        /// 是否成功
        success: bool,
    },
    /// 拖拽开始
    DragStart {
        /// 拖拽数据
        data: DragData,
    },
    /// 拖拽移动
    DragMove {
        /// 当前鼠标位置
        position: (f32, f32),
    },
    /// 拖拽结束
    DragEnd {
        /// 释放位置
        position: (f32, f32),
        /// 拖拽数据
        data: DragData,
    },
    /// 拖拽取消
    DragCancel,
    /// 自定义事件
    Custom {
        /// 事件名称
        name: String,
        /// 事件数据
        data: Box<dyn Any + Send + Sync>,
    },
}

/// 事件总线
///
/// 提供发布-订阅模式的事件系统，支持订阅处理器、发布事件到待处理队列，
/// 以及批量处理待处理事件。
pub struct EventBus {
    handlers: Vec<(SubscriptionId, Box<dyn FnMut(&EditorEvent)>)>,
    pending: Vec<EditorEvent>,
    next_id: u64,
}

impl EventBus {
    /// 创建空的事件总线
    pub fn new() -> Self {
        Self { handlers: Vec::new(), pending: Vec::new(), next_id: 0 }
    }

    /// 订阅事件
    ///
    /// 注册一个事件处理器，所有经 `publish` 发布的事件在 `process_pending` 时
    /// 都会调用该处理器。返回一个 `SubscriptionId` 用于标识此订阅。
    pub fn subscribe(&mut self, handler: Box<dyn FnMut(&EditorEvent)>) -> SubscriptionId {
        let id = SubscriptionId(self.next_id);
        self.next_id += 1;
        self.handlers.push((id, handler));
        id
    }

    /// 发布事件到待处理队列
    ///
    /// 事件不会立即分发，而是在下次调用 `process_pending` 时处理。
    pub fn publish(&mut self, event: EditorEvent) {
        self.pending.push(event);
    }

    /// 处理待处理事件
    ///
    /// 排空待处理队列，依次将每个事件传递给所有已注册的处理器，
    /// 并返回本次处理的事件列表以供进一步分发。
    /// 处理器中调用 `publish` 发布的新事件将在下次 `process_pending` 时处理。
    pub fn process_pending(&mut self) -> Vec<EditorEvent> {
        let events = std::mem::take(&mut self.pending);
        for event in &events {
            for (_, handler) in &mut self.handlers {
                handler(event);
            }
        }
        events
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
