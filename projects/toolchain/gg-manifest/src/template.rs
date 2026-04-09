use crate::manifest::{
    DisplaySection, EngineManifest, EngineSection, GameType, ModulesSection, PlatformEntry,
    ToolchainSection,
};

/// 创建 VisualNovel 类型的默认引擎清单
pub fn visual_novel_template(name: &str) -> EngineManifest {
    EngineManifest {
        engine: EngineSection {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            game_type: GameType::VisualNovel,
        },
        modules: ModulesSection {
            gom: "VisualNovel".to_string(),
            vm: "Wasm".to_string(),
            render: "SpriteStack".to_string(),
            plugins: vec![
                "dialogue".to_string(),
                "portrait".to_string(),
                "scene-transition".to_string(),
                "save".to_string(),
            ],
        },
        platforms: vec![PlatformEntry {
            target: "x86_64-pc-windows-msvc".to_string(),
            name: "Windows".to_string(),
            features: Vec::new(),
        }],
        toolchain: ToolchainSection {
            compiler_steps: Vec::new(),
            editor_panels: Vec::new(),
        },
        display: DisplaySection {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: name.to_string(),
        },
    }
}

/// 创建 ARPG 类型的默认引擎清单
pub fn arpg_template(name: &str) -> EngineManifest {
    EngineManifest {
        engine: EngineSection {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            game_type: GameType::ARPG,
        },
        modules: ModulesSection {
            gom: "ARPG".to_string(),
            vm: "Wasm".to_string(),
            render: "Immediate2D".to_string(),
            plugins: vec![
                "dialogue".to_string(),
                "save".to_string(),
            ],
        },
        platforms: vec![PlatformEntry {
            target: "x86_64-pc-windows-msvc".to_string(),
            name: "Windows".to_string(),
            features: Vec::new(),
        }],
        toolchain: ToolchainSection {
            compiler_steps: Vec::new(),
            editor_panels: Vec::new(),
        },
        display: DisplaySection {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: name.to_string(),
        },
    }
}

/// 创建自定义类型的最小化引擎清单
pub fn custom_template(name: &str) -> EngineManifest {
    EngineManifest {
        engine: EngineSection {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            game_type: GameType::Custom(String::new()),
        },
        modules: ModulesSection {
            gom: String::new(),
            vm: "Wasm".to_string(),
            render: "SpriteStack".to_string(),
            plugins: Vec::new(),
        },
        platforms: vec![PlatformEntry {
            target: "x86_64-pc-windows-msvc".to_string(),
            name: "Windows".to_string(),
            features: Vec::new(),
        }],
        toolchain: ToolchainSection {
            compiler_steps: Vec::new(),
            editor_panels: Vec::new(),
        },
        display: DisplaySection {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: name.to_string(),
        },
    }
}
