use gg_meta::{MetaFile, MetaGenerator};
use std::fs;

mod tests_meta_generator {
    use super::*;

    #[test]
    fn test_default_type_map() {
        let generator = MetaGenerator::new();

        assert_eq!(generator.get_asset_type("png"), "Texture");
        assert_eq!(generator.get_asset_type("jpg"), "Texture");
        assert_eq!(generator.get_asset_type("jpeg"), "Texture");
        assert_eq!(generator.get_asset_type("bmp"), "Texture");
        assert_eq!(generator.get_asset_type("webp"), "Texture");

        assert_eq!(generator.get_asset_type("wav"), "Audio");
        assert_eq!(generator.get_asset_type("mp3"), "Audio");
        assert_eq!(generator.get_asset_type("ogg"), "Audio");
        assert_eq!(generator.get_asset_type("flac"), "Audio");

        assert_eq!(generator.get_asset_type("ttf"), "Font");
        assert_eq!(generator.get_asset_type("otf"), "Font");
        assert_eq!(generator.get_asset_type("woff"), "Font");
        assert_eq!(generator.get_asset_type("woff2"), "Font");

        assert_eq!(generator.get_asset_type("v"), "Script");
        assert_eq!(generator.get_asset_type("vx"), "Script");
        assert_eq!(generator.get_asset_type("script"), "Script");

        assert_eq!(generator.get_asset_type("glsl"), "Shader");
        assert_eq!(generator.get_asset_type("vert"), "Shader");
        assert_eq!(generator.get_asset_type("frag"), "Shader");
        assert_eq!(generator.get_asset_type("gs"), "Shader");

        assert_eq!(generator.get_asset_type("scene"), "Scene");
        assert_eq!(generator.get_asset_type("prefab"), "Prefab");

        assert_eq!(generator.get_asset_type("toml"), "Config");
        assert_eq!(generator.get_asset_type("json"), "Config");
        assert_eq!(generator.get_asset_type("yaml"), "Config");
        assert_eq!(generator.get_asset_type("yml"), "Config");

        assert_eq!(generator.get_asset_type("anim"), "Animation");

        assert_eq!(generator.get_asset_type("spine"), "Spine");
        assert_eq!(generator.get_asset_type("atlas"), "Spine");

        assert_eq!(generator.get_asset_type("xlsx"), "Sheet");
        assert_eq!(generator.get_asset_type("csv"), "Sheet");
        assert_eq!(generator.get_asset_type("tsv"), "Sheet");
    }

    #[test]
    fn test_register_type() {
        let mut generator = MetaGenerator::new();

        assert_eq!(generator.get_asset_type("custom"), "Unknown");

        generator.register_type("custom", "CustomType");
        assert_eq!(generator.get_asset_type("custom"), "CustomType");
    }

    #[test]
    fn test_get_asset_type_unknown() {
        let generator = MetaGenerator::new();
        assert_eq!(generator.get_asset_type("xyz"), "Unknown");
        assert_eq!(generator.get_asset_type(""), "Unknown");
    }

    #[test]
    fn test_generate_for_file_creates_meta() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.png");
        fs::write(&file_path, b"fake png data").unwrap();

        let generator = MetaGenerator::new();
        let result = generator.generate_for_file(&file_path).unwrap();

        assert!(result);

        let meta_path = dir.path().join("test.png.meta");
        assert!(meta_path.exists());

        let meta = MetaFile::from_file(&meta_path).unwrap();
        assert_eq!(meta.asset.r#type, "Texture");
        assert_eq!(meta.asset.name, "test.png");
        assert_eq!(meta.asset.size, 13);
    }

    #[test]
    fn test_generate_for_file_skips_existing() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.wav");
        fs::write(&file_path, b"fake audio").unwrap();

        let generator = MetaGenerator::new();

        let first = generator.generate_for_file(&file_path).unwrap();
        assert!(first);

        let second = generator.generate_for_file(&file_path).unwrap();
        assert!(!second);
    }

    #[test]
    fn test_generate_for_directory_recursive() {
        let dir = tempfile::tempdir().unwrap();

        let sub_dir = dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        fs::write(dir.path().join("top.png"), b"img").unwrap();
        fs::write(sub_dir.join("nested.wav"), b"audio").unwrap();
        fs::write(sub_dir.join("data.toml"), b"config").unwrap();

        let generator = MetaGenerator::new();
        let count = generator.generate_for_directory(dir.path(), true).unwrap();

        assert_eq!(count, 3);
        assert!(dir.path().join("top.png.meta").exists());
        assert!(sub_dir.join("nested.wav.meta").exists());
        assert!(sub_dir.join("data.toml.meta").exists());
    }

    #[test]
    fn test_generate_for_directory_non_recursive() {
        let dir = tempfile::tempdir().unwrap();

        let sub_dir = dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        fs::write(dir.path().join("top.png"), b"img").unwrap();
        fs::write(sub_dir.join("nested.wav"), b"audio").unwrap();

        let generator = MetaGenerator::new();
        let count = generator.generate_for_directory(dir.path(), false).unwrap();

        assert_eq!(count, 1);
        assert!(dir.path().join("top.png.meta").exists());
        assert!(!sub_dir.join("nested.wav.meta").exists());
    }

    #[test]
    fn test_generate_for_directory_skips_meta_files() {
        let dir = tempfile::tempdir().unwrap();

        fs::write(dir.path().join("existing.meta"), b"meta content").unwrap();

        let generator = MetaGenerator::new();
        let count = generator.generate_for_directory(dir.path(), false).unwrap();

        assert_eq!(count, 0);
    }
}
