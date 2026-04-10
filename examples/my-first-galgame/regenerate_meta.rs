use gg_meta::MetaFile;
use std::path::Path;

fn main() {
    // 读取 game.toml.meta 文件
    let meta_path = Path::new("game.toml.meta");
    match MetaFile::from_file(meta_path) {
        Ok(meta) => {
            println!("成功读取 meta 文件");
            // 将 meta 文件写回
            match meta.to_file(meta_path) {
                Ok(_) => println!("成功写回 meta 文件"),
                Err(e) => println!("写回 meta 文件失败: {:?}", e),
            }
        }
        Err(e) => println!("读取 meta 文件失败: {:?}", e),
    }
    
    // 读取 scripts/start.gscript.meta 文件
    let meta_path = Path::new("scripts/start.gscript.meta");
    match MetaFile::from_file(meta_path) {
        Ok(meta) => {
            println!("成功读取 scripts/start.gscript.meta 文件");
            // 将 meta 文件写回
            match meta.to_file(meta_path) {
                Ok(_) => println!("成功写回 scripts/start.gscript.meta 文件"),
                Err(e) => println!("写回 scripts/start.gscript.meta 文件失败: {:?}", e),
            }
        }
        Err(e) => println!("读取 scripts/start.gscript.meta 文件失败: {:?}", e),
    }
}