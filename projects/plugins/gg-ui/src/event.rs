use std::collections::HashMap;

use crate::node::{UiNodeId, UiTree};

/// UI 事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    /// 鼠标点击
    Click {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
    /// 鼠标移动
    MouseMove {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
    /// 鼠标按下
    MouseDown {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
    /// 鼠标释放
    MouseUp {
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
    },
    /// 键盘输入
    KeyInput {
        /// 按键名称
        key: String,
    },
}

/// 事件处理器
pub type EventHandler = Box<dyn FnMut(&UiEvent) -> bool + Send + Sync>;

/// 事件系统
///
/// 管理 UI 节点的事件注册和分发。
/// 支持命中测试，将事件路由到正确的节点处理器。
pub struct EventSystem {
    /// 节点到事件处理器的映射
    handlers: HashMap<UiNodeId, EventHandler>,
}

impl EventSystem {
    /// 创建新的事件系统
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 为指定节点注册事件处理器
    pub fn register(&mut self, node_id: UiNodeId, handler: EventHandler) {
        self.handlers.insert(node_id, handler);
    }

    /// 分发事件
    ///
    /// 执行命中测试，找到最前端的节点，然后调用其事件处理器。
    /// 如果处理器返回 `true`，表示事件已被消费，停止传播。
    pub fn dispatch(&mut self, event: &UiEvent, tree: &UiTree) -> bool {
        let (x, y) = match event {
            UiEvent::Click { x, y } => (*x, *y),
            UiEvent::MouseMove { x, y } => (*x, *y),
            UiEvent::MouseDown { x, y } => (*x, *y),
            UiEvent::MouseUp { x, y } => (*x, *y),
            UiEvent::KeyInput { .. } => {
                return false;
            }
        };

        if let Some(hit_id) = Self::hit_test(tree, x, y) {
            if let Some(handler) = self.handlers.get_mut(&hit_id) {
                return handler(event);
            }
        }

        false
    }

    /// 命中测试
    ///
    /// 从后往前遍历节点（反向渲染顺序），找到包含指定点的最前端可见节点。
    fn hit_test(tree: &UiTree, x: f32, y: f32) -> Option<UiNodeId> {
        let root_id = tree.root()?;
        let mut hit: Option<UiNodeId> = None;
        Self::hit_test_node(tree, root_id, x, y, &mut hit);
        hit
    }

    /// 递归命中测试
    fn hit_test_node(tree: &UiTree, node_id: UiNodeId, x: f32, y: f32, hit: &mut Option<UiNodeId>) {
        let node = tree.get(node_id);
        let node = match node {
            Some(n) => n,
            None => return,
        };

        if !node.visible {
            return;
        }

        if let Some(ref layout) = node.layout_result {
            if x >= layout.x
                && x <= layout.x + layout.width
                && y >= layout.y
                && y <= layout.y + layout.height
            {
                *hit = Some(node_id);
            }
        }

        let children = node.children.clone();
        for &child_id in &children {
            Self::hit_test_node(tree, child_id, x, y, hit);
        }
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}
