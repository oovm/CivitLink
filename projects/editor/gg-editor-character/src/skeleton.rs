//! 骨骼层级结构模块
//! 提供角色骨骼节点管理、层级维护和骨骼绑定功能

use std::collections::HashMap;

/// 骨骼节点，表示角色骨骼层级中的一个节点
#[derive(Debug, Clone)]
pub struct SkeletonNode {
    /// 节点唯一标识
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 父节点 ID
    pub parent_id: Option<String>,
    /// 子节点 ID 列表
    pub children: Vec<String>,
    /// 本地变换位置 [x, y]
    pub local_position: [f32; 2],
    /// 本地旋转角度（弧度）
    pub local_rotation: f32,
    /// 本地缩放
    pub local_scale: [f32; 2],
}

/// 骨骼绑定信息，记录骨骼节点与立绘区域的映射关系
#[derive(Debug, Clone)]
pub struct SkeletonBinding {
    /// 骨骼节点 ID
    pub node_id: String,
    /// 绑定的立绘区域名称
    pub region_name: String,
    /// 绑定偏移量 [x, y]
    pub offset: [f32; 2],
}

/// 骨骼层级结构，管理角色的完整骨骼树
#[derive(Debug, Clone)]
pub struct SkeletonHierarchy {
    /// 根节点 ID
    pub root_id: Option<String>,
    /// 所有骨骼节点
    pub nodes: HashMap<String, SkeletonNode>,
    /// 骨骼绑定映射
    pub bindings: Vec<SkeletonBinding>,
}

impl SkeletonHierarchy {
    /// 创建空的骨骼层级结构
    pub fn new() -> Self {
        Self { root_id: None, nodes: HashMap::new(), bindings: Vec::new() }
    }

    /// 添加骨骼节点到层级结构中
    ///
    /// 如果指定了 parent_id，则将新节点作为该父节点的子节点；
    /// 如果 parent_id 为 None，则将新节点设为根节点。
    /// 返回新创建节点的唯一标识。
    ///
    /// # Panics
    ///
    /// 如果指定的 parent_id 不存在于节点映射中，不会 panic，但节点不会添加到父节点的子列表中。
    pub fn add_node(&mut self, name: String, parent_id: Option<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let node = SkeletonNode {
            id: id.clone(),
            name,
            parent_id: parent_id.clone(),
            children: Vec::new(),
            local_position: [0.0, 0.0],
            local_rotation: 0.0,
            local_scale: [1.0, 1.0],
        };

        if let Some(ref pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(pid) {
                parent.children.push(id.clone());
            }
        }
        else {
            self.root_id = Some(id.clone());
        }

        self.nodes.insert(id.clone(), node);
        id
    }

    /// 移除指定节点及其所有子节点
    ///
    /// 递归移除目标节点的所有子孙节点，并从父节点的子列表中移除引用。
    /// 同时移除与这些节点关联的所有骨骼绑定。
    pub fn remove_node(&mut self, node_id: &str) {
        let children_ids: Vec<String> = self.nodes.get(node_id).map(|n| n.children.clone()).unwrap_or_default();

        for child_id in children_ids {
            self.remove_node(&child_id);
        }

        self.bindings.retain(|b| b.node_id != node_id);

        if let Some(node) = self.nodes.remove(node_id) {
            if let Some(ref pid) = node.parent_id {
                if let Some(parent) = self.nodes.get_mut(pid) {
                    parent.children.retain(|c| c != node_id);
                }
            }

            if self.root_id.as_deref() == Some(node_id) {
                self.root_id = None;
            }
        }
    }

    /// 重命名指定节点
    ///
    /// 如果节点存在，更新其 name 字段。
    pub fn rename_node(&mut self, node_id: &str, new_name: String) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.name = new_name;
        }
    }

    /// 添加骨骼绑定
    ///
    /// 将绑定信息添加到绑定列表中。
    pub fn add_binding(&mut self, binding: SkeletonBinding) {
        self.bindings.push(binding);
    }

    /// 移除指定节点的骨骼绑定
    ///
    /// 从绑定列表中移除所有 node_id 匹配的绑定条目。
    pub fn remove_binding(&mut self, node_id: &str) {
        self.bindings.retain(|b| b.node_id != node_id);
    }

    /// 获取指定节点的不可变引用
    pub fn get_node(&self, node_id: &str) -> Option<&SkeletonNode> {
        self.nodes.get(node_id)
    }

    /// 获取指定节点的可变引用
    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut SkeletonNode> {
        self.nodes.get_mut(node_id)
    }

    /// 获取指定节点的所有子节点引用
    ///
    /// 返回子节点的引用列表，顺序与父节点 children 列表一致。
    /// 如果父节点不存在，返回空列表。
    pub fn children_of(&self, node_id: &str) -> Vec<&SkeletonNode> {
        match self.nodes.get(node_id) {
            Some(node) => node.children.iter().filter_map(|id| self.nodes.get(id)).collect(),
            None => Vec::new(),
        }
    }
}

impl Default for SkeletonHierarchy {
    fn default() -> Self {
        Self::new()
    }
}
