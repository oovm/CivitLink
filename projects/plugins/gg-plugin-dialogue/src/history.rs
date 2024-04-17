//! 对话历史管理模块
//! 提供对话历史的添加、查询和清空功能

use crate::schema::{DialogueHistory, HistoryEntry};

/// 对话历史管理器
///
/// 提供对 DialogueHistory 的操作方法，包括添加条目、获取条目和清空历史。
pub struct DialogueHistoryManager;

impl DialogueHistoryManager {
    /// 添加历史条目
    ///
    /// 将一条对话记录添加到历史中，包含说话者名称、对话文本和时间戳。
    pub fn add_entry(history: &mut DialogueHistory, speaker_name: Option<String>, text: String, timestamp: f64) {
        history.entries.push(HistoryEntry { speaker_name, text, timestamp });
    }

    /// 获取所有历史条目
    ///
    /// 返回对话历史中所有条目的切片引用。
    pub fn get_entries(history: &DialogueHistory) -> &[HistoryEntry] {
        &history.entries
    }

    /// 清空历史
    ///
    /// 清除对话历史中的所有条目，并重置当前节点 ID。
    pub fn clear(history: &mut DialogueHistory) {
        history.entries.clear();
        history.current_node_id = None;
    }
}
