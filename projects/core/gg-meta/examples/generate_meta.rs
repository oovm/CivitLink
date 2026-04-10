use gg_meta::{MetaFile, generate_guid};
use std::path::Path;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 目标目录
    let target_dir = Path::new("../../examples/my-first-galgame");
    
    // 处理 game.toml
    let game_toml_path = target_dir.join("game.toml");
    if game_toml_path.exists() {
        generate_meta_for_file(&game_toml_path, "config")?;
    }
    
    // 处理 scripts 目录
    let scripts_dir = target_dir.join("scripts");
    if scripts_dir.exists() && scripts_dir.is_dir() {
        for entry in fs::read_dir(scripts_dir)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_file() && !entry_path.extension().map_or(false, |ext| ext == "meta") {
                generate_meta_for_file(&entry_path, "script")?;
            }
        }
    }
    
    Ok(())
}

fn generate_meta_for_file(file_path: &Path, asset_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stats = fs::metadata(file_path)?;
    let size = stats.len();
    let name = file_path.file_name().unwrap().to_string_lossy().to_string();
    let relative_path = get_relative_path(file_path)?;
    
    // 使用 gg-meta 库创建 MetaFile
    let mut meta = MetaFile::new(
        asset_type,
        &relative_path,
        &name,
        size,
    );
    
    // 更新时间戳
    meta.update_timestamp();
    
    // 生成 meta 文件路径
    let meta_path = file_path.with_extension(format!("{}.meta", file_path.extension().unwrap_or_default().to_string_lossy()));
    
    // 写入文件
    meta.to_file(&meta_path)?;
    println!("Generated meta file for {}", file_path.display());
    
    Ok(())
}

/// 获取相对路径
fn get_relative_path(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let project_root = Path::new("../../../");
    let absolute_path = std::env::current_dir()?.join(file_path);
    let relative_path = absolute_path.strip_prefix(project_root)?;
    Ok(relative_path.to_string_lossy().replace('\\', "/"))
}
