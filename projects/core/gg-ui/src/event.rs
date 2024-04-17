use std::collections::HashMap;

use crate::{
    gui_event::{EventContext, EventPhase},
    node::{UiNodeId, UiTree},
};

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
    /// 滚动事件
    Scroll {
        /// 水平滚动增量
        delta_x: f32,
        /// 垂直滚动增量
        delta_y: f32,
        /// 鼠标 X 坐标
        x: f32,
        /// 鼠标 Y 坐标
        y: f32,
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
        Self { handlers: HashMap::new() }
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
            UiEvent::Scroll { x, y, .. } => (*x, *y),
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

    /// 向指定节点分发事件
    ///
    /// 直接将事件发送到指定节点的事件处理器，不执行命中测试。
    /// 如果处理器返回 `true`，表示事件已被消费。
    pub fn dispatch_to_node(&mut self, node_id: UiNodeId, event: &UiEvent) -> bool {
        if let Some(handler) = self.handlers.get_mut(&node_id) {
            return handler(event);
        }
        false
    }

    /// 查找从根节点到目标节点的路径
    ///
    /// 从目标节点向上遍历父节点直到根节点，然后反转得到从根到目标的路径。
    /// 如果目标节点不存在于树中，返回空向量。
    pub fn find_node_path(tree: &UiTree, target_id: UiNodeId) -> Vec<UiNodeId> {
        if tree.get(target_id).is_none() {
            return Vec::new();
        }
        let mut path = Vec::new();
        let mut current = Some(target_id);
        while let Some(id) = current {
            path.push(id);
            current = tree.get(id).and_then(|n| n.parent);
        }
        path.reverse();
        path
    }

    /// 三阶段事件分发
    ///
    /// 按照捕获 → 目标 → 冒泡的顺序分发事件：
    /// 1. **捕获阶段**：从根节点到目标节点的父节点，依次调用已注册的处理器
    /// 2. **目标阶段**：调用目标节点的处理器
    /// 3. **冒泡阶段**：从目标节点的父节点到根节点，依次调用已注册的处理器
    ///
    /// 任何处理器返回 `true` 将立即停止传播并返回 `true`（事件已消费）。
    /// 内部为每个阶段创建 `EventContext` 以标记当前传播阶段。
    pub fn dispatch_with_phases(&mut self, event: &UiEvent, tree: &UiTree, target_id: UiNodeId) -> bool {
        let path = Self::find_node_path(tree, target_id);
        if path.is_empty() {
            return false;
        }

        let capture_path = &path[..path.len().saturating_sub(1)];

        for &node_id in capture_path {
            if let Some(handler) = self.handlers.get_mut(&node_id) {
                let _ctx = EventContext::new(EventPhase::Capturing);
                if handler(event) {
                    return true;
                }
            }
        }

        if let Some(handler) = self.handlers.get_mut(&target_id) {
            let _ctx = EventContext::new(EventPhase::AtTarget);
            if handler(event) {
                return true;
            }
        }

        for &node_id in capture_path.iter().rev() {
            if let Some(handler) = self.handlers.get_mut(&node_id) {
                let _ctx = EventContext::new(EventPhase::Bubbling);
                if handler(event) {
                    return true;
                }
            }
        }

        false
    }

    /// 命中测试
    ///
    /// 从后往前遍历节点（反向渲染顺序），找到包含指定点的最前端可见节点。
    pub fn hit_test(tree: &UiTree, x: f32, y: f32) -> Option<UiNodeId> {
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
            if x >= layout.x && x <= layout.x + layout.width && y >= layout.y && y <= layout.y + layout.height {
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
