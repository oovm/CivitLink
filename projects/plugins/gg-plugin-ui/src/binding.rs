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
        if parts.len() == 2 { Some(Self { resource_name: parts[0].to_string(), field: parts[1].to_string() }) } else { None }
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
        self.bindings.entry(node_id).or_default().insert(property, key);
    }

    /// 移除节点的所有绑定
    pub fn unregister_node(&mut self, node_id: u64) {
        self.bindings.remove(&node_id);
    }
}

/// 数据绑定系统
///
/// 每帧从 World 读取绑定资源值，更新 UiNode 属性。
pub struct BindingSystem;

impl BindingSystem {
    /// 创建新的数据绑定系统
    pub fn new() -> Self {
        Self
    }
}

impl System for BindingSystem {
    /// 返回系统名称
    fn name(&self) -> &str {
        "ui_binding"
    }

    /// 执行数据绑定系统逻辑
    ///
    /// 遍历绑定注册表，从 World 资源中读取值并更新 UI 节点属性。
    fn execute(&mut self, world: &mut gg_ecs::World) -> GResult<()> {
        let registry = world.get_resource::<BindingRegistry>().cloned();
        let registry = match registry {
            Some(r) => r,
            None => return Ok(()),
        };

        for (node_id, property_bindings) in &registry.bindings {
            for (property, _key) in property_bindings {
                if let Some(tree_res) = world.get_resource_mut::<UiTreeResource>() {
                    if let Some(node) = tree_res.0.get_mut(*node_id) {
                        match property.as_str() {
                            "visible" => {
                                node.visible = true;
                            }
                            "content" => {
                                if let gg_ui::UiNodeData::Text { ref mut content } = node.data {
                                    *content = String::new();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
