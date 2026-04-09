//! 编辑器事件系统

use std::any::Any;

/// 订阅标识
///
/// 由 `EventBus::subscribe` 返回，用于唯一标识一个事件订阅。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscriptionId(u64);

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
        Self {
            handlers: Vec::new(),
            pending: Vec::new(),
            next_id: 0,
        }
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
    /// 排空待处理队列，依次将每个事件传递给所有已注册的处理器。
    /// 处理器中调用 `publish` 发布的新事件将在下次 `process_pending` 时处理。
    pub fn process_pending(&mut self) {
        let events = std::mem::take(&mut self.pending);
        for event in &events {
            for (_, handler) in &mut self.handlers {
                handler(event);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
