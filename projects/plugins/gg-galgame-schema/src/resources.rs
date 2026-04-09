//! GG Galgame Schema 资源类型模块
//! 定义 Galgame 引擎所需的资源类型

use crate::components::{PortraitState, VariableValue};
use gg_ecs::{Component, Resource};
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

impl Component for DialogueHistory {}

/// 游戏变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVariables {
    /// 变量映射
    pub variables: HashMap<String, VariableValue>,
}

impl Component for GameVariables {}

impl GameVariables {
    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
    }

    /// 设置变量值
    pub fn set_variable(&mut self, name: String, value: VariableValue) {
        self.variables.insert(name, value);
    }

    /// 评估条件表达式
    ///
    /// 支持简单条件格式：`variable_name >= value`、`variable_name <= value`、
    /// `variable_name > value`、`variable_name < value`、`variable_name == value`、
    /// `variable_name != value`
    pub fn evaluate_condition(&self, expression: &str) -> bool {
        let trimmed = expression.trim();

        let (var_name, operator, value_str) = if let Some(idx) = trimmed.find(">=") {
            (&trimmed[..idx], ">=", &trimmed[idx + 2..])
        } else if let Some(idx) = trimmed.find("<=") {
            (&trimmed[..idx], "<=", &trimmed[idx + 2..])
        } else if let Some(idx) = trimmed.find("!=") {
            (&trimmed[..idx], "!=", &trimmed[idx + 2..])
        } else if let Some(idx) = trimmed.find("==") {
            (&trimmed[..idx], "==", &trimmed[idx + 2..])
        } else if let Some(idx) = trimmed.find('>') {
            (&trimmed[..idx], ">", &trimmed[idx + 1..])
        } else if let Some(idx) = trimmed.find('<') {
            (&trimmed[..idx], "<", &trimmed[idx + 1..])
        } else {
            return false;
        };

        let var_name = var_name.trim();
        let value_str = value_str.trim();

        let var_value = match self.variables.get(var_name) {
            Some(v) => v,
            None => return false,
        };

        match operator {
            "==" => Self::compare_equal(var_value, value_str),
            "!=" => !Self::compare_equal(var_value, value_str),
            ">=" => Self::compare_order(var_value, value_str).map_or(false, |o| o == std::cmp::Ordering::Greater || o == std::cmp::Ordering::Equal),
            "<=" => Self::compare_order(var_value, value_str).map_or(false, |o| o == std::cmp::Ordering::Less || o == std::cmp::Ordering::Equal),
            ">" => Self::compare_order(var_value, value_str).map_or(false, |o| o == std::cmp::Ordering::Greater),
            "<" => Self::compare_order(var_value, value_str).map_or(false, |o| o == std::cmp::Ordering::Less),
            _ => false,
        }
    }

    fn compare_equal(var: &VariableValue, value_str: &str) -> bool {
        match var {
            VariableValue::Boolean(b) => value_str.parse::<bool>().map_or(false, |v| *b == v),
            VariableValue::Integer(i) => value_str.parse::<i64>().map_or(false, |v| *i == v),
            VariableValue::Float(f) => value_str.parse::<f64>().map_or(false, |v| (*f - v).abs() < f64::EPSILON),
            VariableValue::String(s) => *s == value_str,
        }
    }

    fn compare_order(var: &VariableValue, value_str: &str) -> Option<std::cmp::Ordering> {
        match var {
            VariableValue::Integer(i) => value_str.parse::<i64>().ok().map(|v| i.cmp(&v)),
            VariableValue::Float(f) => value_str.parse::<f64>().ok().map(|v| f.partial_cmp(&v)).flatten(),
            _ => None,
        }
    }
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

impl Resource for WaitTimer {}
