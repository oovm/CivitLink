#![warn(missing_docs)]

//! 平台通用工具函数
//!
//! 提供跨平台的通用工具函数，包括路径规范化、目录复制、文件查找和命令检测等。
//! 各平台模块应优先使用此模块中的函数，避免重复实现。

use std::path::PathBuf;

use gg_core::{GError, GErrorKind, GResult};

/// 规范化路径为当前平台格式
///
/// 在 Windows 上保留反斜杠，在其他平台上统一使用正斜杠。
/// 同时移除路径中的冗余分隔符和 `.`/`..` 组件。
pub fn normalize_path(path: impl Into<PathBuf>) -> PathBuf {
    let mut p = path.into();
    if cfg!(windows) {
        if let Ok(canonical) = p.canonicalize() {
            p = canonical;
        }
    }
    else if let Ok(canonical) = p.canonicalize() {
        p = canonical;
    }
    p
}

/// 递归复制目录
///
/// 将源目录下的所有文件和子目录递归复制到目标目录。
/// 如果目标目录不存在，会自动创建。
pub fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> GResult<()> {
    std::fs::create_dir_all(dst).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create directory '{}': {}", dst.display(), e),
    })?;

    let entries = std::fs::read_dir(src).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read directory '{}': {}", src.display(), e),
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        }
        else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to copy file '{}': {}", src_path.display(), e),
            })?;
        }
    }

    Ok(())
}

/// 在目录中查找可执行文件
///
/// 首先查找精确名称匹配，然后在 Windows 上尝试添加 `.exe` 后缀，
/// 最后如果目录中只有一个文件，则返回该文件作为回退。
pub fn find_executable(dir: &PathBuf, name: &str) -> GResult<PathBuf> {
    let named = dir.join(name);
    if named.exists() {
        return Ok(named);
    }

    if cfg!(windows) {
        let exe_named = dir.join(format!("{}.exe", name));
        if exe_named.exists() {
            return Ok(exe_named);
        }
    }

    let entries = std::fs::read_dir(dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read directory '{}': {}", dir.display(), e),
    })?;

    let mut executables = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if file_name == name || file_name == format!("{}.exe", name) {
                return Ok(path);
            }
            executables.push(path);
        }
    }

    if executables.len() == 1 {
        return Ok(executables.remove(0));
    }

    Err(GError { kind: GErrorKind::Io, message: format!("Executable '{}' not found in '{}'", name, dir.display()) })
}

/// 写入文件内容
///
/// 将字符串内容写入指定路径的文件，如果文件已存在则覆盖。
pub fn write_file(path: &PathBuf, content: &str) -> GResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create parent directory '{}': {}", parent.display(), e),
        })?;
    }
    std::fs::write(path, content)
        .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to write file '{}': {}", path.display(), e) })
}

/// 检测命令是否可用
///
/// 通过执行 `cmd --version` 来检测指定命令是否在系统 PATH 中可用。
pub fn is_command_available(cmd: &str) -> bool {
    std::process::Command::new(cmd).arg("--version").output().is_ok()
}

/// 按扩展名复制文件
///
/// 将源目录中所有匹配指定扩展名的文件复制到目标目录。
pub fn copy_files_by_ext(src_dir: &PathBuf, dst_dir: &PathBuf, ext: &str) -> GResult<()> {
    let mut found = false;
    let entries = std::fs::read_dir(src_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read directory '{}': {}", src_dir.display(), e),
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            if file_name.ends_with(ext) {
                let dst = dst_dir.join(entry.file_name());
                std::fs::copy(&path, &dst).map_err(|e| GError {
                    kind: GErrorKind::Io,
                    message: format!("Failed to copy file '{}': {}", path.display(), e),
                })?;
                found = true;
            }
        }
    }

    if !found {
        return Err(GError {
            kind: GErrorKind::Io,
            message: format!("No files with extension '{}' found in '{}'", ext, src_dir.display()),
        });
    }

    Ok(())
}
