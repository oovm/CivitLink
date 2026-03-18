//! 文件系统抽象
//!
//! 提供跨平台的文件系统接口。

use std::path::Path;

use super::{PlatformError, PlatformResult};

/// 文件系统 trait
pub trait Filesystem {
    /// 读取文件内容
    fn read_file(&self, path: &Path) -> PlatformResult<Vec<u8>>;

    /// 读取文件为字符串
    fn read_to_string(&self, path: &Path) -> PlatformResult<String> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes)
            .map_err(|e| PlatformError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    /// 写入文件
    fn write_file(&mut self, path: &Path, data: &[u8]) -> PlatformResult<()>;

    /// 检查文件是否存在
    fn exists(&self, path: &Path) -> bool;

    /// 删除文件
    fn remove_file(&mut self, path: &Path) -> PlatformResult<()>;

    /// 创建目录
    fn create_dir(&mut self, path: &Path) -> PlatformResult<()>;

    /// 列出目录内容
    fn list_dir(&self, path: &Path) -> PlatformResult<Vec<String>>;
}

/// 空文件系统（用于平台不支持文件系统的情况）
pub struct NullFilesystem;

impl Filesystem for NullFilesystem {
    fn read_file(&self, _path: &Path) -> PlatformResult<Vec<u8>> {
        Err(PlatformError::UnsupportedOperation("File system not available".to_string()))
    }

    fn write_file(&mut self, _path: &Path, _data: &[u8]) -> PlatformResult<()> {
        Err(PlatformError::UnsupportedOperation("File system not available".to_string()))
    }

    fn exists(&self, _path: &Path) -> bool {
        false
    }

    fn remove_file(&mut self, _path: &Path) -> PlatformResult<()> {
        Err(PlatformError::UnsupportedOperation("File system not available".to_string()))
    }

    fn create_dir(&mut self, _path: &Path) -> PlatformResult<()> {
        Err(PlatformError::UnsupportedOperation("File system not available".to_string()))
    }

    fn list_dir(&self, _path: &Path) -> PlatformResult<Vec<String>> {
        Err(PlatformError::UnsupportedOperation("File system not available".to_string()))
    }
}
