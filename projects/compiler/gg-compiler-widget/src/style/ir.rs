//! Style IR 定义模块

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::artifact::RuleId;

/// Style IR 根节点
#[derive(Debug, Clone)]
pub struct StyleIr {
    /// 解析后的样式规则
    pub rules: Vec<StyleRuleIr>,
    /// 选择器优先级映射
    pub specificity_map: HashMap<String, u32>,
    /// 主题变量解析结果
    pub resolved_vars: HashMap<String, StyleValue>,
}

/// 样式规则 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRuleIr {
    /// 规则 ID
    pub id: RuleId,
    /// 解析后的选择器
    pub selector: SelectorIr,
    /// 解析后的声明
    pub declarations: HashMap<String, ResolvedValue>,
    /// 计算后的优先级
    pub specificity: u32,
}

/// 选择器 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorIr {
    /// 选择器类型
    pub selector_type: SelectorType,
    /// 选择器文本
    pub text: String,
}

/// 选择器类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectorType {
    /// 类选择器
    Class,
    /// ID 选择器
    Id,
    /// 标签选择器
    Tag,
    /// 复合选择器
    Compound,
}

/// 解析后的值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedValue {
    /// 值数据
    pub data: Vec<u8>,
}

/// 样式值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleValue {
    /// 颜色值
    Color(String),
    /// 长度值
    Length(f32),
    /// 百分比值
    Percentage(f32),
    /// 数值
    Number(f32),
    /// 字符串值
    String(String),
    /// 枚举值
    Enum(String),
}
