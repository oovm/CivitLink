use std::path::Path;

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{DirEntry, FileMetadata, FileSystem, FileType},
};

/// 桌面平台文件系统实现
///
/// 基于 `std::fs` 提供桌面平台的文件访问能力。
pub struct DesktopFileSystem;

impl DesktopFileSystem {
    /// 创建新的桌面文件系统实例
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for DesktopFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read(&self, path: &Path) -> GResult<Vec<u8>> {
        std::fs::read(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read file '{}': {}", path.display(), e) })
    }

    fn read_to_string(&self, path: &Path) -> GResult<String> {
        std::fs::read_to_string(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read file '{}': {}", path.display(), e) })
    }

    fn write(&self, path: &Path, content: &[u8]) -> GResult<()> {
        std::fs::write(path, content)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write file '{}': {}", path.display(), e) })
    }

    fn create_dir_all(&self, path: &Path) -> GResult<()> {
        std::fs::create_dir_all(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create directory '{}': {}", path.display(), e),
        })
    }

    fn read_dir(&self, path: &Path) -> GResult<Vec<DirEntry>> {
        let entries = std::fs::read_dir(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read directory '{}': {}", path.display(), e),
        })?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;

            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_type = if entry
                .file_type()
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to get file type: {}", e) })?
                .is_dir()
            {
                FileType::Directory
            }
            else {
                FileType::File
            };

            result.push(DirEntry { name: file_name, file_type });
        }

        Ok(result)
    }

    fn metadata(&self, path: &Path) -> GResult<FileMetadata> {
        let meta = std::fs::metadata(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to get metadata for '{}': {}", path.display(), e),
        })?;

        let file_type = if meta.is_dir() {
            FileType::Directory
        }
        else if meta.is_file() {
            FileType::File
        }
        else {
            FileType::Symlink
        };

        Ok(FileMetadata { file_type, len: meta.len(), modified: meta.modified().ok() })
    }

    fn remove_file(&self, path: &Path) -> GResult<()> {
        std::fs::remove_file(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to remove file '{}': {}", path.display(), e) })
    }
}
