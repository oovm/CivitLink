use std::path::Path;

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{DirEntry, FileMetadata, FileSystem, FileType},
};

/// Android 平台文件系统实现
///
/// 为 Android 平台提供文件系统操作的具体实现。
pub struct AndroidFileSystem;

impl AndroidFileSystem {
    /// 创建 Android 文件系统实例
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for AndroidFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read(&self, path: &Path) -> GResult<Vec<u8>> {
        std::fs::read(path)
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
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read directory '{}': {}", path.display(), e),
        })? {
            let entry = entry
                .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
            let file_type =
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) { FileType::Directory } else { FileType::File };
            entries.push(DirEntry { name: entry.file_name().to_string_lossy().to_string(), file_type });
        }
        Ok(entries)
    }

    fn metadata(&self, path: &Path) -> GResult<FileMetadata> {
        let metadata = std::fs::metadata(path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to get metadata for '{}': {}", path.display(), e),
        })?;
        let file_type = if metadata.is_dir() {
            FileType::Directory
        }
        else if metadata.is_symlink() {
            FileType::Symlink
        }
        else {
            FileType::File
        };
        Ok(FileMetadata { file_type, len: metadata.len(), modified: metadata.modified().ok() })
    }

    fn remove_file(&self, path: &Path) -> GResult<()> {
        std::fs::remove_file(path)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to remove file '{}': {}", path.display(), e) })
    }
}
