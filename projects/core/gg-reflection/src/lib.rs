#![warn(missing_docs)]

//! GG 引擎反射模块
//! 提供运行时类型信息、属性编辑和类型注册功能

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// 反射基础 trait，为类型提供运行时自省能力
pub trait PartialReflect {
    /// 获取 `Any` 引用，用于向下转型
    fn as_any(&self) -> &dyn Any;

    /// 获取 `Any` 可变引用，用于向下转型
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 获取类型名称
    fn type_name(&self) -> &'static str;

    /// 克隆反射值
    fn clone_reflect(&self) -> Box<dyn PartialReflect>;

    /// 获取字段名称列表
    fn field_names(&self) -> &[&str] {
        &[]
    }

    /// 获取指定名称的字段值
    fn field(&self, _name: &str) -> Option<&dyn PartialReflect> {
        None
    }

    /// 获取指定名称的字段可变引用
    fn field_mut(&mut self, _name: &str) -> Option<&mut dyn PartialReflect> {
        None
    }

    /// 尝试从另一个反射值赋值到自身
    ///
    /// 默认实现返回类型不匹配错误，各类型应自行实现以支持动态赋值
    fn try_assign(&mut self, _source: &dyn PartialReflect) -> Result<(), String> {
        Err(format!("try_assign not implemented for {}", self.type_name()))
    }
}

/// 为 `Copy` 基本类型生成 `PartialReflect` 实现
macro_rules! impl_reflect_primitive {
    ($ty:ty) => {
        impl PartialReflect for $ty {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn type_name(&self) -> &'static str {
                std::any::type_name::<Self>()
            }

            fn clone_reflect(&self) -> Box<dyn PartialReflect> {
                Box::new(*self)
            }

            fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
                if let Some(val) = source.as_any().downcast_ref::<Self>() {
                    *self = *val;
                    Ok(())
                } else {
                    Err(format!(
                        "type mismatch: expected {}, got {}",
                        self.type_name(),
                        source.type_name()
                    ))
                }
            }
        }
    };
}

impl_reflect_primitive!(f32);
impl_reflect_primitive!(f64);
impl_reflect_primitive!(i8);
impl_reflect_primitive!(i16);
impl_reflect_primitive!(i32);
impl_reflect_primitive!(i64);
impl_reflect_primitive!(i128);
impl_reflect_primitive!(isize);
impl_reflect_primitive!(u8);
impl_reflect_primitive!(u16);
impl_reflect_primitive!(u32);
impl_reflect_primitive!(u64);
impl_reflect_primitive!(u128);
impl_reflect_primitive!(usize);
impl_reflect_primitive!(bool);

impl PartialReflect for String {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn clone_reflect(&self) -> Box<dyn PartialReflect> {
        Box::new(self.clone())
    }

    fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
        if let Some(val) = source.as_any().downcast_ref::<Self>() {
            *self = val.clone();
            Ok(())
        } else {
            Err(format!(
                "type mismatch: expected {}, got {}",
                self.type_name(),
                source.type_name()
            ))
        }
    }
}

/// 枚举反射 trait，为枚举类型提供运行时变体查询和字段访问能力
pub trait EnumReflect: PartialReflect {
    /// 获取所有变体名称
    fn variants(&self) -> &[&str];

    /// 获取当前活跃变体的名称
    fn variant_name(&self) -> &str;

    /// 获取当前变体中指定索引字段的反射引用
    fn field_at(&self, index: usize) -> Option<&dyn PartialReflect>;

    /// 获取当前变体中指定索引字段的可变反射引用
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    /// 获取当前变体的字段数量
    fn field_count(&self) -> usize;

    /// 尝试切换到指定名称的变体
    ///
    /// 如果变体名称有效，切换到该变体并返回 Ok(())，
    /// 否则返回错误信息。
    fn set_variant(&mut self, name: &str) -> Result<(), String>;
}

/// 列表反射 trait，为有序集合类型提供动态元素访问和修改能力
pub trait ListReflect: PartialReflect {
    /// 获取列表长度
    fn len(&self) -> usize;

    /// 判断列表是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 获取指定索引处元素的反射引用
    fn get(&self, index: usize) -> Option<&dyn PartialReflect>;

    /// 获取指定索引处元素的可变反射引用
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    /// 向列表末尾添加元素
    fn push(&mut self, value: Box<dyn PartialReflect>);

    /// 移除指定索引处的元素
    fn remove(&mut self, index: usize) -> Option<Box<dyn PartialReflect>>;
}

/// 映射反射 trait，为键值对集合类型提供动态元素访问和修改能力
pub trait MapReflect: PartialReflect {
    /// 获取映射中的键值对数量
    fn len(&self) -> usize;

    /// 判断映射是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 根据键获取值的反射引用
    fn get(&self, key: &str) -> Option<&dyn PartialReflect>;

    /// 根据键获取值的可变反射引用
    fn get_mut(&mut self, key: &str) -> Option<&mut dyn PartialReflect>;

    /// 插入键值对
    fn insert(&mut self, key: String, value: Box<dyn PartialReflect>);

    /// 根据键移除键值对
    fn remove(&mut self, key: &str) -> Option<Box<dyn PartialReflect>>;

    /// 获取所有键的列表
    fn keys(&self) -> Vec<String>;
}

impl<T: PartialReflect + Clone + 'static> PartialReflect for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn clone_reflect(&self) -> Box<dyn PartialReflect> {
        Box::new(self.clone())
    }

    fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
        if let Some(val) = source.as_any().downcast_ref::<Self>() {
            *self = val.clone();
            Ok(())
        } else {
            Err(format!(
                "type mismatch: expected {}, got {}",
                self.type_name(),
                source.type_name()
            ))
        }
    }
}

impl<T: PartialReflect + Clone + 'static> ListReflect for Vec<T> {
    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn get(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.as_slice().get(index).map(|v| v as &dyn PartialReflect)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        self.as_mut_slice().get_mut(index).map(|v| v as &mut dyn PartialReflect)
    }

    fn push(&mut self, value: Box<dyn PartialReflect>) {
        if let Some(item) = value.as_any().downcast_ref::<T>() {
            self.push(item.clone());
        }
    }

    fn remove(&mut self, index: usize) -> Option<Box<dyn PartialReflect>> {
        if index < self.len() {
            Some(Box::new(self.remove(index)))
        } else {
            None
        }
    }
}

impl<V: PartialReflect + Clone + 'static> PartialReflect for HashMap<String, V> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn clone_reflect(&self) -> Box<dyn PartialReflect> {
        Box::new(self.clone())
    }

    fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
        if let Some(val) = source.as_any().downcast_ref::<Self>() {
            *self = val.clone();
            Ok(())
        } else {
            Err(format!(
                "type mismatch: expected {}, got {}",
                self.type_name(),
                source.type_name()
            ))
        }
    }
}

impl<V: PartialReflect + Clone + 'static> MapReflect for HashMap<String, V> {
    fn len(&self) -> usize {
        HashMap::len(self)
    }

    fn get(&self, key: &str) -> Option<&dyn PartialReflect> {
        HashMap::get(self, key).map(|v| v as &dyn PartialReflect)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut dyn PartialReflect> {
        HashMap::get_mut(self, key).map(|v| v as &mut dyn PartialReflect)
    }

    fn insert(&mut self, key: String, value: Box<dyn PartialReflect>) {
        if let Some(val) = value.as_any().downcast_ref::<V>() {
            HashMap::insert(self, key, val.clone());
        }
    }

    fn remove(&mut self, key: &str) -> Option<Box<dyn PartialReflect>> {
        HashMap::remove(self, key).map(|v| Box::new(v) as Box<dyn PartialReflect>)
    }

    fn keys(&self) -> Vec<String> {
        self.keys().cloned().collect()
    }
}

/// 运行时类型信息
pub struct TypeInfo {
    /// 类型 ID
    pub type_id: TypeId,
    /// 完整类型名称
    pub type_name: &'static str,
    /// 简短类型名称
    pub short_name: String,
}

impl TypeInfo {
    /// 为类型 `T` 创建 `TypeInfo`
    pub fn new<T: 'static>() -> Self {
        let type_name = std::any::type_name::<T>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name).to_string();
        Self { type_id: TypeId::of::<T>(), type_name, short_name }
    }
}

/// 类型注册信息
pub struct TypeRegistration {
    /// 类型信息
    type_info: TypeInfo,
}

impl TypeRegistration {
    /// 为类型 `T` 创建 `TypeRegistration`
    pub fn new<T: 'static>() -> Self {
        Self { type_info: TypeInfo::new::<T>() }
    }

    /// 获取类型信息引用
    pub fn type_info(&self) -> &TypeInfo {
        &self.type_info
    }

    /// 获取类型 ID
    pub fn type_id(&self) -> TypeId {
        self.type_info.type_id
    }
}

/// 反射组件 trait，提供通过反射动态操作 ECS 组件的能力
pub trait ReflectComponent: Send + Sync {
    /// 向世界中的实体插入反射组件值
    fn insert(&self, world: &mut gg_ecs::World, entity: gg_ecs::Entity, value: Box<dyn PartialReflect>);

    /// 从世界中的实体移除组件
    fn remove(&self, world: &mut gg_ecs::World, entity: gg_ecs::Entity);

    /// 获取世界中实体的组件不可变反射引用
    fn get<'a>(&self, world: &'a gg_ecs::World, entity: gg_ecs::Entity) -> Option<&'a dyn PartialReflect>;

    /// 获取世界中实体的组件可变反射引用
    fn get_mut<'a>(&self, world: &'a mut gg_ecs::World, entity: gg_ecs::Entity) -> Option<&'a mut dyn PartialReflect>;

    /// 克隆世界中实体的组件值为反射装箱值
    fn clone_value(&self, world: &gg_ecs::World, entity: gg_ecs::Entity) -> Option<Box<dyn PartialReflect>>;
}

/// 泛型反射组件实现，为满足 `Component + PartialReflect + Clone` 的类型提供 `ReflectComponent`
pub struct ReflectComponentFor<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: gg_ecs::Component + PartialReflect + Clone + 'static> ReflectComponent for ReflectComponentFor<T> {
    fn insert(&self, world: &mut gg_ecs::World, entity: gg_ecs::Entity, value: Box<dyn PartialReflect>) {
        if let Some(component) = value.as_any().downcast_ref::<T>() {
            let _ = world.add_component(entity, component.clone());
        }
    }

    fn remove(&self, world: &mut gg_ecs::World, entity: gg_ecs::Entity) {
        let _ = world.remove_component::<T>(entity);
    }

    fn get<'a>(&self, world: &'a gg_ecs::World, entity: gg_ecs::Entity) -> Option<&'a dyn PartialReflect> {
        world.get_component::<T>(entity).map(|c| c as &dyn PartialReflect)
    }

    fn get_mut<'a>(&self, world: &'a mut gg_ecs::World, entity: gg_ecs::Entity) -> Option<&'a mut dyn PartialReflect> {
        world.get_component_mut::<T>(entity).map(|c| c as &mut dyn PartialReflect)
    }

    fn clone_value(&self, world: &gg_ecs::World, entity: gg_ecs::Entity) -> Option<Box<dyn PartialReflect>> {
        world.get_component::<T>(entity).map(|c| c.clone_reflect())
    }
}

/// 反射注册表，管理所有可反射类型
pub struct ReflectionRegistry {
    /// 类型注册映射
    registrations: HashMap<TypeId, TypeRegistration>,
    /// 反射组件映射
    reflect_components: HashMap<TypeId, Box<dyn ReflectComponent>>,
}

impl ReflectionRegistry {
    /// 创建空的反射注册表
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
            reflect_components: HashMap::new(),
        }
    }

    /// 注册类型 `T`
    pub fn register<T: PartialReflect + 'static>(&mut self) {
        let registration = TypeRegistration::new::<T>();
        self.registrations.insert(registration.type_id(), registration);
    }

    /// 获取所有注册的映射
    pub fn get(&self) -> &HashMap<TypeId, TypeRegistration> {
        &self.registrations
    }

    /// 获取所有注册的可变映射
    pub fn get_mut(&mut self) -> &mut HashMap<TypeId, TypeRegistration> {
        &mut self.registrations
    }

    /// 根据类型 ID 获取类型信息
    pub fn get_type_info(&self, type_id: TypeId) -> Option<&TypeInfo> {
        self.registrations.get(&type_id).map(|r| r.type_info())
    }

    /// 根据类型 ID 获取类型注册信息
    pub fn get_registration(&self, type_id: TypeId) -> Option<&TypeRegistration> {
        self.registrations.get(&type_id)
    }

    /// 检查类型是否已注册
    pub fn is_registered(&self, type_id: TypeId) -> bool {
        self.registrations.contains_key(&type_id)
    }

    /// 注册反射组件类型 `T`
    ///
    /// 同时调用 `register::<T>()` 注册类型信息，
    /// 并创建 `ReflectComponentFor::<T>` 插入反射组件映射。
    pub fn register_component<T: gg_ecs::Component + PartialReflect + Clone + 'static>(&mut self) {
        self.register::<T>();
        self.reflect_components.insert(TypeId::of::<T>(), Box::new(ReflectComponentFor::<T> { _marker: std::marker::PhantomData }));
    }

    /// 根据类型 ID 获取反射组件操作接口
    pub fn get_component_reflect(&self, type_id: TypeId) -> Option<&dyn ReflectComponent> {
        self.reflect_components.get(&type_id).map(|rc| rc.as_ref())
    }

    /// 根据完整类型名称获取类型注册信息
    pub fn get_by_name(&self, name: &str) -> Option<&TypeRegistration> {
        self.registrations.values().find(|r| r.type_info().type_name == name)
    }

    /// 根据简短类型名称获取类型注册信息
    pub fn get_by_short_name(&self, short_name: &str) -> Option<&TypeRegistration> {
        self.registrations.values().find(|r| r.type_info().short_name == short_name)
    }

    /// 迭代所有类型注册信息
    pub fn iter(&self) -> impl Iterator<Item = &TypeRegistration> {
        self.registrations.values()
    }

    /// 迭代所有反射组件类型
    pub fn iter_component_types(&self) -> impl Iterator<Item = (TypeId, &dyn ReflectComponent)> {
        self.reflect_components.iter().map(|(&type_id, rc)| (type_id, rc.as_ref()))
    }
}

impl Default for ReflectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 属性描述信息
pub struct PropertyInfo {
    /// 属性名称
    pub name: String,
    /// 属性类型名称
    pub type_name: &'static str,
    /// 是否可写
    pub writable: bool,
    /// 属性描述
    pub description: Option<String>,
}

impl PropertyInfo {
    /// 创建新的属性信息
    pub fn new(name: impl Into<String>, type_name: &'static str, writable: bool) -> Self {
        Self { name: name.into(), type_name, writable, description: None }
    }

    /// 设置属性描述并返回自身
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// 属性编辑 trait，提供动态属性读写能力
pub trait PropertyEditor {
    /// 获取所有可编辑属性的信息
    fn editable_properties(&self) -> Vec<PropertyInfo>;

    /// 获取指定名称的属性值
    fn get_property(&self, name: &str) -> Option<&dyn PartialReflect>;

    /// 设置指定名称的属性值
    fn set_property(&mut self, name: &str, value: Box<dyn PartialReflect>) -> Result<(), String>;
}

/// 结构体属性编辑器，为泛型结构体提供属性编辑能力
pub struct StructPropertyEditor<T> {
    /// 被编辑的值
    pub value: T,
}

impl<T> StructPropertyEditor<T> {
    /// 创建新的结构体属性编辑器
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// 获取值的不可变引用
    pub fn get(&self) -> &T {
        &self.value
    }

    /// 获取值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// 取出内部值
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: PartialReflect> PropertyEditor for StructPropertyEditor<T> {
    fn editable_properties(&self) -> Vec<PropertyInfo> {
        self.value
            .field_names()
            .iter()
            .map(|&name| {
                let type_name = self.value.field(name).map(|f| f.type_name()).unwrap_or("unknown");
                PropertyInfo::new(name, type_name, true)
            })
            .collect()
    }

    fn get_property(&self, name: &str) -> Option<&dyn PartialReflect> {
        self.value.field(name)
    }

    fn set_property(&mut self, name: &str, value: Box<dyn PartialReflect>) -> Result<(), String> {
        let field_ref = self.value.field_mut(name).ok_or_else(|| format!("field '{}' not found", name))?;
        field_ref.try_assign(value.as_ref())
    }
}

/// 预导入模块，包含反射系统常用类型
pub mod prelude {
    pub use crate::{
        EnumReflect, ListReflect, MapReflect, PartialReflect, PropertyEditor, PropertyInfo, ReflectComponent,
        ReflectComponentFor, ReflectionRegistry, StructPropertyEditor, TypeInfo, TypeRegistration,
    };
    pub use gg_macros::Reflect;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use gg_ecs::World;
    use std::any::TypeId;

    #[derive(Clone, Debug, PartialEq)]
    struct Health(f32);

    impl PartialReflect for Health {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<Self>()
        }

        fn clone_reflect(&self) -> Box<dyn PartialReflect> {
            Box::new(self.clone())
        }

        fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
            if let Some(val) = source.as_any().downcast_ref::<Self>() {
                *self = val.clone();
                Ok(())
            } else {
                Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Name(String);

    impl PartialReflect for Name {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn type_name(&self) -> &'static str {
            std::any::type_name::<Self>()
        }

        fn clone_reflect(&self) -> Box<dyn PartialReflect> {
            Box::new(self.clone())
        }

        fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
            if let Some(val) = source.as_any().downcast_ref::<Self>() {
                *self = val.clone();
                Ok(())
            } else {
                Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
            }
        }
    }

    #[test]
    fn test_reflect_component_insert_get_remove() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();

        let mut world = World::new();
        let entity = world.spawn().id();

        let rc = registry.get_component_reflect(TypeId::of::<Health>()).unwrap();
        rc.insert(&mut world, entity, Box::new(Health(100.0)));

        let value = rc.get(&world, entity).unwrap();
        let health = value.as_any().downcast_ref::<Health>().unwrap();
        assert_eq!(health.0, 100.0);

        rc.remove(&mut world, entity);
        assert!(rc.get(&world, entity).is_none());
    }

    #[test]
    fn test_reflect_component_get_mut() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();

        let mut world = World::new();
        let entity = world.spawn().id();

        let rc = registry.get_component_reflect(TypeId::of::<Health>()).unwrap();
        rc.insert(&mut world, entity, Box::new(Health(50.0)));

        {
            let value_mut = rc.get_mut(&mut world, entity).unwrap();
            let health = value_mut.as_any_mut().downcast_mut::<Health>().unwrap();
            health.0 = 75.0;
        }

        let value = rc.get(&world, entity).unwrap();
        let health = value.as_any().downcast_ref::<Health>().unwrap();
        assert_eq!(health.0, 75.0);
    }

    #[test]
    fn test_reflect_component_clone_value() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();

        let mut world = World::new();
        let entity = world.spawn().id();

        let rc = registry.get_component_reflect(TypeId::of::<Health>()).unwrap();
        rc.insert(&mut world, entity, Box::new(Health(42.0)));

        let cloned = rc.clone_value(&world, entity).unwrap();
        let health = cloned.as_any().downcast_ref::<Health>().unwrap();
        assert_eq!(health.0, 42.0);
    }

    #[test]
    fn test_registry_get_by_name() {
        let mut registry = ReflectionRegistry::new();
        registry.register::<Health>();

        let type_name = std::any::type_name::<Health>();
        let found = registry.get_by_name(type_name).unwrap();
        assert_eq!(found.type_info().type_name, type_name);

        assert!(registry.get_by_name("nonexistent::Type").is_none());
    }

    #[test]
    fn test_registry_get_by_short_name() {
        let mut registry = ReflectionRegistry::new();
        registry.register::<Health>();

        let found = registry.get_by_short_name("Health").unwrap();
        assert_eq!(found.type_info().short_name, "Health");

        assert!(registry.get_by_short_name("NonExistent").is_none());
    }

    #[test]
    fn test_registry_iter() {
        let mut registry = ReflectionRegistry::new();
        registry.register::<Health>();
        registry.register::<Name>();

        let names: Vec<&str> = registry.iter().map(|r| r.type_info().short_name.as_str()).collect();
        assert!(names.contains(&"Health"));
        assert!(names.contains(&"Name"));
    }

    #[test]
    fn test_registry_iter_component_types() {
        let mut registry = ReflectionRegistry::new();
        registry.register_component::<Health>();
        registry.register_component::<Name>();

        let type_ids: Vec<TypeId> = registry.iter_component_types().map(|(id, _)| id).collect();
        assert!(type_ids.contains(&TypeId::of::<Health>()));
        assert!(type_ids.contains(&TypeId::of::<Name>()));
    }

    #[test]
    fn test_list_reflect_vec() {
        let mut list: Vec<i32> = vec![1, 2, 3];

        assert_eq!(ListReflect::len(&list), 3);
        assert!(!ListReflect::is_empty(&list));

        assert_eq!(ListReflect::get(&list, 0).unwrap().as_any().downcast_ref::<i32>().unwrap(), &1);
        assert_eq!(ListReflect::get(&list, 2).unwrap().as_any().downcast_ref::<i32>().unwrap(), &3);
        assert!(ListReflect::get(&list, 3).is_none());

        ListReflect::push(&mut list, Box::new(4i32));
        assert_eq!(ListReflect::len(&list), 4);

        let removed = ListReflect::remove(&mut list, 1);
        assert_eq!(removed.unwrap().as_any().downcast_ref::<i32>().unwrap(), &2);
        assert_eq!(ListReflect::len(&list), 3);
    }

    #[test]
    fn test_map_reflect_hashmap() {
        let mut map: HashMap<String, i32> = HashMap::new();
        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        assert_eq!(MapReflect::len(&map), 2);
        assert!(!MapReflect::is_empty(&map));

        assert_eq!(MapReflect::get(&map, "a").unwrap().as_any().downcast_ref::<i32>().unwrap(), &1);
        assert!(MapReflect::get(&map, "c").is_none());

        MapReflect::insert(&mut map, "c".to_string(), Box::new(3i32));
        assert_eq!(MapReflect::len(&map), 3);

        let keys = MapReflect::keys(&map);
        assert_eq!(keys.len(), 3);

        let removed = MapReflect::remove(&mut map, "b");
        assert_eq!(removed.unwrap().as_any().downcast_ref::<i32>().unwrap(), &2);
        assert_eq!(MapReflect::len(&map), 2);
    }

    #[test]
    fn test_enum_reflect() {
        #[derive(Clone, Debug, PartialEq)]
        enum Color {
            Red,
            Green,
            Blue,
            Rgb(f32, f32, f32),
        }

        static VARIANTS: &[&str] = &["Red", "Green", "Blue", "Rgb"];

        impl PartialReflect for Color {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn type_name(&self) -> &'static str {
                std::any::type_name::<Self>()
            }

            fn clone_reflect(&self) -> Box<dyn PartialReflect> {
                Box::new(self.clone())
            }

            fn try_assign(&mut self, source: &dyn PartialReflect) -> Result<(), String> {
                if let Some(val) = source.as_any().downcast_ref::<Self>() {
                    *self = val.clone();
                    Ok(())
                } else {
                    Err(format!("type mismatch: expected {}, got {}", self.type_name(), source.type_name()))
                }
            }
        }

        impl EnumReflect for Color {
            fn variants(&self) -> &[&str] {
                VARIANTS
            }

            fn variant_name(&self) -> &str {
                match self {
                    Color::Red => "Red",
                    Color::Green => "Green",
                    Color::Blue => "Blue",
                    Color::Rgb(_, _, _) => "Rgb",
                }
            }

            fn field_at(&self, index: usize) -> Option<&dyn PartialReflect> {
                match self {
                    Color::Rgb(r, g, b) => match index {
                        0 => Some(r as &dyn PartialReflect),
                        1 => Some(g as &dyn PartialReflect),
                        2 => Some(b as &dyn PartialReflect),
                        _ => None,
                    },
                    _ => None,
                }
            }

            fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
                match self {
                    Color::Rgb(r, g, b) => match index {
                        0 => Some(r as &mut dyn PartialReflect),
                        1 => Some(g as &mut dyn PartialReflect),
                        2 => Some(b as &mut dyn PartialReflect),
                        _ => None,
                    },
                    _ => None,
                }
            }

            fn field_count(&self) -> usize {
                match self {
                    Color::Rgb(_, _, _) => 3,
                    _ => 0,
                }
            }

            fn set_variant(&mut self, name: &str) -> Result<(), String> {
                match name {
                    "Red" => {
                        *self = Color::Red;
                        Ok(())
                    }
                    "Green" => {
                        *self = Color::Green;
                        Ok(())
                    }
                    "Blue" => {
                        *self = Color::Blue;
                        Ok(())
                    }
                    "Rgb" => {
                        *self = Color::Rgb(0.0, 0.0, 0.0);
                        Ok(())
                    }
                    _ => Err(format!("unknown variant: {}", name)),
                }
            }
        }

        let mut color = Color::Rgb(1.0, 0.5, 0.0);

        assert_eq!(color.variants(), &["Red", "Green", "Blue", "Rgb"] as &[&str]);
        assert_eq!(color.variant_name(), "Rgb");
        assert_eq!(color.field_count(), 3);

        let r = color.field_at(0).unwrap().as_any().downcast_ref::<f32>().unwrap();
        assert_eq!(*r, 1.0);

        color.set_variant("Red").unwrap();
        assert_eq!(color.variant_name(), "Red");
        assert_eq!(color.field_count(), 0);

        assert!(color.set_variant("Unknown").is_err());
    }
}
