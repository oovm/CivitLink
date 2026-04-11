use std::collections::HashMap;

use gg_render::TextureId;

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
    /// 图片节点
    Image {
        /// 纹理标识符
        texture_id: Option<TextureId>,
        /// 图片尺寸 `[width, height]`
        size: Option<(f32, f32)>,
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
    /// 样式版本号，每次通过 set_style 修改样式时递增
    pub style_version: u64,
    /// 无障碍属性
    pub accessibility: AccessibilityNode,
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

    /// 清空所有节点并重置状态
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.next_id = 1;
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
            style_version: 0,
            accessibility: AccessibilityNode::default(),
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

    /// 将另一个 UiTree 的所有节点合并到当前树中
    ///
    /// 源树中的节点 ID 会被重新映射，以避免与当前树中已有的节点 ID 冲突。
    /// 合并后，源树的根节点将成为合并后树的根节点。
    pub fn merge_from(&mut self, other: &UiTree) {
        let mut id_map: HashMap<UiNodeId, UiNodeId> = HashMap::new();

        let mut sorted_ids: Vec<UiNodeId> = other.nodes.keys().copied().collect();
        sorted_ids.sort();

        for &old_id in &sorted_ids {
            if let Some(node) = other.nodes.get(&old_id) {
                let new_id = self.next_id;
                self.next_id += 1;
                id_map.insert(old_id, new_id);

                let new_node = UiNode {
                    id: new_id,
                    label: node.label.clone(),
                    style: node.style.clone(),
                    children: Vec::new(),
                    parent: None,
                    data: node.data.clone(),
                    layout_result: None,
                    visible: node.visible,
                    style_version: 0,
                    accessibility: node.accessibility.clone(),
                };
                self.nodes.insert(new_id, new_node);
            }
        }

        for &old_id in &sorted_ids {
            if let Some(node) = other.nodes.get(&old_id) {
                let new_id = id_map[&old_id];
                if let Some(self_node) = self.nodes.get_mut(&new_id) {
                    self_node.children = node.children.iter().filter_map(|&c| id_map.get(&c).copied()).collect();
                    self_node.parent = node.parent.and_then(|p| id_map.get(&p).copied());
                }
            }
        }

        if let Some(other_root) = other.root {
            if let Some(&new_root_id) = id_map.get(&other_root) {
                self.root = Some(new_root_id);
            }
        }
    }
}

impl Default for UiTree {
    fn default() -> Self {
        Self::new()
    }
}

/// 无障碍角色
///
/// 定义 UI 节点在无障碍树中的语义角色，
/// 类似于 HTML ARIA role 属性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AriaRole {
    /// 无特定角色
    None,
    /// 按钮
    Button,
    /// 复选框
    Checkbox,
    /// 组合框
    ComboBox,
    /// 对话框
    Dialog,
    /// 图片
    Image,
    /// 链接
    Link,
    /// 列表
    List,
    /// 列表项
    ListItem,
    /// 菜单
    Menu,
    /// 菜单项
    MenuItem,
    /// 进度条
    ProgressBar,
    /// 单选按钮
    RadioButton,
    /// 滚动条
    ScrollBar,
    /// 滑块
    Slider,
    /// 旋转器
    Spinner,
    /// 表格
    Table,
    /// 文本框
    TextBox,
    /// 工具提示
    Tooltip,
    /// 树
    Tree,
    /// 树节点
    TreeItem,
    /// 标签页
    Tab,
    /// 标签栏
    TabList,
    /// 标签面板
    TabPanel,
    /// 自定义角色
    Custom,
}

impl Default for AriaRole {
    fn default() -> Self {
        Self::None
    }
}

/// 无障碍节点
///
/// 描述 UI 节点的无障碍语义信息，
/// 用于屏幕阅读器和其他辅助技术。
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityNode {
    /// 无障碍角色
    pub role: AriaRole,
    /// 可访问名称（用于屏幕阅读器朗读）
    pub label: Option<String>,
    /// 详细描述
    pub description: Option<String>,
    /// 是否启用
    pub enabled: bool,
    /// 是否可见
    pub visible: bool,
    /// 是否可聚焦
    pub focusable: bool,
    /// 是否展开（用于树节点、菜单等）
    pub expanded: Option<bool>,
    /// 是否选中（用于复选框、单选按钮等）
    pub checked: Option<bool>,
    /// 当前值（用于滑块、进度条等）
    pub value: Option<f64>,
    /// 值范围最小值
    pub value_min: Option<f64>,
    /// 值范围最大值
    pub value_max: Option<f64>,
    /// 值文本描述
    pub value_text: Option<String>,
}

impl Default for AccessibilityNode {
    fn default() -> Self {
        Self {
            role: AriaRole::default(),
            label: None,
            description: None,
            enabled: true,
            visible: true,
            focusable: false,
            expanded: None,
            checked: None,
            value: None,
            value_min: None,
            value_max: None,
            value_text: None,
        }
    }
}

impl AccessibilityNode {
    /// 创建新的无障碍节点
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建指定角色的无障碍节点
    pub fn with_role(role: AriaRole) -> Self {
        Self { role, ..Self::default() }
    }

    /// 设置可访问名称
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// 设置是否可聚焦
    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

/// 无障碍树节点
///
/// 无障碍树中的节点，包含节点 ID 和无障碍属性。
#[derive(Debug, Clone)]
pub struct AccessibilityTreeNode {
    /// 对应的 UI 节点 ID
    pub node_id: UiNodeId,
    /// 无障碍属性
    pub accessibility: AccessibilityNode,
    /// 子节点列表
    pub children: Vec<AccessibilityTreeNode>,
}

/// 无障碍树
///
/// 从 UI 节点树构建的语义树，用于屏幕阅读器导航。
/// 仅包含具有语义信息的节点（角色非 None 或有标签的节点）。
#[derive(Debug, Clone)]
pub struct AccessibilityTree {
    /// 根节点
    pub root: Option<AccessibilityTreeNode>,
}

impl AccessibilityTree {
    /// 创建空的无障碍树
    pub fn new() -> Self {
        Self { root: None }
    }

    /// 从 UI 节点树构建无障碍树
    ///
    /// 遍历 UI 树中的所有节点，将具有语义信息的节点
    /// 转换为无障碍树节点，保持树形结构。
    pub fn build_from(tree: &UiTree) -> Self {
        let root_id = match tree.root() {
            Some(id) => id,
            None => return Self { root: None },
        };

        Self { root: Self::build_node(tree, root_id) }
    }

    /// 递归构建无障碍树节点
    fn build_node(tree: &UiTree, node_id: UiNodeId) -> Option<AccessibilityTreeNode> {
        let node = tree.get(node_id)?;

        let children: Vec<AccessibilityTreeNode> = node
            .children
            .iter()
            .filter_map(|&child_id| Self::build_node(tree, child_id))
            .collect();

        let acc = &node.accessibility;
        let has_semantics = acc.role != AriaRole::None
            || acc.label.is_some()
            || acc.description.is_some()
            || acc.focusable;

        if has_semantics || !children.is_empty() {
            Some(AccessibilityTreeNode {
                node_id,
                accessibility: acc.clone(),
                children,
            })
        }
        else {
            None
        }
    }

    /// 查找指定节点 ID 的无障碍节点
    pub fn find(&self, node_id: UiNodeId) -> Option<&AccessibilityTreeNode> {
        Self::find_in_node(self.root.as_ref()?, node_id)
    }

    /// 递归查找无障碍节点
    fn find_in_node(node: &AccessibilityTreeNode, target_id: UiNodeId) -> Option<&AccessibilityTreeNode> {
        if node.node_id == target_id {
            return Some(node);
        }
        for child in &node.children {
            if let Some(found) = Self::find_in_node(child, target_id) {
                return Some(found);
            }
        }
        None
    }

    /// 收集所有可聚焦的节点 ID
    pub fn focusable_nodes(&self) -> Vec<UiNodeId> {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            Self::collect_focusable(root, &mut result);
        }
        result
    }

    /// 递归收集可聚焦节点
    fn collect_focusable(node: &AccessibilityTreeNode, result: &mut Vec<UiNodeId>) {
        if node.accessibility.focusable {
            result.push(node.node_id);
        }
        for child in &node.children {
            Self::collect_focusable(child, result);
        }
    }
}

impl Default for AccessibilityTree {
    fn default() -> Self {
        Self::new()
    }
}

impl UiNode {
    /// 设置节点样式并递增样式版本号
    ///
    /// 当样式发生变化时调用此方法，会自动递增 `style_version`
    /// 以便布局缓存能够检测到样式变更并使对应缓存失效。
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
        self.style_version += 1;
    }
}
