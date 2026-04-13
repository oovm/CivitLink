use std::collections::{HashMap, HashSet};

/// 属性类型
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    /// 字符串
    String,
    /// 整数
    Int,
    /// 浮点数
    Float,
    /// 布尔值
    Bool,
    /// 颜色
    Color,
    /// 长度
    Length,
    /// 枚举值
    Enum(Vec<String>),
    /// 二维向量
    Vec2,
    /// 三维向量
    Vec3,
    /// 四维向量
    Vec4,
    /// 自定义类型
    Custom(String),
}

/// 属性 Schema
#[derive(Debug, Clone)]
pub struct PropertySchema {
    /// 属性名
    pub name: String,
    /// 属性类型
    pub property_type: PropertyType,
    /// 默认值
    pub default_value: Option<String>,
    /// 是否必需
    pub required: bool,
    /// 是否支持绑定
    pub bindable: bool,
}

/// 事件 Schema
#[derive(Debug, Clone)]
pub struct EventSchema {
    /// 事件名
    pub name: String,
    /// 事件参数类型
    pub parameters: Vec<PropertyType>,
}

/// 组件 Schema
#[derive(Debug, Clone)]
pub struct ComponentSchema {
    /// 组件类型名
    pub type_name: String,
    /// 组件属性列表
    pub properties: HashMap<String, PropertySchema>,
    /// 组件事件列表
    pub events: HashMap<String, EventSchema>,
    /// 是否为容器组件
    pub is_container: bool,
    /// 允许的子组件类型
    pub allowed_children: Vec<String>,
    /// 组件来源模块
    pub source_module: String,
}

/// 组件注册表
pub struct ComponentRegistry {
    /// 已注册的组件
    pub(crate) components: HashMap<String, ComponentSchema>,
    /// 内置组件名称集合
    pub(crate) builtins: HashSet<String>,
}
