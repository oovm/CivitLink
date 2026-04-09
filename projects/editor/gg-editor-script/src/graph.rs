//! 节点图数据结构

/// 节点图条目
#[derive(Debug, Clone)]
pub struct NodeGraphEntry {
    /// 节点 ID
    pub id: String,
    /// 节点在图中的位置
    pub position: (f32, f32),
    /// 连接的目标节点 ID
    pub connections: Vec<String>,
}

/// 节点图数据结构
pub struct NodeGraph {
    /// 节点条目
    entries: Vec<NodeGraphEntry>,
}

impl NodeGraph {
    /// 创建新的节点图
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, id: String, position: (f32, f32)) {
        self.entries.push(NodeGraphEntry {
            id,
            position,
            connections: Vec::new(),
        });
    }

    /// 移除节点
    pub fn remove_node(&mut self, id: &str) {
        self.entries.retain(|e| e.id != id);
        for entry in &mut self.entries {
            entry.connections.retain(|c| c != id);
        }
    }

    /// 添加连接
    pub fn add_connection(&mut self, from_id: &str, to_id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == from_id) {
            if !entry.connections.contains(&to_id.to_string()) {
                entry.connections.push(to_id.to_string());
            }
        }
    }

    /// 移除连接
    pub fn remove_connection(&mut self, from_id: &str, to_id: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == from_id) {
            entry.connections.retain(|c| c != to_id);
        }
    }

    /// 获取节点
    pub fn get_node(&self, id: &str) -> Option<&NodeGraphEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// 获取可变节点
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut NodeGraphEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// 获取所有节点条目
    pub fn entries(&self) -> &[NodeGraphEntry] {
        &self.entries
    }

    /// 自动布局（简单的从左到右分层布局）
    pub fn layout_auto(&mut self) {
        let horizontal_spacing: f32 = 300.0;
        let vertical_spacing: f32 = 150.0;
        let max_per_row = 4;

        let count = self.entries.len();
        for (i, entry) in self.entries.iter_mut().enumerate() {
            let column = i % max_per_row;
            let row = i / max_per_row;
            entry.position = (
                column as f32 * horizontal_spacing,
                row as f32 * vertical_spacing,
            );
        }

        let _ = count;
    }
}

impl Default for NodeGraph {
    fn default() -> Self {
        Self::new()
    }
}
