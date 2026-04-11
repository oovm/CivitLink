#![warn(missing_docs)]

//! Meta 命令模块
//!
//! 用于生成和管理资源的 meta 文件，支持增量生成和重新生成

use clap::{Args, Command};
use std::path::Path;

use crate::platform::Platform;
use gg_core::{GError, GErrorKind, GResult};
use gg_meta::{MetaFile, MetaGenerator};

/// Meta 命令参数
#[derive(Args, Debug)]
pub struct MetaArgs {
    /// 目标目录或文件路径
    #[arg(required = true)]
    pub target: String,

    /// 递归处理目录
    #[arg(short, long, default_value_t = true)]
    pub recursive: bool,

    /// 重新生成 meta 文件（覆盖已存在的 .meta 文件）
    #[arg(long, default_value_t = false)]
    pub regenerate: bool,
}

/// 注册 Meta 命令
pub fn register_command() -> Command {
    MetaArgs::augment_args(Command::new("meta").about("生成和管理资源的 meta 文件"))
}

/// 执行 Meta 命令
pub fn execute(args: &MetaArgs, _platform: &Platform) -> GResult<()> {
    let target_path = Path::new(&args.target);

    if target_path.is_dir() {
        if args.regenerate {
            process_directory_regenerate(target_path, args.recursive)?;
        }
        else {
            process_directory_incremental(target_path, args.recursive)?;
        }
    }
    else if target_path.is_file() {
        if args.regenerate {
            regenerate_meta_file(target_path)?;
        }
        else {
            generate_meta_file_incremental(target_path)?;
        }
    }
    else {
        return Err(GError {
            kind: GErrorKind::Runtime,
            message: format!("Target path does not exist: {}", args.target),
        });
    }

    Ok(())
}

/// 增量处理目录中的文件
///
/// 使用 `MetaGenerator` 为目录中尚未拥有 .meta 文件的资源生成元数据。
/// 当 `recursive` 为 true 时递归处理子目录。
fn process_directory_incremental(path: &Path, recursive: bool) -> GResult<()> {
    let generator = MetaGenerator::new();
    let count = generator
        .generate_for_directory(path, recursive)
        .map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!("Failed to generate meta files: {}", e),
        })?;
    println!("Generated {} meta file(s) in {}", count, path.display());
    Ok(())
}

/// 重新生成目录中所有文件的 .meta 文件
///
/// 遍历目录中的每个非 .meta 文件，强制重新生成其 .meta 文件。
/// 当 `recursive` 为 true 时递归处理子目录。
fn process_directory_regenerate(path: &Path, recursive: bool) -> GResult<()> {
    let mut count = 0usize;
    regenerate_dir_inner(path, recursive, &mut count)?;
    println!("Regenerated {} meta file(s) in {}", count, path.display());
    Ok(())
}

/// 递归重新生成目录中文件的内部实现
fn regenerate_dir_inner(path: &Path, recursive: bool, count: &mut usize) -> GResult<()> {
    for entry in std::fs::read_dir(path).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to read directory '{}': {}", path.display(), e),
    })? {
        let entry = entry.map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to read directory entry: {}", e),
        })?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if recursive {
                regenerate_dir_inner(&entry_path, true, count)?;
            }
        }
        else if entry_path.is_file()
            && !entry_path
                .extension()
                .is_some_and(|ext| ext == "meta")
        {
            regenerate_meta_file(&entry_path)?;
            *count += 1;
        }
    }
    Ok(())
}

/// 增量生成单个文件的 .meta 文件
///
/// 如果对应的 .meta 文件已存在则跳过，否则使用 `MetaGenerator` 生成。
fn generate_meta_file_incremental(file_path: &Path) -> GResult<()> {
    let generator = MetaGenerator::new();
    let generated = generator.generate_for_file(file_path).map_err(|e| GError {
        kind: GErrorKind::Runtime,
        message: format!(
            "Failed to generate meta file for '{}': {}",
            file_path.display(),
            e
        ),
    })?;

    if generated {
        println!("Generated meta file for {}", file_path.display());
    }
    else {
        println!("Skipped (meta already exists): {}", file_path.display());
    }

    Ok(())
}

/// 重新生成单个文件的 .meta 文件
///
/// 如果 .meta 文件已存在，则读取后更新时间戳并重新写入；
/// 如果不存在，则生成新的 .meta 文件。
fn regenerate_meta_file(file_path: &Path) -> GResult<()> {
    let meta_path = {
        let original_ext = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_string());
        match original_ext {
            Some(ext) => file_path.with_extension(format!("{}.meta", ext)),
            None => file_path.with_extension("meta"),
        }
    };

    if meta_path.exists() {
        let mut meta = MetaFile::from_file(&meta_path).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!(
                "Failed to read meta file '{}': {}",
                meta_path.display(),
                e
            ),
        })?;

        let file_metadata = std::fs::metadata(file_path).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!(
                "Failed to read file metadata '{}': {}",
                file_path.display(),
                e
            ),
        })?;
        meta.asset.size = file_metadata.len();
        meta.update_timestamp();

        meta.to_file(&meta_path).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!(
                "Failed to write meta file '{}': {}",
                meta_path.display(),
                e
            ),
        })?;
        println!("Regenerated meta file for {}", file_path.display());
    }
    else {
        let generator = MetaGenerator::new();
        generator.generate_for_file(file_path).map_err(|e| GError {
            kind: GErrorKind::Runtime,
            message: format!(
                "Failed to generate meta file for '{}': {}",
                file_path.display(),
                e
            ),
        })?;
        println!("Generated meta file for {}", file_path.display());
    }

    Ok(())
}
