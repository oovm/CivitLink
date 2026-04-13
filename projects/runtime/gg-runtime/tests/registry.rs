use gg_bytecode::BytecodeValue;
use gg_ecs::World;
use gg_runtime_core::{
    Entity,
    registry::{ComponentAccessor, ComponentRegistry},
};

struct TestAccessor;

impl ComponentAccessor for TestAccessor {
    fn get_field(&self, _world: &World, _entity: Entity, field: &str) -> Option<BytecodeValue> {
        match field {
            "x" => Some(BytecodeValue::Int(42)),
            _ => None,
        }
    }

    fn set_field(&self, _world: &mut World, _entity: Entity, _field: &str, _value: BytecodeValue) {}

    fn add_default(&self, _world: &mut World, _entity: Entity) {}
}

#[test]
fn test_component_registry_new() {
    let registry = ComponentRegistry::new();
    assert!(!registry.is_registered("test"));
}

#[test]
fn test_component_registry_register() {
    let mut registry = ComponentRegistry::new();
    assert!(!registry.is_registered("Position"));
    registry.register("Position", Box::new(TestAccessor));
    assert!(registry.is_registered("Position"));
}

#[test]
fn test_component_registry_get_field_unregistered() {
    let registry = ComponentRegistry::new();
    let mut world = World::new();
    let entity = world.spawn().id();
    let result = registry.get_field(&world, entity, "Nonexistent", "x");
    assert!(result.is_none());
}

#[test]
fn test_component_registry_get_field_registered() {
    let mut registry = ComponentRegistry::new();
    registry.register("Position", Box::new(TestAccessor));
    let mut world = World::new();
    let entity = world.spawn().id();
    let result = registry.get_field(&world, entity, "Position", "x");
    assert!(result.is_some());
    if let Some(BytecodeValue::Int(val)) = result {
        assert_eq!(val, 42);
    }
    else {
        panic!("Expected BytecodeValue::Int(42)");
    }
}

#[test]
fn test_component_registry_get_field_missing_field() {
    let mut registry = ComponentRegistry::new();
    registry.register("Position", Box::new(TestAccessor));
    let mut world = World::new();
    let entity = world.spawn().id();
    let result = registry.get_field(&world, entity, "Position", "nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_component_registry_set_field_unregistered() {
    let registry = ComponentRegistry::new();
    let mut world = World::new();
    let entity = world.spawn().id();
    registry.set_field(&mut world, entity, "Nonexistent", "x", BytecodeValue::Int(10));
}

#[test]
fn test_component_registry_add_default_unregistered() {
    let registry = ComponentRegistry::new();
    let mut world = World::new();
    let entity = world.spawn().id();
    registry.add_default(&mut world, entity, "Nonexistent");
}

#[test]
fn test_component_registry_default() {
    let registry = ComponentRegistry::default();
    assert!(!registry.is_registered("test"));
}

#[test]
fn test_component_registry_register_overwrite() {
    let mut registry = ComponentRegistry::new();
    registry.register("Position", Box::new(TestAccessor));
    assert!(registry.is_registered("Position"));
    registry.register("Position", Box::new(TestAccessor));
    assert!(registry.is_registered("Position"));
}
