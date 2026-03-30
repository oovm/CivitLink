//! GWG Engine 反射系统
//!
//! 提供动态类型查询和属性编辑功能。

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// 反射 trait，所有可反射类型都需要实现
pub trait Reflect: Any + Send + Sync {
    /// 获取类型名称
    fn type_name(&self) -> &'static str;

    /// 获取类型 ID
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// 转换为 Any 引用
    fn as_any(&self) -> &dyn Any;

    /// 转换为 Any 可变引用
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 克隆为 Box
    fn clone_value(&self) -> Box<dyn Reflect>;
}

/// 反射注册表，用于管理所有可反射类型
pub struct ReflectionRegistry {
    /// 类型注册表
    types: HashMap<TypeId, TypeInfo>,
}

impl ReflectionRegistry {
    /// 创建新的反射注册表
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// 注册一个可反射类型
    pub fn register<T: Reflect + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.types.insert(
            type_id,
            TypeInfo {
                type_id,
                type_name: std::any::type_name::<T>(),
            },
        );
    }

    /// 检查类型是否已注册
    pub fn is_registered(&self, type_id: TypeId) -> bool {
        self.types.contains_key(&type_id)
    }

    /// 获取类型信息
    pub fn get_type_info(&self, type_id: TypeId) -> Option<&TypeInfo> {
        self.types.get(&type_id)
    }

    /// 获取所有已注册类型
    pub fn registered_types(&self) -> impl Iterator<Item = &TypeInfo> {
        self.types.values()
    }
}

impl Default for ReflectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 类型信息
#[derive(Clone, Debug)]
pub struct TypeInfo {
    /// 类型 ID
    pub type_id: TypeId,
    /// 类型名称
    pub type_name: &'static str,
}

/// 属性编辑器 trait，用于动态编辑可反射类型的属性
pub trait PropertyEditor {
    /// 获取可编辑属性的列表
    fn editable_properties(&self) -> Vec<PropertyInfo>;

    /// 获取属性的值
    fn get_property(&self, name: &str) -> Option<&dyn Reflect>;

    /// 设置属性的值
    fn set_property(&mut self, name: &str, value: Box<dyn Reflect>) -> Result<(), String>;
}

/// 属性信息
#[derive(Clone, Debug)]
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
    /// 创建新的属性信息
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

/// 为基本类型实现 Reflect
macro_rules! impl_reflect_primitive {
    ($($ty:ty),*) => {
        $(
            impl Reflect for $ty {
                fn type_name(&self) -> &'static str {
                    std::any::type_name::<%ty>()
                }

                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }

                fn clone_value(&self) -> Box<dyn Reflect> {
                    Box::new(*self)
                }
            }
        )*
    };
}

impl_reflect_primitive!(i8, i16, i32, i64, i128, isize);
impl_reflect_primitive!(u8, u16, u32, u64, u128, usize);
impl_reflect_primitive!(f32, f64);
impl_reflect_primitive!(bool, char);

impl Reflect for String {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<String>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }
}

pub mod prelude {
    //! 反射系统的预导入模块

    pub use super::{
        PropertyInfo, PropertyEditor, Reflect, ReflectionRegistry, TypeInfo,
    };
}
