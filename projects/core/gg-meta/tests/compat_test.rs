use gg_meta::MetaFile;
use std::fs;

#[test]
fn test_ron_compatibility() {
    // 创建一个临时目录
    let temp_dir = tempfile::tempdir().unwrap();
    let meta_path = temp_dir.path().join("test.png.meta");
    
    // 创建一个 VON 格式的元数据文件
    let von_content = r#"{
    version = "1.0"
    asset = {
        type = "texture"
        path = "assets/textures/test.png"
        guid = "018dc3f0-82c9-7d1a-8c3a-9d8b7e6f0000"
        name = "Test Texture"
        size = 4194304
        modified = "2026-04-10T12:00:00Z"
    }
    import_settings = null
    dependencies = []
    references = []
    timestamp = "2026-04-10T12:00:00Z"
    hash = null
}
"#;
    
    // 写入 VON 格式的文件
    fs::write(&meta_path, von_content).unwrap();
    
    // 尝试使用 Oak-von 解析 VON 格式的文件
    let result = MetaFile::from_file(&meta_path);
    
    // 验证解析成功
    assert!(result.is_ok(), "Failed to parse VON format: {:?}", result);
    
    let meta = result.unwrap();
    
    // 验证解析的数据正确
    assert_eq!(meta.version, "1.0");
    assert_eq!(meta.asset.r#type, "texture");
    assert_eq!(meta.asset.path, "assets/textures/test.png");
    assert_eq!(meta.asset.name, "Test Texture");
    assert_eq!(meta.asset.size, 1024 * 1024 * 4);
    
    // 清理
    fs::remove_file(meta_path).unwrap();
}
