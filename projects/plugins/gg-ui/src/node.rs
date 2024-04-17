use std::collections::HashMap;

use crate::{layout::LayoutResult, style::Style};

/// UI 节点 ID
pub type UiNodeId = u64;

/// UI 节点数据
#[derive(Debug, Clone, PartialEq)]
pub enum UiNodeData {
    /// 空容器
    Container,
    /// 文本内容
    Text {
        /// 文本字符串
        content: String,
    },
    /// 自定义控件
    Custom {
        /// 控件类型标识
        kind: String,
    },
}

/// UI 节点
#[derive(Debug, Clone)]
pub struct UiNode {
    /// 节点 ID
    pub id: UiNodeId,
    /// 节点标签（用于调试和查找）
    pub label: String,
    /// 样式
    pub style: Style,
    /// 子节点 ID 列表
    pub children: Vec<UiNodeId>,
    /// 父节点 ID
    pub parent: Option<UiNodeId>,
    /// 节点数据（文本内容等）
    pub data: UiNodeData,
    /// 布局计算结果
    pub layout_result: Option<LayoutResult>,
    /// 是否可见
    pub visible: bool,
}

/// UI 树
#[derive(Debug, Clone)]
pub struct UiTree {
    /// 节点映射
    nodes: HashMap<UiNodeId, UiNode>,
    /// 根节点 ID
    root: Option<UiNodeId>,
    /// 下一个节点 ID
    next_id: UiNodeId,
}

impl UiTree {
    /// 创建空的 UI 树
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), root: None, next_id: 1 }
    }

    /// 创建新节点并插入树中
    ///
    /// 返回新节点的 ID。
    pub fn create_node(&mut self, label: impl Into<String>, style: Style, data: UiNodeData) -> UiNodeId {
        let id = self.next_id;
        self.next_id += 1;
        let node = UiNode {
            id,
            label: label.into(),
            style,
            children: Vec::new(),
            parent: None,
            data,
            layout_result: None,
            visible: true,
        };
        self.nodes.insert(id, node);
        id
    }

    /// 将子节点添加到父节点的子列表中
    ///
    /// 如果父节点或子节点不存在，返回 `false`。
    pub fn add_child(&mut self, parent_id: UiNodeId, child_id: UiNodeId) -> bool {
        if !self.nodes.contains_key(&parent_id) || !self.nodes.contains_key(&child_id) {
            return false;
        }
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            if !parent.children.contains(&child_id) {
                parent.children.push(child_id);
            }
        }
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = Some(parent_id);
        }
        true
    }

    /// 从树中移除节点及其所有子节点
    ///
    /// 如果节点不存在，返回 `false`。
    pub fn remove_node(&mut self, node_id: UiNodeId) -> bool {
        if !self.nodes.contains_key(&node_id) {
            return false;
        }
        let children: Vec<UiNodeId> = self.nodes.get(&node_id).map(|n| n.children.clone()).unwrap_or_default();
        for child_id in children {
            self.remove_node(child_id);
        }
        if let Some(node) = self.nodes.remove(&node_id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|&id| id != node_id);
                }
            }
            if self.root == Some(node_id) {
                self.root = None;
            }
        }
        true
    }

    /// 获取节点的不可变引用
    pub fn get(&self, id: UiNodeId) -> Option<&UiNode> {
        self.nodes.get(&id)
    }

    /// 获取节点的可变引用
    pub fn get_mut(&mut self, id: UiNodeId) -> Option<&mut UiNode> {
        self.nodes.get_mut(&id)
    }

    /// 获取根节点 ID
    pub fn root(&self) -> Option<UiNodeId> {
        self.root
    }

    /// 获取指定节点的子节点 ID 列表
    pub fn children_of(&self, id: UiNodeId) -> Option<&[UiNodeId]> {
        self.nodes.get(&id).map(|n| n.children.as_slice())
    }

    /// 设置根节点
    ///
    /// 如果节点不存在，返回 `false`。
    pub fn set_root(&mut self, id: UiNodeId) -> bool {
        if !self.nodes.contains_key(&id) {
            return false;
        }
        self.root = Some(id);
        true
    }
}

impl Default for UiTree {
    fn default() -> Self {
        Self::new()
    }
}
