//! 剧本编译转换器模块
//! 提供剧本编译、验证和数据库构建功能

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use gg_core::{GError, GErrorKind, GResult};
use gg_galgame_schema::components::DialogueNode;
use serde::{Deserialize, Serialize};

use crate::parser::GscriptParser;

/// 剧本序列，表示一个 .gscript 文件编译后的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySequence {
    /// 按顺序排列的对话节点列表
    pub nodes: Vec<DialogueNode>,
    /// 节点 ID 到索引的映射
    pub node_index: HashMap<String, usize>,
}

impl StorySequence {
    /// 根据节点 ID 查找对话节点
    pub fn get_node(&self, id: &str) -> Option<&DialogueNode> {
        self.node_index.get(id).map(|&idx| &self.nodes[idx])
    }

    /// 获取节点数量
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// 判断序列是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// 对话数据库，包含所有编译后的剧本序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueDB {
    /// 文件名到序列的映射
    pub sequences: HashMap<String, StorySequence>,
    /// 所有节点 ID 集合
    pub all_node_ids: HashSet<String>,
}

impl DialogueDB {
    /// 创建空的对话数据库
    pub fn new() -> Self {
        Self { sequences: HashMap::new(), all_node_ids: HashSet::new() }
    }

    /// 根据文件名查找剧本序列
    pub fn get_sequence(&self, file_name: &str) -> Option<&StorySequence> {
        self.sequences.get(file_name)
    }

    /// 判断指定节点 ID 是否存在
    pub fn contains_node(&self, node_id: &str) -> bool {
        self.all_node_ids.contains(node_id)
    }
}

impl Default for DialogueDB {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证错误枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// 目标节点不存在
    MissingNode {
        /// 引用目标节点的源节点 ID
        source_node: String,
        /// 不存在的目标节点 ID
        target_node: String,
    },
    /// 节点 ID 重复
    DuplicateNodeId {
        /// 重复的节点 ID
        id: String,
    },
    /// 空节点（无文本、无命令、无选项、无跳转）
    EmptyNode {
        /// 空节点的 ID
        id: String,
    },
    /// 解析错误
    ParseError {
        /// 错误行号
        line: usize,
        /// 错误消息
        message: String,
    },
}

/// 剧本编译器，将 .gscript 文件编译为结构化的对话数据
pub struct ScriptCompiler;

impl ScriptCompiler {
    /// 编译单个 .gscript 文件
    ///
    /// 解析文件并构建 StorySequence，同时建立节点索引。
    pub fn compile_file(path: &Path) -> GResult<StorySequence> {
        let nodes = GscriptParser::parse_file(path)?;

        let mut node_index = HashMap::new();
        for (idx, node) in nodes.iter().enumerate() {
            node_index.insert(node.id.clone(), idx);
        }

        Ok(StorySequence { nodes, node_index })
    }

    /// 编译目录下所有 .gscript 文件
    ///
    /// 遍历目录中所有 .gscript 扩展名的文件，编译为 StorySequence，
    /// 并汇总构建 DialogueDB。
    pub fn compile_directory(dir: &Path) -> GResult<DialogueDB> {
        let mut db = DialogueDB::new();

        let entries = std::fs::read_dir(dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read directory '{}': {}", dir.display(), e),
        })?;

        for entry in entries {
            let entry = entry
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gscript") {
                let sequence = Self::compile_file(&path)?;

                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string();

                for node in &sequence.nodes {
                    db.all_node_ids.insert(node.id.clone());
                }

                db.sequences.insert(file_name, sequence);
            }
        }

        Ok(db)
    }

    /// 验证单个剧本序列的节点引用完整性
    ///
    /// 检查：重复节点 ID、空节点、引用不存在的目标节点。
    pub fn validate(sequence: &StorySequence) -> GResult<Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut seen_ids = HashSet::new();

        for node in &sequence.nodes {
            if seen_ids.contains(&node.id) {
                errors.push(ValidationError::DuplicateNodeId { id: node.id.clone() });
            }
            else {
                seen_ids.insert(node.id.clone());
            }

            if node.text.is_empty() && node.commands.is_empty() && node.choices.is_empty() && node.next_node_id.is_none() {
                errors.push(ValidationError::EmptyNode { id: node.id.clone() });
            }

            if let Some(ref target) = node.next_node_id {
                if !seen_ids.contains(target) && !sequence.node_index.contains_key(target) {
                    errors.push(ValidationError::MissingNode { source_node: node.id.clone(), target_node: target.clone() });
                }
            }

            for choice in &node.choices {
                if !seen_ids.contains(&choice.next_node_id) && !sequence.node_index.contains_key(&choice.next_node_id) {
                    errors.push(ValidationError::MissingNode {
                        source_node: node.id.clone(),
                        target_node: choice.next_node_id.clone(),
                    });
                }
            }
        }

        Ok(errors)
    }

    /// 验证整个对话数据库的节点引用完整性
    ///
    /// 对每个序列执行验证，并检查跨文件的节点引用。
    pub fn validate_db(db: &DialogueDB) -> GResult<Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (_file_name, sequence) in &db.sequences {
            let seq_errors = Self::validate(sequence)?;
            errors.extend(seq_errors);
        }

        for (_file_name, sequence) in &db.sequences {
            for node in &sequence.nodes {
                if let Some(ref target) = node.next_node_id {
                    if !db.all_node_ids.contains(target) {
                        errors.push(ValidationError::MissingNode { source_node: node.id.clone(), target_node: target.clone() });
                    }
                }

                for choice in &node.choices {
                    if !db.all_node_ids.contains(&choice.next_node_id) {
                        errors.push(ValidationError::MissingNode {
                            source_node: node.id.clone(),
                            target_node: choice.next_node_id.clone(),
                        });
                    }
                }
            }
        }

        Ok(errors)
    }
}
