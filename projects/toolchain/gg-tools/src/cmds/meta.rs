//! Meta 命令模块
//! 
//! 用于生成和管理资源的 meta 文件

use std::fs;
use std::path::{Path, PathBuf};
use clap::{Args, Command};

use gg_core::{GError, GErrorKind, GResult};
use gg_meta::MetaFile;
use crate::platform::Platform;

/// Meta 命令参数
#[derive(Args, Debug)]
pub struct MetaArgs {
    /// 目标目录或文件路径
    #[arg(required = true)]
    pub target: String,
    
    /// 递归处理目录
    #[arg(short, long, default_value_t = true)]
    pub recursive: bool,
    
    /// 先读取再输出（重新生成 meta 文件）
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
        if args.recursive {
            if args.regenerate {
                process_directory_regenerate(target_path)?;
            } else {
                process_directory(target_path)?;
            }
        } else {
            if args.regenerate {
                process_files_in_directory_regenerate(target_path)?;
            } else {
                process_files_in_directory(target_path)?;
            }
        }
    } else if target_path.is_file() {
        if args.regenerate {
            regenerate_meta_file(target_path)?;
        } else {
            generate_meta_file(target_path)?;
        }
    } else {
        return Err(GError { kind: GErrorKind::Runtime, message: format!("Target path does not exist: {}", args.target) });
    }
    
    Ok(())
}

/// 处理目录中的所有文件
fn process_directory(path: &Path) -> GResult<()> {
    for entry in fs::read_dir(path).map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory '{}': {}", path.display(), e) })? {
        let entry = entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            process_directory(&entry_path)?;
        } else if !entry_path.extension().map_or(false, |ext| ext == "meta") {
            generate_meta_file(&entry_path)?;
        }
    }
    Ok(())
}

/// 处理目录中的直接文件（非递归）
fn process_files_in_directory(path: &Path) -> GResult<()> {
    for entry in fs::read_dir(path).map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory '{}': {}", path.display(), e) })? {
        let entry = entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let entry_path = entry.path();
        
        if entry_path.is_file() && !entry_path.extension().map_or(false, |ext| ext == "meta") {
            generate_meta_file(&entry_path)?;
        }
    }
    Ok(())
}

/// 生成 meta 文件
fn generate_meta_file(file_path: &Path) -> GResult<()> {
    let stats = fs::metadata(file_path).map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read file metadata '{}': {}", file_path.display(), e) })?;
    let size = stats.len() as u64;
    let name = file_path.file_name().unwrap().to_string_lossy().to_string();
    let relative_path = get_relative_path(file_path)?;
    let asset_type = get_asset_type(&name);
    
    // 使用 gg-meta 库创建 MetaFile
    let mut meta = MetaFile::new(
        &asset_type,
        &relative_path,
        &name,
        size,
    );
    
    // 更新时间戳
    meta.update_timestamp();
    
    // 生成 meta 文件路径
    let meta_path = file_path.with_extension(format!("{}.meta", file_path.extension().unwrap_or_default().to_string_lossy()));
    
    // 写入文件
    meta.to_file(&meta_path).map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to write meta file '{}': {}", meta_path.display(), e) })?;
    println!("Generated meta file for {}", file_path.display());
    
    Ok(())
}

/// 重新生成 meta 文件（先读取再输出）
fn regenerate_meta_file(file_path: &Path) -> GResult<()> {
    // 生成 meta 文件路径
    let meta_path = file_path.with_extension(format!("{}.meta", file_path.extension().unwrap_or_default().to_string_lossy()));
    
    // 检查 meta 文件是否存在
    if meta_path.exists() {
        // 读取现有的 meta 文件
        let mut meta = MetaFile::from_file(&meta_path).map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to read meta file '{}': {}", meta_path.display(), e) })?;
        
        // 更新时间戳
        meta.update_timestamp();
        
        // 写入文件
        meta.to_file(&meta_path).map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to write meta file '{}': {}", meta_path.display(), e) })?;
        println!("Regenerated meta file for {}", file_path.display());
    } else {
        // 如果 meta 文件不存在，则生成新的
        generate_meta_file(file_path)?;
    }
    
    Ok(())
}

/// 处理目录中的所有文件（重新生成）
fn process_directory_regenerate(path: &Path) -> GResult<()> {
    for entry in fs::read_dir(path).map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory '{}': {}", path.display(), e) })? {
        let entry = entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            process_directory_regenerate(&entry_path)?;
        } else if entry_path.is_file() && !entry_path.extension().map_or(false, |ext| ext == "meta") {
            regenerate_meta_file(&entry_path)?;
        }
    }
    Ok(())
}

/// 处理目录中的直接文件（非递归，重新生成）
fn process_files_in_directory_regenerate(path: &Path) -> GResult<()> {
    for entry in fs::read_dir(path).map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory '{}': {}", path.display(), e) })? {
        let entry = entry.map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to read directory entry: {}", e) })?;
        let entry_path = entry.path();
        
        if entry_path.is_file() && !entry_path.extension().map_or(false, |ext| ext == "meta") {
            regenerate_meta_file(&entry_path)?;
        }
    }
    Ok(())
}

/// 获取相对路径
fn get_relative_path(file_path: &Path) -> GResult<String> {
    let project_root = Path::new("e:\灵之镜有限公司\gg-game-engine");
    let relative_path = file_path.strip_prefix(project_root).map_err(|e| GError { kind: GErrorKind::Runtime, message: format!("Failed to get relative path: {}", e) })?;
    Ok(relative_path.to_string_lossy().replace('\\', "/"))
}

/// 获取资源类型
fn get_asset_type(filename: &str) -> String {
    let ext = Path::new(filename).extension().unwrap_or_default().to_string_lossy().to_lowercase();
    match ext.as_str() {
        "toml" => "config".to_string(),
        "gscript" => "script".to_string(),
        "png" => "texture".to_string(),
        "jpg" => "texture".to_string(),
        "jpeg" => "texture".to_string(),
        "wav" => "audio".to_string(),
        "mp3" => "audio".to_string(),
        "ogg" => "audio".to_string(),
        "glsl" => "shader".to_string(),
        "vert" => "shader".to_string(),
        "frag" => "shader".to_string(),
        "scene" => "scene".to_string(),
        "prefab" => "prefab".to_string(),
        "anim" => "animation".to_string(),
        "material" => "material".to_string(),
        _ => "unknown".to_string(),
    }
}
