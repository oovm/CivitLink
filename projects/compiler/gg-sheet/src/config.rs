//! GG-Sheet 项目配置模块
//! 提供从 sheet.config (VON 格式) 加载配置的功能

use std::path::Path;

use crate::error::{SheetError, SheetResult};

/// GG-Sheet 项目配置
#[derive(Debug, Clone)]
pub struct SheetConfig {
    /// 配置表目录，默认 "asset/sheet"
    pub sheet_dir: String,
    /// 输出目录，默认 "asset/script/table"
    pub output_dir: String,
    /// 延迟加载阈值，行数超过此值的表使用延迟加载，None 表示不启用
    pub lazy_load_threshold: Option<usize>,
}

impl Default for SheetConfig {
    fn default() -> Self {
        Self { sheet_dir: "asset/sheet".to_string(), output_dir: "asset/script/table".to_string(), lazy_load_threshold: None }
    }
}

impl SheetConfig {
    /// 从指定路径加载配置文件
    pub fn load(path: &Path) -> SheetResult<SheetConfig> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SheetError::Io { path: path.to_path_buf(), message: format!("无法读取配置文件: {}", e) })?;

        Ok(parse_von_sheet_config(&content))
    }

    /// 从工作区目录加载 sheet.config 配置文件
    pub fn load_from_workspace(workspace: &Path) -> SheetResult<SheetConfig> {
        let config_path = workspace.join("sheet.config");

        if !config_path.exists() {
            return Ok(SheetConfig::default());
        }

        Self::load(&config_path)
    }
}

/// 从 VON 格式文本中解析 sheet 配置
fn parse_von_sheet_config(content: &str) -> SheetConfig {
    let mut config = SheetConfig::default();

    let sheet_block = extract_sheet_block(content);
    if let Some(block) = sheet_block {
        if let Some(value) = extract_field(&block, "sheet_dir") {
            config.sheet_dir = value;
        }
        if let Some(value) = extract_field(&block, "output_dir") {
            config.output_dir = value;
        }
        if let Some(value) = extract_field(&block, "lazy_load_threshold") {
            if let Ok(threshold) = value.parse::<usize>() {
                config.lazy_load_threshold = Some(threshold);
            }
        }
    }

    config
}

/// 从 VON 内容中提取 sheet 块
/// 支持 `sheet: SheetConfig { ... }` 和 `sheet: { ... }` 两种格式
fn extract_sheet_block(content: &str) -> Option<String> {
    let content = content.trim_start();

    for prefix in &["sheet: SheetConfig", "sheet:"] {
        if let Some(pos) = content.find(prefix) {
            let after_prefix = &content[pos + prefix.len()..];
            let after_prefix = after_prefix.trim_start();

            if let Some(braced) = extract_braced_block(after_prefix) {
                return Some(braced);
            }
        }
    }

    None
}

/// 从以 `{` 开头的文本中提取花括号内的内容
fn extract_braced_block(content: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with('{') {
        return None;
    }

    let mut depth = 0;
    let mut start = None;
    let chars: Vec<char> = content.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(i + 1);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        return Some(chars[s..i].iter().collect());
                    }
                }
            }
            _ => {}
        }
    }

    None
}

/// 从块内容中提取指定字段的值
fn extract_field(block: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{}:", field_name);

    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&pattern) {
            let rest = rest.trim();
            let value = if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
                &rest[1..rest.len() - 1]
            }
            else {
                let end = rest.find(|c: char| c.is_whitespace() || c == ',' || c == '}');
                match end {
                    Some(pos) => &rest[..pos],
                    None => rest,
                }
            };
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}
