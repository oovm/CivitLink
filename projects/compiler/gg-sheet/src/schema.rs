//! 配置表结构定义模块
//! 定义表头、表格类型和结构化表格数据

use crate::{
    error::{SheetError, SheetResult},
    reader::RawTable,
    types::SheetType,
};

/// 字段约束类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldConstraint {
    /// 无约束
    None,
    /// 唯一约束（@field）
    Unique,
    /// 主键约束（@@field）
    Primary,
}

/// 配置表字段头信息
#[derive(Debug, Clone)]
pub struct SheetHeader {
    /// 列索引（从 0 开始）
    pub column: usize,
    /// 字段名
    pub field_name: String,
    /// 字段类型
    pub typing: SheetType,
    /// 字段注释
    pub comment: String,
    /// 字段约束
    pub constraint: FieldConstraint,
    /// 字段验证规则列表
    pub validation_rules: Vec<crate::types::ValidationRule>,
    /// 字段默认值
    pub default_value: Option<String>,
}

/// 表格类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableKind {
    /// 列表表（首列字段名为 id，数字键）
    List,
    /// 字典表（首列字段名为 key，字符串键）
    Dict,
    /// 枚举表（首列字段名为 enum）
    Enumerate,
    /// 单例配置表（首列字段名以 @class 开头）
    Class,
    /// 多语言表（首列字段名以 @language 开头）
    Language,
}

impl TableKind {
    /// 根据首列原始字段名自动检测表格类型
    ///
    /// 支持显式标记：@dict, @list, @enum, @class, @language
    /// 也支持向后兼容的隐式推断
    pub fn detect(raw_first_field_name: &str) -> TableKind {
        let trimmed = raw_first_field_name.trim();

        if trimmed.starts_with("@class") {
            return TableKind::Class;
        }
        if trimmed.starts_with("@language") {
            return TableKind::Language;
        }
        if trimmed.starts_with("@dict") {
            return TableKind::Dict;
        }
        if trimmed.starts_with("@list") {
            return TableKind::List;
        }
        if trimmed.starts_with("@enum") {
            return TableKind::Enumerate;
        }

        let name = trimmed.trim_start_matches('@');
        match name {
            "id" => TableKind::List,
            "key" => TableKind::Dict,
            "enum" => TableKind::Enumerate,
            _ => TableKind::List,
        }
    }
}

/// 一行数据，以字段名到字符串值的映射表示
pub type DataRow = Vec<String>;

/// 结构化表格数据
#[derive(Debug, Clone)]
pub struct SheetTable {
    /// 表格名称
    pub name: String,
    /// 表格类型
    pub kind: TableKind,
    /// 表头列表
    pub headers: Vec<SheetHeader>,
    /// 数据行列表
    pub rows: Vec<DataRow>,
}

impl SheetTable {
    /// 从原始表格数据构建结构化表格
    pub fn from_raw(raw: RawTable) -> SheetResult<SheetTable> {
        if raw.header_row.is_empty() {
            return Err(SheetError::Parse { path: raw.path.clone(), line: 1, message: "表头行为空".to_string() });
        }

        let kind = TableKind::detect(raw.header_row[0].as_str());

        let mut headers = Vec::new();
        let col_count = raw.header_row.len();

        for col in 0..col_count {
            let field_str = raw.header_row.get(col).map(|s| s.as_str()).unwrap_or("");
            let type_str = raw.type_row.get(col).map(|s| s.as_str()).unwrap_or("");
            let comment = raw.comment_row.get(col).map(|s| s.as_str()).unwrap_or("");

            let parsed_field = parse_field_info(field_str, type_str);
            let typing = SheetType::parse(&parsed_field.clean_type_str).map_err(|_| SheetError::Type {
                path: raw.path.clone(),
                column: col,
                type_str: parsed_field.clean_type_str.clone(),
            })?;

            headers.push(SheetHeader {
                column: col,
                field_name: parsed_field.name,
                typing,
                comment: comment.to_string(),
                constraint: parsed_field.constraint,
                validation_rules: parsed_field.validation_rules,
                default_value: parsed_field.default_value,
            });
        }

        Ok(SheetTable { name: raw.name, kind, headers, rows: raw.data_rows })
    }

    /// 获取主键字段的索引，若无主键则返回 0
    pub fn primary_key_index(&self) -> usize {
        self.headers.iter().find(|h| h.constraint == FieldConstraint::Primary).map(|h| h.column).unwrap_or(0)
    }

    /// 获取枚举表的枚举名列索引（enum 列）
    pub fn enum_name_column(&self) -> Option<usize> {
        if self.kind != TableKind::Enumerate {
            return None;
        }
        self.headers.iter().find(|h| h.field_name == "enum").map(|h| h.column)
    }

    /// 获取枚举表的 ID 列索引
    pub fn enum_id_column(&self) -> Option<usize> {
        if self.kind != TableKind::Enumerate {
            return None;
        }
        self.headers.iter().find(|h| h.field_name == "id").map(|h| h.column)
    }
}

/// 从可能带有表类型标记的字段名中提取实际字段名
///
/// 例如 "@dict key" → "key"，"@list id" → "id"
/// 如果没有标记前缀，则按原有逻辑处理（@/@@ 约束标记）
pub fn extract_field_name(raw_field_name: &str) -> String {
    let trimmed = raw_field_name.trim();

    let type_markers = ["@dict", "@list", "@enum", "@class", "@language"];
    for marker in &type_markers {
        if trimmed.starts_with(marker) {
            let rest = trimmed[marker.len()..].trim();
            return rest.to_string();
        }
    }

    if trimmed.starts_with("@@") {
        return trimmed[2..].to_string();
    }
    if trimmed.starts_with('@') {
        let after_at = &trimmed[1..];
        return after_at.to_string();
    }

    trimmed.to_string()
}

/// 解析后的字段信息
struct ParsedFieldInfo {
    /// 字段名
    name: String,
    /// 约束类型
    constraint: FieldConstraint,
    /// 清洗后的类型字符串（去除注解和默认值）
    clean_type_str: String,
    /// 验证规则列表
    validation_rules: Vec<crate::types::ValidationRule>,
    /// 默认值
    default_value: Option<String>,
}

/// 解析字段名和类型行中的约束标记、验证规则和默认值
fn parse_field_info(field_str: &str, type_str: &str) -> ParsedFieldInfo {
    let trimmed_field = field_str.trim();

    let (name, constraint) = if trimmed_field.starts_with("@@") {
        (trimmed_field[2..].to_string(), FieldConstraint::Primary)
    }
    else if trimmed_field.starts_with('@') {
        let after_at = &trimmed_field[1..];
        let first_word = after_at.split_whitespace().next().unwrap_or("");
        match first_word {
            "dict" | "list" | "enum" | "class" | "language" => {
                let rest = after_at[first_word.len()..].trim();
                (rest.to_string(), FieldConstraint::None)
            }
            _ => (after_at.to_string(), FieldConstraint::Unique),
        }
    }
    else {
        (trimmed_field.to_string(), FieldConstraint::None)
    };

    let (clean_type_str, validation_rules, default_value) = parse_type_annotations(type_str);

    ParsedFieldInfo { name, constraint, clean_type_str, validation_rules, default_value }
}

/// 解析类型行中的验证规则注解和默认值
fn parse_type_annotations(type_str: &str) -> (String, Vec<crate::types::ValidationRule>, Option<String>) {
    let trimmed = type_str.trim();
    let mut rules = Vec::new();
    let mut default_value = None;
    let mut clean_parts = String::new();
    let mut pos = 0;
    let chars: Vec<char> = trimmed.chars().collect();

    while pos < chars.len() {
        if chars[pos] == '=' {
            let rest: String = chars[pos + 1..].iter().collect();
            let rest = rest.trim();
            default_value = Some(rest.to_string());
            break;
        }

        if chars[pos] == '@' {
            let rest: String = chars[pos + 1..].iter().collect();
            if let Some(rule) = try_parse_annotation(&rest) {
                rules.push(rule.0);
                pos += rule.1;
                continue;
            }
        }

        clean_parts.push(chars[pos]);
        pos += 1;
    }

    (clean_parts.trim().to_string(), rules, default_value)
}

/// 尝试解析类型行中的注解标记
fn try_parse_annotation(s: &str) -> Option<(crate::types::ValidationRule, usize)> {
    let s_lower = s.to_lowercase();

    if s_lower.starts_with("min(") {
        if let Some(end) = s.find(')') {
            let inner = &s[4..end];
            if let Ok(v) = inner.trim().parse::<f64>() {
                return Some((crate::types::ValidationRule::Min(v), end + 2));
            }
        }
    }

    if s_lower.starts_with("max(") {
        if let Some(end) = s.find(')') {
            let inner = &s[4..end];
            if let Ok(v) = inner.trim().parse::<f64>() {
                return Some((crate::types::ValidationRule::Max(v), end + 2));
            }
        }
    }

    if s_lower.starts_with("min_length(") {
        if let Some(end) = s.find(')') {
            let inner = &s[11..end];
            if let Ok(v) = inner.trim().parse::<usize>() {
                return Some((crate::types::ValidationRule::MinLength(v), end + 2));
            }
        }
    }

    if s_lower.starts_with("max_length(") {
        if let Some(end) = s.find(')') {
            let inner = &s[11..end];
            if let Ok(v) = inner.trim().parse::<usize>() {
                return Some((crate::types::ValidationRule::MaxLength(v), end + 2));
            }
        }
    }

    if s_lower.starts_with("pattern(") {
        if let Some(end) = s.find(')') {
            let inner = &s[8..end];
            let pattern = inner.trim().trim_matches('"').trim_matches('\'');
            return Some((crate::types::ValidationRule::Pattern(pattern.to_string()), end + 2));
        }
    }

    if s_lower.starts_with("required") {
        return Some((crate::types::ValidationRule::Required, 9));
    }

    if s_lower.starts_with("custom(") {
        if let Some(end) = s.find(')') {
            let inner = &s[7..end];
            let name = inner.trim().trim_matches('"').trim_matches('\'');
            if !name.is_empty() {
                return Some((crate::types::ValidationRule::Custom(name.to_string()), end + 2));
            }
        }
    }

    None
}
