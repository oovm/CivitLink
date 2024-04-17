//! 字节码调试信息模块
//! 提供源码位置到字节码偏移的映射，支持调试和错误报告

use std::collections::HashMap;

/// 源码位置信息
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// 源文件路径
    pub file: String,
    /// 行号（从 1 开始）
    pub line: u32,
    /// 列号（从 1 开始）
    pub column: u32,
}

impl SourceLocation {
    /// 创建新的源码位置
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self { file: file.into(), line, column }
    }
}

/// 字节码调试信息
/// 记录源码位置到字节码偏移的映射
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    /// 字节码偏移到源码位置的映射
    entries: HashMap<u32, SourceLocation>,
    /// 函数名到源文件的映射
    function_map: HashMap<String, String>,
}

impl DebugInfo {
    /// 创建空的调试信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一条字节码偏移到源码位置的映射
    pub fn add_entry(&mut self, offset: u32, location: SourceLocation) {
        self.entries.insert(offset, location);
    }

    /// 添加一条函数名到源文件的映射
    pub fn add_function(&mut self, function_name: impl Into<String>, source_file: impl Into<String>) {
        self.function_map.insert(function_name.into(), source_file.into());
    }

    /// 根据字节码偏移查找源码位置
    pub fn lookup(&self, offset: u32) -> Option<&SourceLocation> {
        self.entries.get(&offset)
    }

    /// 根据函数名查找源文件
    pub fn lookup_function(&self, name: &str) -> Option<&str> {
        self.function_map.get(name).map(|s| s.as_str())
    }

    /// 获取所有条目的引用
    pub fn entries(&self) -> &HashMap<u32, SourceLocation> {
        &self.entries
    }

    /// 获取所有函数映射的引用
    pub fn function_map(&self) -> &HashMap<String, String> {
        &self.function_map
    }

    /// 检查调试信息是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.function_map.is_empty()
    }
}
