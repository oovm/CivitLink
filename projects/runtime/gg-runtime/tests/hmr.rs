use gg_bytecode::BytecodeValue;
use gg_runtime::{ComponentStateSnapshot, HmrManager, HmrMigrationStrategy, StateMigrator, StateSnapshot};
use std::collections::HashMap;

#[test]
fn test_hmr_manager_new() {
    let manager = HmrManager::new();
    assert_eq!(manager.migration_strategy(), HmrMigrationStrategy::FullReset);
}

#[test]
fn test_hmr_manager_set_migration_strategy() {
    let mut manager = HmrManager::new();
    manager.set_migration_strategy(HmrMigrationStrategy::Incremental);
    assert_eq!(manager.migration_strategy(), HmrMigrationStrategy::Incremental);
}

#[test]
fn test_hmr_manager_push_and_process_script() {
    let mut manager = HmrManager::new();
    let module = BytecodeValue::Int(42);
    manager.push_event(gg_runtime::HmrEvent::ScriptChanged {
        module_name: "test".to_string(),
        new_module: gg_bytecode::format::BytecodeModule {
            name: "test".to_string(),
            version: 1,
            constants: vec![],
            int_constants: vec![],
            float_constants: vec![],
            string_constants: vec![],
            entity_constants: vec![],
            bool_constants: vec![],
            string_pool: vec![],
            functions: vec![],
            debug_info: None,
            entry_points: vec![],
            function_index: HashMap::new(),
        },
    });

    let result = manager.process_script_reload();
    assert!(result.is_some());
}

#[test]
fn test_hmr_manager_process_asset() {
    let mut manager = HmrManager::new();
    manager.push_event(gg_runtime::HmrEvent::AssetChanged {
        asset_path: "assets/textures/player.png".to_string(),
        new_data: vec![1, 2, 3],
    });

    let changed = manager.process_asset_reload();
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0], "assets/textures/player.png");
}

#[test]
fn test_state_snapshot_with_components() {
    let mut globals = HashMap::new();
    globals.insert("score".to_string(), BytecodeValue::Int(100));

    let mut component_states = HashMap::new();
    component_states.insert(
        "Player".to_string(),
        ComponentStateSnapshot {
            component_type: "Player".to_string(),
            fields: {
                let mut m = HashMap::new();
                m.insert("health".to_string(), BytecodeValue::Int(80));
                m
            },
        },
    );

    let mut migrator = StateMigrator::new();
    migrator.capture_with_components(globals, component_states);

    let snapshot = migrator.restore();
    assert!(snapshot.is_some());
    let snap = snapshot.unwrap();
    assert_eq!(snap.globals.len(), 1);
    assert_eq!(snap.component_states.len(), 1);
}

#[test]
fn test_state_migrator_capture_restore() {
    let mut globals = HashMap::new();
    globals.insert("x".to_string(), BytecodeValue::Int(10));

    let mut migrator = StateMigrator::new();
    migrator.capture(globals);

    let snapshot = migrator.restore();
    assert!(snapshot.is_some());
    assert_eq!(snapshot.unwrap().globals.get("x"), Some(&BytecodeValue::Int(10)));
}

#[test]
fn test_state_migrator_clear() {
    let mut migrator = StateMigrator::new();
    migrator.capture(HashMap::new());
    migrator.clear();
    assert!(migrator.restore().is_none());
}
