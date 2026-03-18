//! GWG Engine 反射系统
//!
//! 提供动态类型查询和属性编辑功能。

use bevy_reflect as reflect;
use std::any::TypeId;

pub use reflect::prelude::*;
pub use reflect::{
    Array, ArrayInfo, DynamicArray, DynamicEnum, DynamicList, DynamicMap, DynamicStruct,
    DynamicTuple, DynamicTupleStruct, Enum, EnumInfo, GetTypeRegistration, List, ListInfo,
    Map, MapInfo, PartialReflect, Reflect, ReflectDeserialize, ReflectFromReflect,
    ReflectSerialize, Struct, StructInfo, Tuple, TupleInfo, TupleStruct, TupleStructInfo,
    TypeInfo, TypePath, TypeRegistration, TypeRegistry, TypeRegistryArc,
};

/// 反射注册表，用于管理所有可反射类型
pub struct ReflectionRegistry {
    inner: TypeRegistry,
}

impl ReflectionRegistry {
    /// 创建一个新的反射注册表
    pub fn new() -> Self {
        Self {
            inner: TypeRegistry::new(),
        }
    }

    /// 注册一个可反射类型
    pub fn register<T: GetTypeRegistration + 'static>(&mut self) {
        self.inner.register::<T>();
    }

    /// 获取类型注册表的引用
    pub fn get(&self) -> &TypeRegistry {
        &self.inner
    }

    /// 获取类型注册表的可变引用
    pub fn get_mut(&mut self) -> &mut TypeRegistry {
        &mut self.inner
    }

    /// 根据 TypeId 获取类型信息
    pub fn get_type_info(&self, type_id: TypeId) -> Option<&'static TypeInfo> {
        self.inner.get_type_info(type_id)
    }

    /// 根据 TypeId 获取类型注册信息
    pub fn get_registration(&self, type_id: TypeId) -> Option<&TypeRegistration> {
        self.inner.get(type_id)
    }

    /// 检查类型是否已注册
    pub fn is_registered(&self, type_id: TypeId) -> bool {
        self.inner.get(type_id).is_some()
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
    fn get_property(&self, name: &str) -> Option<&dyn PartialReflect>;

    /// 设置属性的值
    fn set_property(&mut self, name: &str, value: Box<dyn PartialReflect>) -> anyhow::Result<()>;
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
pub struct StructPropertyEditor<T: Struct> {
    value: T,
}

impl<T: Struct> StructPropertyEditor<T> {
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

impl<T: Struct> PropertyEditor for StructPropertyEditor<T> {
    fn editable_properties(&self) -> Vec<PropertyInfo> {
        let mut properties = Vec::new();
        if let Some(TypeInfo::Struct(struct_info)) = self.value.get_represented_type_info() {
            for field in struct_info.iter() {
                properties.push(PropertyInfo {
                    name: field.name().to_string(),
                    type_name: field.type_path(),
                    writable: true,
                    description: None,
                });
            }
        }
        properties
    }

    fn get_property(&self, name: &str) -> Option<&dyn PartialReflect> {
        self.value.field(name)
    }

    fn set_property(&mut self, name: &str, value: Box<dyn PartialReflect>) -> anyhow::Result<()> {
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
