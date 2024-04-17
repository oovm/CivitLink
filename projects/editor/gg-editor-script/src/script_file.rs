//! 脚本文件数据结构

use std::path::PathBuf;

/// 脚本文件描述，包含文件元信息和诊断摘要
#[derive(Debug, Clone)]
pub struct ScriptFile {
    /// 文件完整路径
    pub path: PathBuf,
    /// 文件名（含扩展名）
    pub file_name: String,
    /// 修改时间（UNIX 时间戳，秒）
    pub modified_time: u64,
    /// 文件大小（字节）
    pub size: u64,
    /// 错误数量
    pub error_count: usize,
    /// 警告数量
    pub warning_count: usize,
}

impl ScriptFile {
    /// 创建新的脚本文件描述
    ///
    /// # 参数
    /// - `path`: 文件完整路径
    /// - `file_name`: 文件名（含扩展名）
    /// - `modified_time`: 修改时间（UNIX 时间戳，秒）
    /// - `size`: 文件大小（字节）
    pub fn new(path: PathBuf, file_name: String, modified_time: u64, size: u64) -> Self {
        Self { path, file_name, modified_time, size, error_count: 0, warning_count: 0 }
    }

    /// 更新诊断摘要
    ///
    /// # 参数
    /// - `error_count`: 错误数量
    /// - `warning_count`: 警告数量
    pub fn update_diagnostics(&mut self, error_count: usize, warning_count: usize) {
        self.error_count = error_count;
        self.warning_count = warning_count;
    }

    /// 是否有诊断问题
    pub fn has_issues(&self) -> bool {
        self.error_count > 0 || self.warning_count > 0
    }
}
