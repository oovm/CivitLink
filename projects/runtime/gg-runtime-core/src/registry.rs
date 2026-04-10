//! 组件注册表模块
//!
//! 管理组件类型的动态注册和访问，替代硬编码的组件类型匹配。
//! 脚本虚拟机和 WASM 沙箱通过注册表按名称操作组件。

use gg_bytecode::BytecodeValue;
use gg_ecs::{Entity, World};
use std::collections::HashMap;

/// 组件字段访问器 trait
///
/// 定义对组件字段的读取和写入操作，
/// 用于脚本虚拟机通过名称动态访问组件属性。
pub trait ComponentAccessor: Send + Sync {
    /// 获取组件字段值
    fn get_field(&self, world: &World, entity: Entity, field: &str) -> Option<BytecodeValue>;

    /// 设置组件字段值
    fn set_field(&self, world: &mut World, entity: Entity, field: &str, value: BytecodeValue);

    /// 添加默认组件到实体
    fn add_default(&self, world: &mut World, entity: Entity);
}

/// 组件注册表
///
/// 管理组件类型的动态注册和访问，替代硬编码的组件类型匹配。
/// 脚本虚拟机和 WASM 沙箱通过注册表按名称操作组件。
pub struct ComponentRegistry {
    /// 组件类型名到访问器的映射
    accessors: HashMap<String, Box<dyn ComponentAccessor>>,
}

impl ComponentRegistry {
    /// 创建新的组件注册表
    pub fn new() -> Self {
        Self { accessors: HashMap::new() }
    }

    /// 注册组件类型
    pub fn register(&mut self, type_name: &str, accessor: Box<dyn ComponentAccessor>) {
        self.accessors.insert(type_name.to_string(), accessor);
    }

    /// 获取组件字段值
    pub fn get_field(&self, world: &World, entity: Entity, type_name: &str, field: &str) -> Option<BytecodeValue> {
        self.accessors.get(type_name).and_then(|a| a.get_field(world, entity, field))
    }

    /// 设置组件字段值
    pub fn set_field(&self, world: &mut World, entity: Entity, type_name: &str, field: &str, value: BytecodeValue) {
        if let Some(accessor) = self.accessors.get(type_name) {
            accessor.set_field(world, entity, field, value);
        }
    }

    /// 添加默认组件到实体
    pub fn add_default(&self, world: &mut World, entity: Entity, type_name: &str) {
        if let Some(accessor) = self.accessors.get(type_name) {
            accessor.add_default(world, entity);
        }
    }

    /// 检查组件类型是否已注册
    pub fn is_registered(&self, type_name: &str) -> bool {
        self.accessors.contains_key(type_name)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gg_bytecode::BytecodeValue;
    use gg_ecs::World;

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
        } else {
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
}
