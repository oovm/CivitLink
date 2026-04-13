//! GG Galgame Schema 资源类型模块
//!
//! 定义 Galgame 引擎所需的资源类型。

use crate::components::{CharacterDef, DialogueNode, PortraitState};
use pleroma::{DeltaTime, GameVariables, VariableValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 说话者名称
    pub speaker_name: Option<String>,
    /// 对话文本
    pub text: String,
    /// 时间戳
    pub timestamp: f64,
}

/// 对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueHistory {
    /// 历史条目列表
    pub entries: Vec<HistoryEntry>,
    /// 当前对话节点 ID
    pub current_node_id: Option<String>,
}

/// 存档数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// 当前对话节点
    pub current_node_id: String,
    /// 游戏变量快照
    pub variables: HashMap<String, VariableValue>,
    /// 立绘状态快照
    pub portrait_states: Vec<PortraitState>,
    /// 当前背景路径
    pub background_path: Option<String>,
    /// 当前 BGM 路径
    pub bgm_path: Option<String>,
    /// 截图数据
    pub screenshot: Option<Vec<u8>>,
    /// 保存时间戳
    pub timestamp: f64,
}

/// 等待计时器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitTimer {
    /// 剩余等待时间（秒）
    pub remaining_secs: f32,
}

/// 对话脚本
///
/// 从 JSON 文件加载的对话数据，包含节点、角色和变量定义。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueScript {
    /// 对话节点列表
    pub nodes: Vec<DialogueNode>,
    /// 角色定义列表
    pub characters: Vec<CharacterDef>,
    /// 初始变量值
    pub variables: HashMap<String, VariableValue>,
}

impl DialogueScript {
    /// 从 JSON 字符串解析对话脚本
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}
