use gg_meta::{generate_guid, MetaFile};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_create_meta_file() {
    // Create a temporary directory
    let temp_dir = tempdir().unwrap();
    let meta_path = temp_dir.path().join("test.png.meta");

    // Create a new metadata file
    let mut meta = MetaFile::new("texture", "assets/textures/test.png", "Test Texture", 1024 * 1024 * 4);

    // Add dependencies and references
    meta.add_dependency("assets/textures/test_normal.png", &generate_guid());
    meta.add_reference("assets/prefabs/test.prefab", Some("texture"));

    // Update timestamp and hash
    meta.update_timestamp();
    meta.update_hash("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");

    // Write to file
    meta.to_file(&meta_path).unwrap();

    // Read back from file
    let read_meta = MetaFile::from_file(&meta_path).unwrap();

    // Verify fields
    assert_eq!(read_meta.version, "1.0");
    assert_eq!(read_meta.asset.r#type, "texture");
    assert_eq!(read_meta.asset.path, "assets/textures/test.png");
    assert_eq!(read_meta.asset.name, "Test Texture");
    assert_eq!(read_meta.asset.size, 1024 * 1024 * 4);
    assert_eq!(read_meta.dependencies.len(), 1);
    assert_eq!(read_meta.references.len(), 1);
    assert_eq!(read_meta.hash, Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()));

    // Clean up
    fs::remove_file(meta_path).unwrap();
}

#[test]
fn test_generate_guid() {
    let guid1 = generate_guid();
    let guid2 = generate_guid();
    assert_ne!(guid1, guid2);
    assert_eq!(guid1.len(), 36); // UUID v7 length
}
