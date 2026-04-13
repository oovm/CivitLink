//! Widget 文件解析器模块

use crate::error::{WidgetError, WidgetResult};

/// Widget 文件解析结果
pub struct WidgetFile {
    /// 模板部分内容
    pub template: Option<String>,
    /// 脚本部分内容
    pub script: Option<String>,
    /// 样式部分内容
    pub style: Option<String>,
}

/// Widget 文件解析器
pub struct WidgetParser;

impl WidgetParser {
    /// 解析 .widget 文件内容为 WidgetFile 结构
    pub fn parse(content: &str) -> WidgetResult<WidgetFile> {
        let template = Self::extract_section(content, "template")?;
        let script = Self::extract_section(content, "script")?;
        let style = Self::extract_section(content, "style")?;

        if template.is_none() && script.is_none() && style.is_none() {
            return Err(WidgetError::ParseError("No valid section found in .widget file".to_string()));
        }

        Ok(WidgetFile { template, script, style })
    }

    /// 从内容中提取指定标签的区块内容
    fn extract_section(content: &str, tag_name: &str) -> WidgetResult<Option<String>> {
        let opening_tag = format!("<{}>", tag_name);
        let closing_tag = format!("</{}>", tag_name);
        let tag_prefix = format!("<{}", tag_name);

        let open_pos = match content.find(&opening_tag) {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let content_start = open_pos + opening_tag.len();
        let mut depth: usize = 1;
        let mut search_from = content_start;

        while depth > 0 {
            let next_open = Self::find_tag_open(content, &tag_prefix, search_from);
            let next_close = content[search_from..].find(&closing_tag).map(|p| search_from + p);

            match (next_open, next_close) {
                (Some(op), Some(cp)) => {
                    if op < cp {
                        depth += 1;
                        search_from = op + tag_prefix.len();
                    }
                    else {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(Some(content[content_start..cp].to_string()));
                        }
                        search_from = cp + closing_tag.len();
                    }
                }
                (None, Some(cp)) => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(Some(content[content_start..cp].to_string()));
                    }
                    search_from = cp + closing_tag.len();
                }
                (Some(_), None) | (None, None) => {
                    return Err(WidgetError::ParseError(format!("Unclosed <{}> tag", tag_name)));
                }
            }
        }

        Ok(None)
    }

    /// 从指定位置开始查找标签前缀的起始位置
    fn find_tag_open(content: &str, tag_prefix: &str, from: usize) -> Option<usize> {
        let mut search_from = from;
        while let Some(pos) = content[search_from..].find(tag_prefix) {
            let abs_pos = search_from + pos;
            let after_start = abs_pos + tag_prefix.len();
            if after_start >= content.len() {
                return None;
            }
            let next_char = content.as_bytes()[after_start];
            if matches!(next_char, b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/') {
                return Some(abs_pos);
            }
            search_from = abs_pos + tag_prefix.len();
        }
        None
    }
}
