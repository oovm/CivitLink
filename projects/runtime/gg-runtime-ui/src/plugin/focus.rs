use std::collections::HashMap;

use gg_ui::UiNodeId;

/// 焦点管理器
///
/// 管理当前拥有输入焦点的 UI 节点，
/// 支持 Tab 键导航和焦点陷阱（用于模态对话框）。
#[derive(Debug, Clone, Default)]
pub struct FocusManager {
    /// 当前拥有焦点的节点 ID
    pub focused_node_id: Option<UiNodeId>,
    /// 按 Tab 顺序排列的可聚焦节点列表
    pub tab_order: Vec<UiNodeId>,
    /// 焦点陷阱节点，设置后 Tab 导航限制在该节点的子树内
    pub focus_trap: Option<UiNodeId>,
    /// 节点到 Tab 索引的映射
    pub tab_index: HashMap<u64, u32>,
    /// 节点到父节点的映射，用于焦点陷阱的子树判断
    parents: HashMap<u64, Option<u64>>,
}

impl FocusManager {
    /// 创建新的焦点管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取焦点
    ///
    /// 将输入焦点转移到指定节点，前一个焦点节点自动失去焦点。
    pub fn acquire_focus(&mut self, node_id: UiNodeId) {
        self.focused_node_id = Some(node_id);
    }

    /// 释放焦点
    ///
    /// 如果指定节点当前拥有焦点，则释放之。
    pub fn release_focus(&mut self, node_id: UiNodeId) {
        if self.focused_node_id == Some(node_id) {
            self.focused_node_id = None;
        }
    }

    /// 检查指定节点是否拥有焦点
    pub fn is_focused(&self, node_id: UiNodeId) -> bool {
        self.focused_node_id == Some(node_id)
    }

    /// 获取当前焦点节点 ID
    pub fn focused(&self) -> Option<UiNodeId> {
        self.focused_node_id
    }

    /// 注册可聚焦节点
    ///
    /// 将节点添加到 Tab 导航序列中，使用指定的 Tab 索引决定顺序。
    pub fn register_focusable(&mut self, node_id: UiNodeId, tab_index: u32) {
        self.tab_index.insert(node_id, tab_index);
        if !self.tab_order.contains(&node_id) {
            self.tab_order.push(node_id);
        }
        self.rebuild_tab_order();
    }

    /// 注销可聚焦节点
    ///
    /// 从 Tab 导航序列中移除指定节点。
    pub fn unregister_focusable(&mut self, node_id: UiNodeId) {
        self.tab_index.remove(&node_id);
        self.tab_order.retain(|&id| id != node_id);
        self.parents.remove(&node_id);
        if self.focused_node_id == Some(node_id) {
            self.focused_node_id = None;
        }
    }

    /// 设置焦点陷阱节点
    ///
    /// 设置后，Tab 导航将限制在该节点的子树范围内，
    /// 适用于模态对话框等场景。传入 None 清除陷阱。
    pub fn set_focus_trap(&mut self, trap_node: Option<UiNodeId>) {
        self.focus_trap = trap_node;
    }

    /// 设置节点的父节点关系
    ///
    /// 用于焦点陷阱的子树判断，需在注册节点时同步设置。
    pub fn set_parent(&mut self, node_id: UiNodeId, parent: Option<UiNodeId>) {
        self.parents.insert(node_id, parent);
    }

    /// 导航到下一个可聚焦节点
    ///
    /// 按 Tab 顺序移动焦点到下一个节点，到达末尾后回到开头。
    /// 如果设置了焦点陷阱，仅在陷阱子树内循环。
    pub fn navigate_next(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }

        let current_idx = self.focused_node_id.and_then(|id| self.tab_order.iter().position(|&x| x == id));

        let len = self.tab_order.len();
        let start_idx = match current_idx {
            Some(idx) if idx + 1 < len => idx + 1,
            _ => 0,
        };

        for i in 0..len {
            let idx = (start_idx + i) % len;
            if let Some(&candidate) = self.tab_order.get(idx) {
                if self.is_within_trap(candidate) {
                    self.focused_node_id = Some(candidate);
                    return;
                }
            }
        }
    }

    /// 导航到上一个可聚焦节点
    ///
    /// 按 Tab 逆序移动焦点到上一个节点，到达开头后跳到末尾。
    /// 如果设置了焦点陷阱，仅在陷阱子树内循环。
    pub fn navigate_prev(&mut self) {
        if self.tab_order.is_empty() {
            return;
        }

        let current_idx = self.focused_node_id.and_then(|id| self.tab_order.iter().position(|&x| x == id));

        let len = self.tab_order.len();
        let start_idx = match current_idx {
            Some(0) => len - 1,
            Some(idx) => idx - 1,
            None => len - 1,
        };

        for i in 0..len {
            let idx = (start_idx + len - i) % len;
            if let Some(&candidate) = self.tab_order.get(idx) {
                if self.is_within_trap(candidate) {
                    self.focused_node_id = Some(candidate);
                    return;
                }
            }
        }
    }

    /// 重建 Tab 顺序
    ///
    /// 根据 tab_index 对 tab_order 进行排序，索引小的排在前面。
    pub fn rebuild_tab_order(&mut self) {
        self.tab_order.sort_by_key(|id| self.tab_index.get(id).copied().unwrap_or(u32::MAX));
    }

    /// 检查节点是否在焦点陷阱范围内
    fn is_within_trap(&self, node_id: UiNodeId) -> bool {
        self.focus_trap.map_or(true, |trap| self.is_descendant_or_self(node_id, trap))
    }

    /// 检查节点是否是目标节点的后代或自身
    ///
    /// 通过 parents 映射向上遍历父节点链进行判断。
    fn is_descendant_or_self(&self, node_id: UiNodeId, ancestor: UiNodeId) -> bool {
        if node_id == ancestor {
            return true;
        }
        let mut current = node_id;
        let mut visited = 0;
        while let Some(Some(parent)) = self.parents.get(&current) {
            if *parent == ancestor {
                return true;
            }
            current = *parent;
            visited += 1;
            if visited > 1000 {
                break;
            }
        }
        false
    }

    /// 方向键导航：向前移动焦点
    ///
    /// 在 Tab 顺序中向前移动焦点（对应向下/向右方向键），
    /// 如果设置了焦点陷阱，仅在陷阱子树内循环。
    pub fn navigate_arrow_next(&mut self) {
        self.navigate_next();
    }

    /// 方向键导航：向后移动焦点
    ///
    /// 在 Tab 顺序中向后移动焦点（对应向上/向左方向键），
    /// 如果设置了焦点陷阱，仅在陷阱子树内循环。
    pub fn navigate_arrow_prev(&mut self) {
        self.navigate_prev();
    }

    /// 方向键导航：根据方向移动焦点
    ///
    /// 根据方向键的方向在 Tab 顺序中移动焦点：
    /// - 下/右：向前移动
    /// - 上/左：向后移动
    pub fn navigate_arrow(&mut self, direction: ArrowDirection) {
        match direction {
            ArrowDirection::Down | ArrowDirection::Right => self.navigate_arrow_next(),
            ArrowDirection::Up | ArrowDirection::Left => self.navigate_arrow_prev(),
        }
    }
}

/// 方向键方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    /// 上方向键
    Up,
    /// 下方向键
    Down,
    /// 左方向键
    Left,
    /// 右方向键
    Right,
}
