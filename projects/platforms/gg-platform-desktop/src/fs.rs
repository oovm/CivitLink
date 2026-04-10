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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use gg_core::platform::FileSystem;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join("gg_platform_desktop_test")
    }

    fn setup() {
        let _ = std::fs::create_dir_all(temp_dir());
    }

    fn cleanup() {
        let _ = std::fs::remove_dir_all(temp_dir());
    }

    #[test]
    fn test_write_and_read() {
        setup();
        let fs = DesktopFileSystem::new();
        let path = temp_dir().join("test_write.txt");
        let content = b"Hello, GG!";
        fs.write(&path, content).unwrap();
        let read = fs.read(&path).unwrap();
        assert_eq!(read, content);
        cleanup();
    }

    #[test]
    fn test_exists() {
        setup();
        let fs = DesktopFileSystem::new();
        let path = temp_dir().join("test_exists.txt");
        assert!(!fs.exists(&path));
        fs.write(&path, b"test").unwrap();
        assert!(fs.exists(&path));
        cleanup();
    }

    #[test]
    fn test_create_dir_all() {
        setup();
        let fs = DesktopFileSystem::new();
        let dir = temp_dir().join("nested/dir");
        fs.create_dir_all(&dir).unwrap();
        assert!(dir.exists());
        cleanup();
    }

    #[test]
    fn test_read_to_string() {
        setup();
        let fs = DesktopFileSystem::new();
        let path = temp_dir().join("test_string.txt");
        fs.write(&path, "Hello String".as_bytes()).unwrap();
        let content = fs.read_to_string(&path).unwrap();
        assert_eq!(content, "Hello String");
        cleanup();
    }

    #[test]
    fn test_remove_file() {
        setup();
        let fs = DesktopFileSystem::new();
        let path = temp_dir().join("test_remove.txt");
        fs.write(&path, b"temp").unwrap();
        assert!(fs.exists(&path));
        fs.remove_file(&path).unwrap();
        assert!(!fs.exists(&path));
        cleanup();
    }
}
