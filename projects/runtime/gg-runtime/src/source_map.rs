//! 源码映射模块
//! 提供字节码偏移到源码位置的映射能力

/// 源码映射条目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapEntry {
    /// 字节码偏移
    pub bytecode_offset: usize,
    /// 源文件路径
    pub source_file: String,
    /// 源码行号（1-based）
    pub source_line: usize,
    /// 源码列号（1-based）
    pub source_column: usize,
}

/// 源码映射
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// 映射条目列表，按 bytecode_offset 排序
    pub entries: Vec<SourceMapEntry>,
}

impl SourceMap {
    /// 创建新的空源码映射
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 添加映射条目
    pub fn add_entry(&mut self, entry: SourceMapEntry) {
        let pos = self.entries.partition_point(|e| e.bytecode_offset < entry.bytecode_offset);
        self.entries.insert(pos, entry);
    }

    /// 根据字节码偏移查找源码位置
    pub fn lookup(&self, bytecode_offset: usize) -> Option<&SourceMapEntry> {
        let idx = self.entries.partition_point(|e| e.bytecode_offset <= bytecode_offset);
        if idx == 0 {
            return None;
        }
        Some(&self.entries[idx - 1])
    }

    /// 获取映射条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 映射是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
