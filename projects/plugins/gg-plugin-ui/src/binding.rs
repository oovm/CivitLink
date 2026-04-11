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

/// 算术运算符
///
/// 用于绑定表达式中的算术运算。
#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticOp {
    /// 加法
    Add,
    /// 减法
    Subtract,
    /// 乘法
    Multiply,
    /// 除法
    Divide,
}

/// 绑定表达式
///
/// 支持属性路径访问、字面量、算术运算和条件表达式的求值树。
#[derive(Debug, Clone, PartialEq)]
pub enum BindingExpression {
    /// 属性路径访问，支持嵌套点分隔路径如 "player.name"
    Path(BindingKey),
    /// 字面量值
    Literal(BindingValue),
    /// 算术运算表达式
    Arithmetic {
        /// 左操作数
        left: Box<BindingExpression>,
        /// 运算符
        op: ArithmeticOp,
        /// 右操作数
        right: Box<BindingExpression>,
    },
    /// 条件表达式（三元 condition ? true_expr : false_expr）
    Conditional {
        /// 条件表达式
        condition: Box<BindingExpression>,
        /// 条件为真时的表达式
        true_expr: Box<BindingExpression>,
        /// 条件为假时的表达式
        false_expr: Box<BindingExpression>,
    },
}

impl BindingExpression {
    /// 递归求值绑定表达式
    ///
    /// 使用解析器链依次尝试解析路径表达式，返回求值结果。
    /// 算术运算仅支持数值类型（Float 和 Int）；
    /// 条件表达式根据条件值的布尔语义选择分支。
    pub fn evaluate(&self, world: &gg_ecs::World, resolvers: &[Box<dyn BindingResolver>]) -> Option<BindingValue> {
        match self {
            BindingExpression::Path(key) => resolvers.iter().find_map(|r| r.resolve(world, key)),
            BindingExpression::Literal(value) => Some(value.clone()),
            BindingExpression::Arithmetic { left, op, right } => {
                let left_val = left.evaluate(world, resolvers)?;
                let right_val = right.evaluate(world, resolvers)?;
                Self::eval_arithmetic(&left_val, op, &right_val)
            }
            BindingExpression::Conditional {
                condition,
                true_expr,
                false_expr,
            } => {
                let cond_val = condition.evaluate(world, resolvers)?;
                if cond_val.as_bool().unwrap_or(false) {
                    true_expr.evaluate(world, resolvers)
                } else {
                    false_expr.evaluate(world, resolvers)
                }
            }
        }
    }

    /// 执行算术运算
    fn eval_arithmetic(left: &BindingValue, op: &ArithmeticOp, right: &BindingValue) -> Option<BindingValue> {
        match (left, right) {
            (BindingValue::Float(l), BindingValue::Float(r)) => match op {
                ArithmeticOp::Add => Some(BindingValue::Float(l + r)),
                ArithmeticOp::Subtract => Some(BindingValue::Float(l - r)),
                ArithmeticOp::Multiply => Some(BindingValue::Float(l * r)),
                ArithmeticOp::Divide => {
                    if *r == 0.0 {
                        None
                    } else {
                        Some(BindingValue::Float(l / r))
                    }
                }
            },
            (BindingValue::Int(l), BindingValue::Int(r)) => match op {
                ArithmeticOp::Add => Some(BindingValue::Int(l + r)),
                ArithmeticOp::Subtract => Some(BindingValue::Int(l - r)),
                ArithmeticOp::Multiply => Some(BindingValue::Int(l * r)),
                ArithmeticOp::Divide => {
                    if *r == 0 {
                        None
                    } else {
                        Some(BindingValue::Int(l / r))
                    }
                }
            },
            (BindingValue::Float(l), BindingValue::Int(r)) => match op {
                ArithmeticOp::Add => Some(BindingValue::Float(l + *r as f32)),
                ArithmeticOp::Subtract => Some(BindingValue::Float(l - *r as f32)),
                ArithmeticOp::Multiply => Some(BindingValue::Float(l * *r as f32)),
                ArithmeticOp::Divide => {
                    if *r == 0 {
                        None
                    } else {
                        Some(BindingValue::Float(l / *r as f32))
                    }
                }
            },
            (BindingValue::Int(l), BindingValue::Float(r)) => match op {
                ArithmeticOp::Add => Some(BindingValue::Float(*l as f32 + r)),
                ArithmeticOp::Subtract => Some(BindingValue::Float(*l as f32 - r)),
                ArithmeticOp::Multiply => Some(BindingValue::Float(*l as f32 * r)),
                ArithmeticOp::Divide => {
                    if *r == 0.0 {
                        None
                    } else {
                        Some(BindingValue::Float(*l as f32 / r))
                    }
                }
            },
            _ => None,
        }
    }
}

/// 绑定注册表
///
/// 管理 UI 节点属性到 ECS 资源字段或绑定表达式的映射。
#[derive(Debug, Clone, Default)]
pub struct BindingRegistry {
    /// 简单绑定映射：节点 ID → (属性名 → 绑定键)
    pub bindings: HashMap<u64, HashMap<String, BindingKey>>,
    /// 表达式绑定映射：节点 ID → (属性名 → 绑定表达式)
    pub expression_bindings: HashMap<u64, HashMap<String, BindingExpression>>,
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

    /// 为节点属性注册表达式绑定
    pub fn register_expression(&mut self, node_id: u64, property: String, expr: BindingExpression) {
        self.expression_bindings
            .entry(node_id)
            .or_default()
            .insert(property, expr);
    }

    /// 移除节点的所有绑定
    pub fn unregister_node(&mut self, node_id: u64) {
        self.bindings.remove(&node_id);
        self.expression_bindings.remove(&node_id);
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
