//! GWG Engine 反射系统
//!
//! 提供动态类型查询和属性编辑功能。

use bevy_reflect as reflect;

pub use reflect::prelude::*;
pub use reflect::{
    Array, ArrayInfo, DynamicArray, DynamicEnum, DynamicList, DynamicMap, DynamicStruct,
    DynamicTuple, DynamicTupleStruct, Enum, EnumInfo, List, ListInfo, Map, MapInfo,
    Reflect, ReflectDeserialize, ReflectFromReflect, ReflectSerialize, Struct, StructInfo,
    Tuple, TupleInfo, TupleStruct, TupleStructInfo, TypeInfo, TypePath, TypeRegistry,
    TypeRegistryArc,
};

/// 反射注册表，用于管理所有可反射类型
pub struct ReflectionRegistry {
    inner: TypeRegistryArc,
}

impl ReflectionRegistry {
    /// 创建一个新的反射注册表
    pub fn new() -> Self {
        Self {
            inner: TypeRegistry::default().into(),
        }
    }

    /// 注册一个可反射类型
    pub fn register<T: Reflect + TypePath + FromReflect + 'static>(&self) {
        let mut registry = self.inner.write();
        registry.register::<T>();
        registry.register_type_data::<T, ReflectFromReflect>();
    }

    /// 获取类型注册表的只读引用
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, TypeRegistry> {
        self.inner.read()
    }

    /// 获取类型注册表的可变引用
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, TypeRegistry> {
        self.inner.write()
    }

    /// 根据类型名称获取类型信息
    pub fn get_type_info(&self, type_name: &str) -> Option<&'static TypeInfo> {
        let registry = self.inner.read();
        registry.get_type_data::<ReflectFromReflect>(type_name)?;
        registry.get_type_info(type_name)
    }

    /// 检查类型是否已注册
    pub fn is_registered(&self, type_name: &str) -> bool {
        let registry = self.inner.read();
        registry.get(type_name).is_some()
    }
}

impl Default for ReflectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 属性编辑器 trait，用于动态编辑可反射类型的属性
pub trait PropertyEditor {
    /// 获取可编辑属性的列表
    fn editable_properties(&self) -> Vec<PropertyInfo>;

    /// 获取属性的值
    fn get_property(&self, name: &str) -> Option<&dyn Reflect>;

    /// 设置属性的值
    fn set_property(&mut self, name: &str, value: Box<dyn Reflect>) -> anyhow::Result<()>;
}

/// 属性信息
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    /// 属性名称
    pub name: String,
    /// 属性类型名称
    pub type_name: &'static str,
    /// 属性是否可写
    pub writable: bool,
    /// 属性描述
    pub description: Option<String>,
}

impl PropertyInfo {
    /// 创建一个新的属性信息
    pub fn new(name: String, type_name: &'static str, writable: bool) -> Self {
        Self {
            name,
            type_name,
            writable,
            description: None,
        }
    }

    /// 设置属性描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// 为结构体实现属性编辑器
pub struct StructPropertyEditor<T: Reflect + Struct> {
    value: T,
}

impl<T: Reflect + Struct> StructPropertyEditor<T> {
    /// 创建一个新的结构体属性编辑器
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        &self.value
    }

    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// 消费编辑器并返回内部值
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T: Reflect + Struct> PropertyEditor for StructPropertyEditor<T> {
    fn editable_properties(&self) -> Vec<PropertyInfo> {
        let mut properties = Vec::new();
        if let Some(type_info) = self.value.type_info() {
            if let TypeInfo::Struct(struct_info) = type_info {
                for field in struct_info.iter() {
                    properties.push(PropertyInfo {
                        name: field.name().to_string(),
                        type_name: field.type_path(),
                        writable: true,
                        description: None,
                    });
                }
            }
        }
        properties
    }

    fn get_property(&self, name: &str) -> Option<&dyn Reflect> {
        self.value.field(name)
    }

    fn set_property(&mut self, name: &str, value: Box<dyn Reflect>) -> anyhow::Result<()> {
        if let Some(field) = self.value.field_mut(name) {
            field.apply(&*value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Property not found: {}", name))
        }
    }
}

pub mod prelude {
    //! 反射系统的预导入模块

    pub use super::reflect::prelude::*;
    pub use super::{
        PropertyEditor, PropertyInfo, ReflectionRegistry, StructPropertyEditor,
    };
}
