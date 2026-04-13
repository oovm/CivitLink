#![warn(missing_docs)]

//! GG 引擎角色资源模块
//! 提供通用的角色相关资源定义

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

/// 变量值枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// 字符串
    String(String),
}

/// 游戏变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVariables {
    /// 变量映射
    pub variables: HashMap<String, VariableValue>,
}

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
        }
        else if let Some(idx) = trimmed.find("<=") {
            (&trimmed[..idx], "<=", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find("!=") {
            (&trimmed[..idx], "!=", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find("==") {
            (&trimmed[..idx], "==", &trimmed[idx + 2..])
        }
        else if let Some(idx) = trimmed.find('>') {
            (&trimmed[..idx], ">", &trimmed[idx + 1..])
        }
        else if let Some(idx) = trimmed.find('<') {
            (&trimmed[..idx], "<", &trimmed[idx + 1..])
        }
        else {
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
            ">=" => Self::compare_order(var_value, value_str)
                .map_or(false, |o| o == std::cmp::Ordering::Greater || o == std::cmp::Ordering::Equal),
            "<=" => Self::compare_order(var_value, value_str)
                .map_or(false, |o| o == std::cmp::Ordering::Less || o == std::cmp::Ordering::Equal),
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

/// 帧间隔时间资源
///
/// 存储当前帧与上一帧之间的时间间隔，供系统使用真实时间更新。
#[derive(Debug, Clone, Copy)]
pub struct DeltaTime {
    /// 帧间隔时间（秒）
    pub secs: f32,
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self { secs: 1.0 / 60.0 }
    }
}

/// 等待计时器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitTimer {
    /// 剩余等待时间（秒）
    pub remaining_secs: f32,
}

/// 游戏状态资源
///
/// 管理全局游戏状态，包括变量、标志、天数、时间和游戏结束标志。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// 游戏变量映射
    pub variables: HashMap<String, VariableValue>,
    /// 布尔标志集合
    pub flags: std::collections::HashSet<String>,
    /// 当前天数
    pub day: i32,
    /// 当前时间
    pub time: f32,
    /// 游戏是否结束
    pub game_over: bool,
}

impl GameState {
    /// 创建新的游戏状态
    pub fn new() -> Self {
        Self { variables: HashMap::new(), flags: std::collections::HashSet::new(), day: 1, time: 0.0, game_over: false }
    }

    /// 设置布尔标志
    pub fn set_flag(&mut self, flag: String) {
        self.flags.insert(flag);
    }

    /// 检查布尔标志是否存在
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }

    /// 清除布尔标志
    pub fn clear_flag(&mut self, flag: &str) {
        self.flags.remove(flag);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
