//! 剧本编辑器面板

use gg_core::{GError, GErrorKind, GResult};
use gg_galgame_schema::components::DialogueNode;
use gg_editor_shell::panel::{EditorPanel, PanelContext, PanelData};

use crate::graph::{NodeGraph, NodeGraphEntry};
use crate::templates::ScriptTemplate;

/// 剧本编辑器面板
pub struct ScriptEditorPanel {
    /// 可见性
    visible: bool,
    /// 节点图条目列表
    nodes: Vec<NodeGraphEntry>,
    /// 选中的节点 ID
    selected_node_id: Option<String>,
    /// 搜索关键词
    search_query: String,
    /// 滚动偏移
    scroll_offset: (f32, f32),
    /// 正在编辑的节点
    editing_node: Option<DialogueNode>,
    /// 内部节点图
    graph: NodeGraph,
}

impl ScriptEditorPanel {
    /// 创建面板
    pub fn new() -> Self {
        Self {
            visible: true,
            nodes: Vec::new(),
            selected_node_id: None,
            search_query: String::new(),
            scroll_offset: (0.0, 0.0),
            editing_node: None,
            graph: NodeGraph::new(),
        }
    }

    /// 同步节点列表与内部节点图
    fn sync_nodes(&mut self) {
        self.nodes = self.graph.entries().to_vec();
    }

    /// 创建新对话节点
    pub fn create_node(&mut self, context: &mut PanelData) -> GResult<()> {
        let world = context.world().ok_or_else(|| GError {
            kind: GErrorKind::Other,
            message: "World not available".to_string(),
        })?;

        let entity = world.spawn();
        let node_id = format!("node_{}", entity);

        let node = DialogueNode {
            id: node_id.clone(),
            speaker_id: None,
            text: String::new(),
            commands: Vec::new(),
            choices: Vec::new(),
            next_node_id: None,
        };

        world.add_component(entity, node.clone())?;

        let position = (
            self.scroll_offset.0 + 100.0,
            self.scroll_offset.1 + 100.0,
        );

        self.graph.add_node(node_id.clone(), position);
        self.sync_nodes();
        self.selected_node_id = Some(node_id);
        self.editing_node = Some(node);

        Ok(())
    }

    /// 删除节点
    pub fn delete_node(&mut self, node_id: &str, context: &mut PanelData) -> GResult<()> {
        let _ = context;

        self.graph.remove_node(node_id);
        self.sync_nodes();

        if self.selected_node_id.as_deref() == Some(node_id) {
            self.selected_node_id = None;
            self.editing_node = None;
        }

        Ok(())
    }

    /// 连接两个节点
    pub fn connect_nodes(&mut self, from_id: &str, to_id: &str) {
        self.graph.add_connection(from_id, to_id);
        self.sync_nodes();
    }

    /// 搜索节点
    pub fn search_nodes(&self, query: &str) -> Vec<&NodeGraphEntry> {
        if query.is_empty() {
            return self.nodes.iter().collect();
        }
        self.nodes
            .iter()
            .filter(|e| e.id.contains(query))
            .collect()
    }

    /// 获取搜索关键词
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// 设置搜索关键词
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// 获取滚动偏移
    pub fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset
    }

    /// 设置滚动偏移
    pub fn set_scroll_offset(&mut self, offset: (f32, f32)) {
        self.scroll_offset = offset;
    }

    /// 获取正在编辑的节点
    pub fn editing_node(&self) -> Option<&DialogueNode> {
        self.editing_node.as_ref()
    }

    /// 设置正在编辑的节点
    pub fn set_editing_node(&mut self, node: Option<DialogueNode>) {
        self.editing_node = node;
    }

    /// 获取选中的节点 ID
    pub fn selected_node_id(&self) -> Option<&str> {
        self.selected_node_id.as_deref()
    }

    /// 获取节点列表
    pub fn nodes(&self) -> &[NodeGraphEntry] {
        &self.nodes
    }

    /// 自动布局节点图
    pub fn auto_layout(&mut self) {
        self.graph.layout_auto();
        self.sync_nodes();
    }

    /// 插入模板
    pub fn insert_template(&mut self, template: ScriptTemplate, context: &mut PanelData) -> GResult<()> {
        let nodes = match template {
            ScriptTemplate::DailyConversation => crate::templates::generate_daily_conversation(),
            ScriptTemplate::ConfessionScene => crate::templates::generate_confession_scene(),
            ScriptTemplate::BattleNarration => crate::templates::generate_battle_narration(),
        };

        let world = context.world().ok_or_else(|| GError {
            kind: GErrorKind::Other,
            message: "World not available".to_string(),
        })?;

        for (i, node) in nodes.into_iter().enumerate() {
            let entity = world.spawn();
            world.add_component(entity, node.clone())?;

            let position = (
                self.scroll_offset.0 + i as f32 * 300.0,
                self.scroll_offset.1,
            );

            self.graph.add_node(node.id.clone(), position);

            if let Some(next_id) = &node.next_node_id {
                self.graph.add_connection(&node.id, next_id);
            }

            for choice in &node.choices {
                self.graph.add_connection(&node.id, &choice.next_node_id);
            }
        }

        self.graph.layout_auto();
        self.sync_nodes();

        Ok(())
    }
}

impl Default for ScriptEditorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for ScriptEditorPanel {
    fn name(&self) -> &str {
        "Script Editor"
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn render(&mut self, _context: &mut PanelContext) -> GResult<()> {
        Ok(())
    }
}
