//! 服务注册表

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// 类型擦除的服务容器
///
/// 使用 `TypeId` 作为键存储任意实现了 `Any + Send + Sync` 的服务实例，
/// 支持按类型注册和检索服务。
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    /// 创建空的服务注册表
    pub fn new() -> Self {
        Self { services: HashMap::new() }
    }

    /// 注册服务
    ///
    /// 如果相同类型的服务已存在，将被替换。
    pub fn register<T: Any + Send + Sync>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }

    /// 获取服务引用
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.services.get(&TypeId::of::<T>()).and_then(|s| s.downcast_ref::<T>())
    }

    /// 获取服务可变引用
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.services.get_mut(&TypeId::of::<T>()).and_then(|s| s.downcast_mut::<T>())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 窗口标识
///
/// 唯一标识一个编辑器窗口的新类型包装。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(
    /// 窗口的数值标识
    pub u64,
);

/// 窗口服务 trait
///
/// 定义窗口管理操作的抽象接口，包括创建、销毁和聚焦窗口。
pub trait WindowService {
    /// 创建浮动窗口
    ///
    /// 根据给定的标题和尺寸创建一个浮动窗口，返回新窗口的标识。
    fn create_floating_window(&mut self, title: &str, size: (f32, f32)) -> WindowId;

    /// 销毁窗口
    ///
    /// 根据窗口标识销毁指定窗口。
    fn destroy_window(&mut self, id: WindowId);

    /// 聚焦窗口
    ///
    /// 将指定窗口设为焦点窗口。
    fn focus_window(&mut self, id: WindowId);
}

/// 窗口句柄
///
/// 内部用于跟踪窗口状态的数据结构。
struct WindowHandle {
    /// 窗口标识
    id: WindowId,
    /// 窗口标题
    title: String,
    /// 窗口尺寸
    size: (f32, f32),
    /// 是否聚焦
    focused: bool,
}

/// 默认窗口服务
///
/// 使用 `Vec` 存储窗口句柄的简单窗口服务实现。
pub struct DefaultWindowService {
    /// 窗口句柄列表
    windows: Vec<WindowHandle>,
    /// 下一个窗口 ID
    next_id: u64,
    /// 待处理的焦点事件列表
    pending_focus: Vec<u64>,
}

impl DefaultWindowService {
    /// 创建空的默认窗口服务
    pub fn new() -> Self {
        Self { windows: Vec::new(), next_id: 0, pending_focus: Vec::new() }
    }

    /// 排空待处理的焦点事件
    ///
    /// 返回所有待处理的焦点窗口 ID，并清空内部队列。
    pub fn drain_pending_focus(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.pending_focus)
    }
}

impl Default for DefaultWindowService {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowService for DefaultWindowService {
    fn create_floating_window(&mut self, title: &str, size: (f32, f32)) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        self.windows.push(WindowHandle { id: WindowId(id.0), title: title.to_string(), size, focused: false });
        id
    }

    fn destroy_window(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id.0 != id.0);
    }

    fn focus_window(&mut self, id: WindowId) {
        for w in &mut self.windows {
            w.focused = w.id.0 == id.0;
        }
        self.pending_focus.push(id.0);
    }
}

/// 待处理的窗口创建请求
///
/// 由于 winit 窗口创建需要事件循环，此结构暂存窗口创建参数，
/// 等待在事件循环中实际创建 winit 窗口。
pub struct PendingWindowCreate {
    /// 窗口标识
    pub window_id: WindowId,
    /// 窗口标题
    pub title: String,
    /// 窗口尺寸 (宽, 高)
    pub size: (f32, f32),
}

/// Winit 窗口信息
///
/// 跟踪由 WinitWindowService 管理的窗口状态。
pub struct WinitWindowInfo {
    /// 窗口标识
    pub id: WindowId,
    /// 窗口标题
    pub title: String,
    /// 窗口尺寸 (宽, 高)
    pub size: (f32, f32),
    /// 是否聚焦
    pub focused: bool,
}

/// Winit 窗口服务
///
/// 与 winit 事件循环配合使用的窗口服务实现。
/// 窗口创建和销毁操作通过待处理队列延迟执行，
/// 以便在事件循环中统一处理实际的 winit 窗口操作。
pub struct WinitWindowService {
    /// 下一个窗口 ID 计数器
    next_id: u64,
    /// 已创建的窗口信息映射
    windows: HashMap<u64, WinitWindowInfo>,
    /// 待处理的窗口创建请求
    pending_creates: Vec<PendingWindowCreate>,
    /// 待处理的窗口销毁请求
    pending_destroys: Vec<WindowId>,
    /// 待处理的窗口焦点事件
    pending_focus: Vec<u64>,
}

impl WinitWindowService {
    /// 创建空的 Winit 窗口服务
    pub fn new() -> Self {
        Self {
            next_id: 0,
            windows: HashMap::new(),
            pending_creates: Vec::new(),
            pending_destroys: Vec::new(),
            pending_focus: Vec::new(),
        }
    }

    /// 排空待处理的窗口创建请求
    ///
    /// 返回所有待处理的窗口创建请求，并清空内部队列。
    pub fn drain_pending_creates(&mut self) -> Vec<PendingWindowCreate> {
        std::mem::take(&mut self.pending_creates)
    }

    /// 排空待处理的窗口销毁请求
    ///
    /// 返回所有待处理的窗口销毁 ID，并清空内部队列。
    pub fn drain_pending_destroys(&mut self) -> Vec<WindowId> {
        std::mem::take(&mut self.pending_destroys)
    }

    /// 排空待处理的焦点事件
    ///
    /// 返回所有待处理的焦点窗口 ID，并清空内部队列。
    pub fn drain_pending_focus(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.pending_focus)
    }
}

impl Default for WinitWindowService {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowService for WinitWindowService {
    fn create_floating_window(&mut self, title: &str, size: (f32, f32)) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;
        self.windows.insert(
            id.0,
            WinitWindowInfo {
                id,
                title: title.to_string(),
                size,
                focused: false,
            },
        );
        self.pending_creates.push(PendingWindowCreate {
            window_id: id,
            title: title.to_string(),
            size,
        });
        id
    }

    fn destroy_window(&mut self, id: WindowId) {
        self.windows.remove(&id.0);
        self.pending_destroys.push(id);
    }

    fn focus_window(&mut self, id: WindowId) {
        if let Some(info) = self.windows.get_mut(&id.0) {
            info.focused = true;
        }
        self.pending_focus.push(id.0);
    }
}
