use gg_meta::{MetaFile, generate_guid};
use std::path::Path;

fn main() {
    // 创建一个新的元数据文件
    let mut meta = MetaFile::new(
        "texture",
        "assets/textures/player.png",
        "Player Texture",
        1024 * 1024 * 4, // 4MB
    );

    // 添加依赖
    meta.add_dependency("assets/textures/player_normal.png", &generate_guid());
    meta.add_dependency("assets/textures/player_specular.png", &generate_guid());

    // 添加引用
    meta.add_reference("assets/prefabs/player.prefab", Some("texture"));
    meta.add_reference("assets/scenes/main.scene", Some("player_texture"));

    // 更新时间戳
    meta.update_timestamp();

    // 更新哈希值（可选）
    meta.update_hash("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

    // 打印元数据信息
    println!("Asset Type: {}", meta.asset.r#type);
    println!("Asset Path: {}", meta.asset.path);
    println!("Asset GUID: {}", meta.asset.guid);
    println!("Asset Name: {}", meta.asset.name);
    println!("Asset Size: {} bytes", meta.asset.size);
    println!("Dependencies: {}", meta.dependencies.len());
    println!("References: {}", meta.references.len());

    // 尝试写入文件（需要实际的文件系统权限）
    // meta.to_file(Path::new("assets/textures/player.png.meta")).unwrap();
    // println!("Metadata written to file");

    println!("\nMetaFile created successfully!");
}
