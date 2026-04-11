use gg_factory::generator::{name_to_package_name, name_to_struct_name, generate_system_registration, generate_component_registration, generate_asset_loader_registration};
use gg_factory::EngineFactory;
use gg_manifest::{
    DisplaySection, EngineManifest, EngineSection, GameType, ModulesSection, PlatformEntry,
    ToolchainSection,
};

fn make_visual_novel_manifest() -> EngineManifest {
    EngineManifest {
        extends: None,
        engine: EngineSection {
            name: "Test VN".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            game_type: GameType::VisualNovel,
        },
        modules: ModulesSection {
            gom: "VisualNovel".to_string(),
            vm: "Wasm".to_string(),
            render: "SpriteStack".to_string(),
            plugins: vec!["dialogue".to_string(), "portrait".to_string(), "scene-transition".to_string()],
        },
        platforms: vec![PlatformEntry {
            target: "x86_64-pc-windows-msvc".to_string(),
            name: "Windows".to_string(),
            features: vec![],
        }],
        toolchain: ToolchainSection {
            compiler_steps: vec![],
            editor_panels: vec![],
        },
        display: DisplaySection {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: String::new(),
        },
    }
}

fn make_stg_manifest() -> EngineManifest {
    EngineManifest {
        extends: None,
        engine: EngineSection {
            name: "Test STG".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            game_type: GameType::STG,
        },
        modules: ModulesSection {
            gom: "STG".to_string(),
            vm: "Wasm".to_string(),
            render: "SpriteStack".to_string(),
            plugins: vec![],
        },
        platforms: vec![PlatformEntry {
            target: "x86_64-pc-windows-msvc".to_string(),
            name: "Windows".to_string(),
            features: vec![],
        }],
        toolchain: ToolchainSection {
            compiler_steps: vec![],
            editor_panels: vec![],
        },
        display: DisplaySection {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: String::new(),
        },
    }
}

fn make_platformer_manifest() -> EngineManifest {
    EngineManifest {
        extends: None,
        engine: EngineSection {
            name: "Test Platformer".to_string(),
            version: "0.1.0".to_string(),
            description: String::new(),
            game_type: GameType::ARPG,
        },
        modules: ModulesSection {
            gom: "Platformer".to_string(),
            vm: "Wasm".to_string(),
            render: "SpriteStack".to_string(),
            plugins: vec![],
        },
        platforms: vec![PlatformEntry {
            target: "x86_64-pc-windows-msvc".to_string(),
            name: "Windows".to_string(),
            features: vec![],
        }],
        toolchain: ToolchainSection {
            compiler_steps: vec![],
            editor_panels: vec![],
        },
        display: DisplaySection {
            width: 1280,
            height: 720,
            fullscreen: false,
            title: String::new(),
        },
    }
}

#[test]
fn test_name_to_struct_name_spaces() {
    assert_eq!(name_to_struct_name("My Galgame"), "MyGalgameEngine");
}

#[test]
fn test_name_to_struct_name_hyphens() {
    assert_eq!(name_to_struct_name("my-galgame"), "MyGalgameEngine");
}

#[test]
fn test_name_to_struct_name_underscores() {
    assert_eq!(name_to_struct_name("my_galgame"), "MyGalgameEngine");
}

#[test]
fn test_name_to_struct_name_single_word() {
    assert_eq!(name_to_struct_name("Galgame"), "GalgameEngine");
}

#[test]
fn test_name_to_package_name() {
    assert_eq!(name_to_package_name("My Galgame"), "my-galgame");
}

#[test]
fn test_name_to_package_name_single() {
    assert_eq!(name_to_package_name("Galgame"), "galgame");
}

#[test]
fn test_generated_engine_rs_contains_app_builder() {
    let manifest = make_visual_novel_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-app-builder");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("use gg_runtime_core::App;"), "engine.rs should import App from gg_runtime_core");
    assert!(engine_rs.contains("let mut app = App::new();"), "engine.rs should create App with builder pattern");
    assert!(engine_rs.contains("app.run()"), "engine.rs should call app.run()");
    assert!(engine_rs.contains("pub fn run() -> GResult<()>"), "engine.rs should have a run() function");
}

#[test]
fn test_generated_engine_rs_contains_system_registration() {
    let manifest = make_visual_novel_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-system-reg");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_plugin_dialogue::systems::dialogue_system);"), "engine.rs should register DialogueSystem for VisualNovel");
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_plugin_portrait::systems::portrait_system);"), "engine.rs should register PortraitSystem for VisualNovel");
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_plugin_scene_transition::systems::scene_transition_system);"), "engine.rs should register SceneTransitionSystem for VisualNovel");
}

#[test]
fn test_generated_engine_rs_stg_systems() {
    let manifest = make_stg_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-stg-systems");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_ecs::systems::bullet_system);"), "engine.rs should register BulletSystem for STG");
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_ecs::systems::collision_system);"), "engine.rs should register CollisionSystem for STG");
}

#[test]
fn test_generated_engine_rs_platformer_systems() {
    let manifest = make_platformer_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-platformer-systems");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_ecs::systems::movement_system);"), "engine.rs should register MovementSystem for Platformer");
    assert!(engine_rs.contains("app.add_system(SystemSet::Update, gg_ecs::systems::physics_system);"), "engine.rs should register PhysicsSystem for Platformer");
}

#[test]
fn test_generated_engine_rs_contains_component_registration() {
    let manifest = make_visual_novel_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-component-reg");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("app.register_component::<gg_plugin_dialogue::components::DialogueComponent>();"), "engine.rs should register DialogueComponent for VisualNovel");
    assert!(engine_rs.contains("app.register_component::<gg_plugin_portrait::components::PortraitComponent>();"), "engine.rs should register PortraitComponent for VisualNovel");
    assert!(engine_rs.contains("app.register_component::<gg_plugin_scene_transition::components::SceneTransitionComponent>();"), "engine.rs should register SceneTransitionComponent for VisualNovel");
}

#[test]
fn test_generated_engine_rs_stg_components() {
    let manifest = make_stg_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-stg-components");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("app.register_component::<gg_ecs::components::BulletComponent>();"), "engine.rs should register BulletComponent for STG");
    assert!(engine_rs.contains("app.register_component::<gg_ecs::components::HitboxComponent>();"), "engine.rs should register HitboxComponent for STG");
}

#[test]
fn test_generated_engine_rs_contains_asset_loader_registration() {
    let manifest = make_visual_novel_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-loader-reg");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let engine_rs = std::fs::read_to_string(output_dir.join("src/engine.rs")).unwrap();
    assert!(engine_rs.contains("app.register_loader::<gg_asset::loader::TextLoader>();"), "engine.rs should register TextLoader");
    assert!(engine_rs.contains("app.register_loader::<gg_asset::loader::BinaryLoader>();"), "engine.rs should register BinaryLoader");
    assert!(engine_rs.contains("app.register_loader::<gg_asset::loader::ImageLoader>();"), "engine.rs should register ImageLoader");
    assert!(engine_rs.contains("app.register_loader::<gg_asset::loader::AudioLoader>();"), "engine.rs should register AudioLoader");
}

#[test]
fn test_generated_cargo_toml_has_correct_dependency_paths() {
    let manifest = make_visual_novel_manifest();
    let output_dir = std::env::temp_dir().join("gg-factory-test-cargo-paths");
    let _ = std::fs::remove_dir_all(&output_dir);

    EngineFactory::generate(&manifest, &output_dir).unwrap();

    let cargo_toml = std::fs::read_to_string(output_dir.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("path = \"../../projects/core/gg-core\""), "Cargo.toml should have correct gg-core path");
    assert!(cargo_toml.contains("path = \"../../projects/core/gg-ecs\""), "Cargo.toml should have correct gg-ecs path");
    assert!(cargo_toml.contains("path = \"../../projects/core/gg-asset\""), "Cargo.toml should have correct gg-asset path");
    assert!(cargo_toml.contains("path = \"../../projects/runtime/gg-runtime-core\""), "Cargo.toml should have gg-runtime-core dependency");
    assert!(cargo_toml.contains("path = \"../../projects/core/gg-schedule\""), "Cargo.toml should have gg-schedule dependency");
    assert!(cargo_toml.contains("path = \"../../projects/plugins/gg-plugin-dialogue\""), "Cargo.toml should have correct plugin path");
    assert!(!cargo_toml.contains("path = \"../../core/"), "Cargo.toml should not have old-style core paths");
    assert!(!cargo_toml.contains("path = \"../../plugins/"), "Cargo.toml should not have old-style plugin paths");
    assert!(!cargo_toml.contains("path = \"../../platforms/"), "Cargo.toml should not have old-style platform paths");
}

#[test]
fn test_generate_system_registration_visual_novel() {
    let result = generate_system_registration("VisualNovel");
    assert!(result.contains("dialogue_system"));
    assert!(result.contains("portrait_system"));
    assert!(result.contains("scene_transition_system"));
}

#[test]
fn test_generate_system_registration_stg() {
    let result = generate_system_registration("STG");
    assert!(result.contains("bullet_system"));
    assert!(result.contains("collision_system"));
}

#[test]
fn test_generate_system_registration_platformer() {
    let result = generate_system_registration("Platformer");
    assert!(result.contains("movement_system"));
    assert!(result.contains("physics_system"));
}

#[test]
fn test_generate_system_registration_empty() {
    let result = generate_system_registration("");
    assert!(result.is_empty());
}

#[test]
fn test_generate_component_registration_visual_novel() {
    let result = generate_component_registration("VisualNovel");
    assert!(result.contains("DialogueComponent"));
    assert!(result.contains("PortraitComponent"));
    assert!(result.contains("SceneTransitionComponent"));
}

#[test]
fn test_generate_component_registration_stg() {
    let result = generate_component_registration("STG");
    assert!(result.contains("BulletComponent"));
    assert!(result.contains("HitboxComponent"));
}

#[test]
fn test_generate_component_registration_empty() {
    let result = generate_component_registration("");
    assert!(result.is_empty());
}

#[test]
fn test_generate_asset_loader_registration() {
    let result = generate_asset_loader_registration();
    assert!(result.contains("TextLoader"));
    assert!(result.contains("BinaryLoader"));
    assert!(result.contains("ImageLoader"));
    assert!(result.contains("AudioLoader"));
}
