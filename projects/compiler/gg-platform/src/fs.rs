use std::path::PathBuf;

use gg_core::{GError, GResult};

/// 平台通用文件系统接口
///
/// 为各平台提供文件系统操作的统一接口。
pub trait PlatformFileSystem {
    /// 读取文件内容
    fn read_file(&self, path: &PathBuf) -> GResult<Vec<u8>>;

    /// 写入文件内容
    fn write_file(&self, path: &PathBuf, content: &[u8]) -> GResult<()>;

    /// 创建目录
    fn create_dir(&self, path: &PathBuf) -> GResult<()>;

    /// 创建目录（递归）
    fn create_dir_all(&self, path: &PathBuf) -> GResult<()>;

    /// 列出目录内容
    fn read_dir(&self, path: &PathBuf) -> GResult<Vec<PathBuf>>;

    /// 检查路径是否存在
    fn exists(&self, path: &PathBuf) -> bool;

    /// 检查路径是否为文件
    fn is_file(&self, path: &PathBuf) -> bool;

    /// 检查路径是否为目录
    fn is_dir(&self, path: &PathBuf) -> bool;

    /// 删除文件或目录
    fn remove(&self, path: &PathBuf) -> GResult<()>;

    /// 复制文件
    fn copy(&self, from: &PathBuf, to: &PathBuf) -> GResult<()>;

    /// 移动文件
    fn rename(&self, from: &PathBuf, to: &PathBuf) -> GResult<()>;
}
