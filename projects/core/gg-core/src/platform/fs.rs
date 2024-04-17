#![warn(missing_docs)]

//! 文件系统抽象层
//! 提供跨平台的文件系统操作接口

use std::path::Path;

use crate::{GError, GErrorKind, GResult};

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// 普通文件
    File,
    /// 目录
    Directory,
    /// 符号链接
    Symlink,
}

/// 文件元数据
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// 文件类型
    pub file_type: FileType,
    /// 文件大小（字节）
    pub len: u64,
    /// 最后修改时间
    pub modified: Option<std::time::SystemTime>,
}

/// 目录条目
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// 条目名称
    pub name: String,
    /// 条目类型
    pub file_type: FileType,
}

/// 文件系统抽象 trait
pub trait FileSystem {
    /// 检查路径是否存在
    fn exists(&self, path: &Path) -> bool;

    /// 读取文件内容为字节
    fn read(&self, path: &Path) -> GResult<Vec<u8>>;

    /// 读取文件内容为字符串
    fn read_to_string(&self, path: &Path) -> GResult<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|e| GError { kind: GErrorKind::Io, message: e.to_string() })
    }

    /// 写入字节到文件
    fn write(&self, path: &Path, content: &[u8]) -> GResult<()>;

    /// 递归创建目录
    fn create_dir_all(&self, path: &Path) -> GResult<()>;

    /// 读取目录内容
    fn read_dir(&self, path: &Path) -> GResult<Vec<DirEntry>>;

    /// 获取文件元数据
    fn metadata(&self, path: &Path) -> GResult<FileMetadata>;

    /// 删除文件
    fn remove_file(&self, path: &Path) -> GResult<()>;
}
