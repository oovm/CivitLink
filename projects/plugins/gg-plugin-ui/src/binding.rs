//! UI 数据绑定模块
//! 提供 UI 节点属性与 ECS 资源的绑定机制

use std::collections::HashMap;

use gg_core::GResult;
use gg_ecs::System;

use crate::UiTreeResource;

/// 绑定键
///
/// 标识一个 ECS 资源字段的路径，格式为 "resource_name.field"。
#[derive(Debug, Clone)]
pub struct BindingKey {
    /// 资源名称
    pub resource_name: String,
    /// 字段名称
    pub field: String,
}

impl BindingKey {
    /// 从路径字符串创建绑定键
    ///
    /// 路径格式为 "resource_name.field"。
    pub fn from_path(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.splitn(2, '.').collect();
        if parts.len() == 2 {
            Some(Self {
                resource_name: parts[0].to_string(),
                field: parts[1].to_string(),
            })
        } else {
            None
        }
    }

    /// 返回完整的资源路径字符串
    ///
    /// 格式为 "resource_name.field"。
    pub fn resource_path(&self) -> String {
        format!("{}.{}", self.resource_name, self.field)
    }
}

/// 绑定值
///
/// 表示绑定解析后得到的数据值，支持多种基础类型。
#[derive(Debug, Clone, PartialEq)]
pub enum BindingValue {
    /// 布尔值
    Bool(bool),
    /// 字符串值
    String(String),
    /// 32位浮点数值
    Float(f32),
    /// 64位整数值
    Int(i64),
}

impl BindingValue {
    /// 尝试将绑定值转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            BindingValue::Bool(b) => Some(*b),
            BindingValue::Int(i) => Some(*i != 0),
            _ => None,
        }
    }

    /// 尝试将绑定值转换为字符串引用
    pub fn as_str(&self) -> Option<&str> {
        match self {
            BindingValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 尝试将绑定值转换为 f32
    pub fn as_float(&self) -> Option<f32> {
        match self {
            BindingValue::Float(f) => Some(*f),
            BindingValue::Int(i) => Some(*i as f32),
            _ => None,
        }
    }
}

/// 绑定解析器 trait
///
/// 定义从 ECS World 中解析绑定值的接口。
/// 实现此 trait 以支持不同数据源的绑定解析。
pub trait BindingResolver: Send + Sync {
    /// 从 World 中解析指定绑定键对应的值
    ///
    /// 如果该解析器无法解析给定的键，返回 None。
    fn resolve(&self, world: &gg_ecs::World, key: &BindingKey) -> Option<BindingValue>;
}

/// 基于 HashMap 的绑定解析器
///
/// 使用内部 HashMap 存储键值对，适用于外部动态注入绑定值的场景。
/// 该解析器不依赖 World 资源，仅根据键的完整路径进行查找。
pub struct HashMapResolver {
    values: HashMap<String, BindingValue>,
}

impl HashMapResolver {
    /// 创建空的 HashMap 解析器
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// 设置指定路径的绑定值
    pub fn set(&mut self, key: &str, value: BindingValue) {
        self.values.insert(key.to_string(), value);
    }
}

impl Default for HashMapResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BindingResolver for HashMapResolver {
    fn resolve(&self, _world: &gg_ecs::World, key: &BindingKey) -> Option<BindingValue> {
        self.values.get(&key.resource_path()).cloned()
    }
}

/// 绑定注册表
///
/// 管理 UI 节点属性到 ECS 资源字段的绑定映射。
#[derive(Debug, Clone, Default)]
pub struct BindingRegistry {
    /// 绑定映射：节点 ID → (属性名 → 绑定键)
    pub bindings: HashMap<u64, HashMap<String, BindingKey>>,
}

impl BindingRegistry {
    /// 创建新的绑定注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 为节点属性注册绑定
    pub fn register(&mut self, node_id: u64, property: String, key: BindingKey) {
        self.bindings
            .entry(node_id)
            .or_default()
            .insert(property, key);
    }

    /// 移除节点的所有绑定
    pub fn unregister_node(&mut self, node_id: u64) {
        self.bindings.remove(&node_id);
    }
}

/// 数据绑定系统
///
/// 每帧从 World 读取绑定资源值，通过注册的解析器链解析绑定键，
/// 并将解析结果更新到对应的 UI 节点属性上。
pub struct BindingSystem {
    resolvers: Vec<Box<dyn BindingResolver>>,
}

impl BindingSystem {
    /// 创建新的数据绑定系统
    pub fn new() -> Self {
        Self {
            resolvers: Vec::new(),
        }
    }

    /// 添加绑定解析器
    ///
    /// 解析器按添加顺序依次尝试，首个返回 Some 的解析器结果将被使用。
    pub fn add_resolver(&mut self, resolver: Box<dyn BindingResolver>) {
        self.resolvers.push(resolver);
    }
}

impl Default for BindingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for BindingSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_binding"
    }

    /// 执行数据绑定系统逻辑
    ///
    /// 遍历绑定注册表，通过解析器链解析每个绑定键的值，
    /// 根据属性名更新 UI 节点的对应字段：
    /// - "content" → 更新 UiNodeData::Text 的 content
    /// - "visible" → 更新 UiNode 的 visible 字段
    /// - "opacity" → 更新 Style 的 opacity 字段
    fn execute(&mut self, world: &mut gg_ecs::World) -> GResult<()> {
        let registry = world.get_resource::<BindingRegistry>().cloned();
        let registry = match registry {
            Some(r) => r,
            None => return Ok(()),
        };

        for (node_id, property_bindings) in &registry.bindings {
            for (property, key) in property_bindings {
                let value = self
                    .resolvers
                    .iter()
                    .find_map(|r| r.resolve(world, key));

                let Some(value) = value else {
                    continue;
                };

                let Some(tree_res) = world.get_resource_mut::<UiTreeResource>() else {
                    continue;
                };

                let Some(node) = tree_res.0.get_mut(*node_id) else {
                    continue;
                };

                match property.as_str() {
                    "visible" => {
                        if let Some(b) = value.as_bool() {
                            node.visible = b;
                        }
                    }
                    "content" => {
                        if let gg_ui::UiNodeData::Text { ref mut content } = node.data {
                            if let Some(s) = value.as_str() {
                                *content = s.to_string();
                            } else {
                                *content = format!("{:?}", value);
                            }
                        }
                    }
                    "opacity" => {
                        if let Some(f) = value.as_float() {
                            node.style.opacity = f.clamp(0.0, 1.0);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
