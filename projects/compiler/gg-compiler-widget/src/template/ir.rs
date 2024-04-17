//! Template IR 定义模块

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 元素 ID 类型
pub type ElementId = u64;
/// 组件类型 ID
pub type ComponentTypeId = String;

/// Template IR 根节点
#[derive(Debug, Clone)]
pub struct TemplateIr {
    /// 根元素 IR
    pub root: ElementIr,
    /// 组件依赖图
    pub dependencies: HashMap<String, ComponentDependency>,
    /// 数据绑定信息
    pub bindings: Vec<DataBinding>,
    /// 事件绑定信息
    pub events: Vec<EventBinding>,
}

/// 元素 IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementIr {
    /// 元素唯一 ID
    pub id: ElementId,
    /// 组件类型 ID
    pub component_type: ComponentTypeId,
    /// 解析后的属性
    pub properties: HashMap<String, PropertyValue>,
    /// 子元素列表
    pub children: Vec<ElementIr>,
}

/// 组件依赖
#[derive(Debug, Clone)]
pub struct ComponentDependency {
    /// 组件类型 ID
    pub type_id: ComponentTypeId,
    /// 来源模块
    pub source_module: String,
}

/// 数据绑定
#[derive(Debug, Clone)]
pub struct DataBinding {
    /// 目标元素 ID
    pub target: ElementId,
    /// 目标属性名
    pub property: String,
    /// 绑定表达式
    pub expression: String,
    /// 绑定方向
    pub direction: BindingDirection,
}

/// 绑定方向
#[derive(Debug, Clone, PartialEq)]
pub enum BindingDirection {
    /// 单向绑定（数据到视图）
    OneWay,
    /// 双向绑定
    TwoWay,
}

/// 事件绑定
#[derive(Debug, Clone)]
pub struct EventBinding {
    /// 目标元素 ID
    pub target: ElementId,
    /// 事件类型
    pub event_type: String,
    /// 处理函数引用
    pub handler: String,
}

/// 属性值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    /// 字符串值
    String(String),
    /// 整数值
    Int(i64),
    /// 浮点数值
    Float(f64),
    /// 布尔值
    Bool(bool),
    /// 绑定表达式
    Binding(String),
}
