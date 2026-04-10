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

/// 反射注册表，管理所有可反射类型
pub struct ReflectionRegistry {
    /// 类型注册映射
    registrations: HashMap<TypeId, TypeRegistration>,
}

impl ReflectionRegistry {
    /// 创建空的反射注册表
    pub fn new() -> Self {
        Self { registrations: HashMap::new() }
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
        PartialReflect, PropertyEditor, PropertyInfo, ReflectionRegistry, StructPropertyEditor, TypeInfo, TypeRegistration,
    };
    pub use gg_macros::Reflect;
}

