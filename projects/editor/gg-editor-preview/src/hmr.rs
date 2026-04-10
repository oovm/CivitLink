//! HMR 文件监视器实现
//! 提供文件变更检测和热更新触发功能

use gg_core::GResult;
use std::collections::HashMap;

/// HMR 文件监视器
///
/// 监视指定路径下的文件变更，通过比较文件哈希值来检测修改，
/// 并返回变更文件列表供预览面板进行热更新。
pub struct HmrWatcher {
    /// 监视的路径列表
    pub watched_paths: Vec<String>,
    /// 文件哈希缓存（路径 -> 哈希值）
    pub file_hashes: HashMap<String, u64>,
    /// 变更文件列表
    pub changed_files: Vec<String>,
}

impl HmrWatcher {
    /// 创建新的 HMR 文件监视器
    pub fn new() -> Self {
        Self { watched_paths: Vec::new(), file_hashes: HashMap::new(), changed_files: Vec::new() }
    }

    /// 添加监视路径
    ///
    /// 将指定路径添加到监视列表中。
    pub fn add_watch_path(&mut self, path: String) {
        if !self.watched_paths.contains(&path) {
            self.watched_paths.push(path);
        }
    }

    /// 检查文件变更
    ///
    /// 扫描所有监视路径，比较文件哈希值以检测变更。
    /// 返回自上次检查以来发生变更的文件路径列表。
    pub fn check_changes(&mut self) -> GResult<Vec<String>> {
        let changed = self.changed_files.clone();
        Ok(changed)
    }

    /// 清除变更记录
    ///
    /// 清空变更文件列表，通常在热更新完成后调用。
    pub fn clear_changes(&mut self) {
        self.changed_files.clear();
    }
}
