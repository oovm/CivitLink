#![warn(missing_docs)]

//! GG 引擎清单模块
//! 定义引擎清单格式，提供解析、验证和模板功能

/// 引擎清单类型定义
pub mod manifest;
/// 引擎清单模板
pub mod template;

pub use manifest::{
    DisplaySection, EngineManifest, EngineSection, GameType, ModulesSection, PlatformEntry,
    ToolchainSection,
};
pub use template::{arpg_template, custom_template, visual_novel_template};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml() {
        let toml_str = r#"
[engine]
name = "MyGame"
version = "1.0.0"
description = "A test game"
game_type = "VisualNovel"

[modules]
gom = "VisualNovel"
vm = "Wasm"
render = "SpriteStack"
plugins = ["dialogue", "portrait"]

[[platforms]]
target = "x86_64-pc-windows-msvc"
name = "Windows"

[toolchain]
compiler_steps = []
editor_panels = []

[display]
width = 1280
height = 720
fullscreen = false
title = "MyGame"
"#;
        let manifest = EngineManifest::from_toml(toml_str).expect("Failed to parse valid TOML");
        assert_eq!(manifest.engine.name, "MyGame");
        assert_eq!(manifest.engine.version, "1.0.0");
        assert_eq!(manifest.engine.description, "A test game");
        assert_eq!(manifest.engine.game_type, GameType::VisualNovel);
        assert_eq!(manifest.modules.gom, "VisualNovel");
        assert_eq!(manifest.modules.vm, "Wasm");
        assert_eq!(manifest.modules.render, "SpriteStack");
        assert_eq!(manifest.modules.plugins, vec!["dialogue", "portrait"]);
        assert_eq!(manifest.platforms.len(), 1);
        assert_eq!(manifest.platforms[0].target, "x86_64-pc-windows-msvc");
    }

    #[test]
    fn test_validation_rejects_empty_name() {
        let mut manifest = visual_novel_template("TestGame");
        manifest.engine.name = String::new();
        let result = manifest.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, gg_core::GErrorKind::Runtime);
        assert!(err.message.contains("engine.name"));
    }

    #[test]
    fn test_validation_rejects_empty_plugins() {
        let mut manifest = visual_novel_template("TestGame");
        manifest.modules.plugins = Vec::new();
        let result = manifest.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, gg_core::GErrorKind::Runtime);
        assert!(err.message.contains("plugins"));
    }

    #[test]
    fn test_validation_rejects_empty_platforms() {
        let mut manifest = visual_novel_template("TestGame");
        manifest.platforms = Vec::new();
        let result = manifest.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, gg_core::GErrorKind::Runtime);
        assert!(err.message.contains("platforms"));
    }

    #[test]
    fn test_visual_novel_template_is_valid() {
        let manifest = visual_novel_template("TestVN");
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.engine.game_type, GameType::VisualNovel);
        assert_eq!(manifest.modules.gom, "VisualNovel");
        assert_eq!(manifest.modules.render, "SpriteStack");
        assert_eq!(manifest.modules.vm, "Wasm");
        assert!(manifest.modules.plugins.contains(&"dialogue".to_string()));
        assert!(manifest.modules.plugins.contains(&"portrait".to_string()));
        assert!(manifest.modules.plugins.contains(&"scene-transition".to_string()));
        assert!(manifest.modules.plugins.contains(&"save".to_string()));
    }

    #[test]
    fn test_arpg_template_is_valid() {
        let manifest = arpg_template("TestARPG");
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.engine.game_type, GameType::ARPG);
        assert_eq!(manifest.modules.gom, "ARPG");
        assert_eq!(manifest.modules.render, "Immediate2D");
        assert_eq!(manifest.modules.vm, "Wasm");
        assert!(manifest.modules.plugins.contains(&"dialogue".to_string()));
        assert!(manifest.modules.plugins.contains(&"save".to_string()));
    }

    #[test]
    fn test_custom_template_has_empty_plugins() {
        let manifest = custom_template("TestCustom");
        assert_eq!(manifest.engine.game_type, GameType::Custom(String::new()));
        assert_eq!(manifest.modules.gom, "");
        assert!(manifest.modules.plugins.is_empty());
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_round_trip_serialize_parse() {
        let original = visual_novel_template("RoundTrip");
        let toml_str = toml::to_string(&original).expect("Failed to serialize");
        let parsed = EngineManifest::from_toml(&toml_str).expect("Failed to parse back");
        assert_eq!(parsed.engine.name, original.engine.name);
        assert_eq!(parsed.engine.version, original.engine.version);
        assert_eq!(parsed.engine.game_type, original.engine.game_type);
        assert_eq!(parsed.modules.gom, original.modules.gom);
        assert_eq!(parsed.modules.vm, original.modules.vm);
        assert_eq!(parsed.modules.render, original.modules.render);
        assert_eq!(parsed.modules.plugins, original.modules.plugins);
        assert_eq!(parsed.platforms.len(), original.platforms.len());
        assert_eq!(parsed.platforms[0].target, original.platforms[0].target);
        assert_eq!(parsed.display.width, original.display.width);
        assert_eq!(parsed.display.height, original.display.height);
    }
}
