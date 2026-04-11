//! 字节码调试信息模块
//! 提供源码位置到字节码偏移的映射，支持调试和错误报告

use std::collections::HashMap;

/// 源码位置信息
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    /// 源文件路径
    pub file: String,
    /// 行号（从 1 开始）
    pub line: u32,
    /// 列号（从 1 开始）
    pub column: u32,
    /// 起始字节码偏移
    #[serde(default)]
    pub start_offset: u32,
    /// 结束字节码偏移
    #[serde(default)]
    pub end_offset: u32,
}

impl SourceLocation {
    /// 创建新的源码位置
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self { file: file.into(), line, column, start_offset: 0, end_offset: 0 }
    }

    /// 创建带字节码偏移范围的源码位置
    pub fn with_offsets(file: impl Into<String>, line: u32, column: u32, start_offset: u32, end_offset: u32) -> Self {
        Self { file: file.into(), line, column, start_offset, end_offset }
    }
}

/// 调试条目，关联字节码偏移与源码位置
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DebugEntry {
    /// 字节码偏移
    pub offset: u32,
    /// 源码位置
    pub location: SourceLocation,
}

impl DebugEntry {
    /// 创建新的调试条目
    pub fn new(offset: u32, location: SourceLocation) -> Self {
        Self { offset, location }
    }
}

/// 字节码调试信息
/// 记录源码位置到字节码偏移的映射
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DebugInfo {
    /// 字节码偏移到源码位置的映射条目（按 offset 排序）
    entries: Vec<DebugEntry>,
    /// 函数名到源文件的映射
    function_map: HashMap<String, String>,
    /// 函数名到局部变量名列表的映射
    #[serde(default)]
    variable_names: HashMap<String, Vec<String>>,
}

impl DebugInfo {
    /// 创建空的调试信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一条字节码偏移到源码位置的映射
    pub fn add_entry(&mut self, offset: u32, location: SourceLocation) {
        let entry = DebugEntry::new(offset, location);
        let pos = self.entries.iter().position(|e| e.offset > offset).unwrap_or(self.entries.len());
        self.entries.insert(pos, entry);
    }

    /// 添加一条函数名到源文件的映射
    pub fn add_function(&mut self, function_name: impl Into<String>, source_file: impl Into<String>) {
        self.function_map.insert(function_name.into(), source_file.into());
    }

    /// 根据字节码偏移查找源码位置（精确匹配）
    pub fn lookup(&self, offset: u32) -> Option<&SourceLocation> {
        self.entries.binary_search_by_key(&offset, |e| e.offset).ok().map(|idx| &self.entries[idx].location)
    }

    /// 根据字节码偏移查找最近的源码位置（范围查找）
    ///
    /// 使用二分查找找到 offset <= 给定偏移的最近条目。
    /// 如果给定偏移超过所有条目，返回最后一个条目的位置。
    pub fn lookup_by_range(&self, offset: u32) -> Option<&SourceLocation> {
        if self.entries.is_empty() {
            return None;
        }

        match self.entries.binary_search_by_key(&offset, |e| e.offset) {
            Ok(idx) => Some(&self.entries[idx].location),
            Err(idx) => {
                if idx == 0 {
                    None
                }
                else {
                    Some(&self.entries[idx - 1].location)
                }
            }
        }
    }

    /// 根据函数名查找源文件
    pub fn lookup_function(&self, name: &str) -> Option<&str> {
        self.function_map.get(name).map(|s| s.as_str())
    }

    /// 添加函数的局部变量名列表
    pub fn add_variable_names(&mut self, function_name: String, names: Vec<String>) {
        self.variable_names.insert(function_name, names);
    }

    /// 根据函数名查找局部变量名列表
    pub fn lookup_variable_names(&self, function_name: &str) -> Option<&Vec<String>> {
        self.variable_names.get(function_name)
    }

    /// 获取所有条目的引用
    pub fn entries(&self) -> &Vec<DebugEntry> {
        &self.entries
    }

    /// 获取所有函数映射的引用
    pub fn function_map(&self) -> &HashMap<String, String> {
        &self.function_map
    }

    /// 获取所有变量名称映射的引用
    pub fn variable_names(&self) -> &HashMap<String, Vec<String>> {
        &self.variable_names
    }

    /// 检查调试信息是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.function_map.is_empty() && self.variable_names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_entry_creation() {
        let location = SourceLocation::new("test.gg", 10, 5);
        let entry = DebugEntry::new(42, location);
        assert_eq!(entry.offset, 42);
        assert_eq!(entry.location.file, "test.gg");
        assert_eq!(entry.location.line, 10);
        assert_eq!(entry.location.column, 5);
    }

    #[test]
    fn test_lookup_by_range_exact_match() {
        let mut info = DebugInfo::new();
        info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        info.add_entry(5, SourceLocation::new("test.gg", 2, 1));
        info.add_entry(10, SourceLocation::new("test.gg", 3, 1));

        let result = info.lookup_by_range(5);
        assert!(result.is_some());
        let loc = result.unwrap();
        assert_eq!(loc.line, 2);
    }

    #[test]
    fn test_lookup_by_range_non_exact_offset() {
        let mut info = DebugInfo::new();
        info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        info.add_entry(5, SourceLocation::new("test.gg", 2, 1));
        info.add_entry(10, SourceLocation::new("test.gg", 3, 1));

        let result = info.lookup_by_range(7);
        assert!(result.is_some());
        let loc = result.unwrap();
        assert_eq!(loc.line, 2);
    }

    #[test]
    fn test_lookup_by_range_offset_exceeds_range() {
        let mut info = DebugInfo::new();
        info.add_entry(0, SourceLocation::new("test.gg", 1, 1));
        info.add_entry(5, SourceLocation::new("test.gg", 2, 1));
        info.add_entry(10, SourceLocation::new("test.gg", 3, 1));

        let result = info.lookup_by_range(100);
        assert!(result.is_some());
        let loc = result.unwrap();
        assert_eq!(loc.line, 3);
    }

    #[test]
    fn test_variable_names_add_and_lookup() {
        let mut info = DebugInfo::new();
        info.add_variable_names("main".to_string(), vec!["x".to_string(), "y".to_string()]);
        info.add_variable_names("helper".to_string(), vec!["a".to_string()]);

        let names = info.lookup_variable_names("main");
        assert!(names.is_some());
        assert_eq!(names.unwrap().len(), 2);
        assert_eq!(names.unwrap()[0], "x");
        assert_eq!(names.unwrap()[1], "y");

        let names = info.lookup_variable_names("helper");
        assert!(names.is_some());
        assert_eq!(names.unwrap().len(), 1);

        assert!(info.lookup_variable_names("nonexistent").is_none());
    }

    #[test]
    fn test_source_location_new_fields() {
        let loc = SourceLocation::new("test.gg", 5, 10);
        assert_eq!(loc.file, "test.gg");
        assert_eq!(loc.line, 5);
        assert_eq!(loc.column, 10);
        assert_eq!(loc.start_offset, 0);
        assert_eq!(loc.end_offset, 0);

        let loc_with_offsets = SourceLocation::with_offsets("test.gg", 5, 10, 20, 40);
        assert_eq!(loc_with_offsets.start_offset, 20);
        assert_eq!(loc_with_offsets.end_offset, 40);
    }

    #[test]
    fn test_lookup_by_range_empty() {
        let info = DebugInfo::new();
        assert!(info.lookup_by_range(0).is_none());
    }

    #[test]
    fn test_lookup_by_range_before_first_entry() {
        let mut info = DebugInfo::new();
        info.add_entry(10, SourceLocation::new("test.gg", 1, 1));

        assert!(info.lookup_by_range(5).is_none());
    }

    #[test]
    fn test_entries_sorted_by_offset() {
        let mut info = DebugInfo::new();
        info.add_entry(20, SourceLocation::new("test.gg", 3, 1));
        info.add_entry(5, SourceLocation::new("test.gg", 1, 1));
        info.add_entry(10, SourceLocation::new("test.gg", 2, 1));

        let entries = info.entries();
        assert_eq!(entries[0].offset, 5);
        assert_eq!(entries[1].offset, 10);
        assert_eq!(entries[2].offset, 20);
    }
}
