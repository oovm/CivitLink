//! Widget 产物定义模块

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Widget 编译产物
#[derive(Debug, Clone)]
pub struct WidgetArtifact {
    /// Widget 元信息
    pub meta: WidgetMeta,
    /// Template 编译产物
    pub template: TemplateBundle,
    /// Script 编译产物（可选）
    pub script: Option<Vec<u8>>,
    /// Style 编译产物（可选）
    pub style: Option<StyleBundle>,
    /// 依赖列表
    pub dependencies: Vec<WidgetDependency>,
    /// 资源引用
    pub resources: Vec<ResourceRef>,
}

/// Widget 元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMeta {
    /// Widget 名称
    pub name: String,
    /// Widget 版本
    pub version: String,
    /// 导出的组件名
    pub export_name: Option<String>,
    /// 编译时间戳
    pub compiled_at: u64,
    /// 编译器版本
    pub compiler_version: String,
}

/// Widget 依赖
#[derive(Debug, Clone)]
pub struct WidgetDependency {
    /// 依赖类型
    pub dep_type: DependencyType,
    /// 依赖路径
    pub path: String,
    /// 依赖版本
    pub version: Option<String>,
}

/// 依赖类型
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    /// 组件依赖
    Component,
    /// 脚本依赖
    Script,
    /// 样式依赖
    Style,
    /// 资源依赖
    Resource,
}

/// Template 编译产物
#[derive(Debug, Clone)]
pub struct TemplateBundle {
    /// 序列化的 UI 节点树
    pub node_tree: Vec<u8>,
    /// 组件引用表
    pub component_refs: HashMap<String, String>,
    /// 数据绑定表
    pub bindings: Vec<SerializedBinding>,
    /// 事件绑定表
    pub event_handlers: Vec<SerializedEventHandler>,
    /// 资源引用
    pub resource_refs: Vec<ResourceRef>,
}

/// 序列化的数据绑定
#[derive(Debug, Clone)]
pub struct SerializedBinding {
    /// 目标元素 ID
    pub target_id: String,
    /// 目标属性名
    pub property: String,
    /// 绑定表达式
    pub expression: String,
}

/// 序列化的事件处理器
#[derive(Debug, Clone)]
pub struct SerializedEventHandler {
    /// 目标元素 ID
    pub target_id: String,
    /// 事件类型
    pub event_type: String,
    /// 处理函数引用
    pub handler: String,
}

/// Style 编译产物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleBundle {
    /// 序列化的样式规则
    pub rules: Vec<u8>,
    /// 选择器索引
    pub selector_index: HashMap<String, Vec<RuleId>>,
    /// 主题变量值
    pub theme_values: HashMap<String, Vec<u8>>,
}

/// 规则 ID
pub type RuleId = u32;

/// 资源引用
#[derive(Debug, Clone)]
pub struct ResourceRef {
    /// 资源路径
    pub path: String,
    /// 资源类型
    pub resource_type: String,
}

/// Widget 产物序列化魔数
pub const WIDGET_MAGIC: &[u8; 4] = b"GGWT";
/// Widget 产物格式版本
pub const WIDGET_VERSION: u16 = 1;

impl WidgetArtifact {
    /// 将 WidgetArtifact 序列化为二进制格式
    pub fn serialize(&self) -> crate::error::WidgetResult<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(WIDGET_MAGIC);
        buffer.extend_from_slice(&WIDGET_VERSION.to_le_bytes());
        let meta_json = serde_json::to_vec(&self.meta)
            .map_err(|e| crate::error::WidgetError::SerializeError(format!("Failed to serialize meta: {}", e)))?;
        buffer.extend_from_slice(&(meta_json.len() as u64).to_le_bytes());
        buffer.extend_from_slice(&meta_json);
        buffer.extend_from_slice(&(self.template.node_tree.len() as u64).to_le_bytes());
        buffer.extend_from_slice(&self.template.node_tree);
        if let Some(ref script) = self.script {
            buffer.extend_from_slice(&(script.len() as u64).to_le_bytes());
            buffer.extend_from_slice(script);
        }
        else {
            buffer.extend_from_slice(&0u64.to_le_bytes());
        }
        if let Some(ref style) = self.style {
            let style_data = serde_json::to_vec(style)
                .map_err(|e| crate::error::WidgetError::SerializeError(format!("Failed to serialize style: {}", e)))?;
            buffer.extend_from_slice(&(style_data.len() as u64).to_le_bytes());
            buffer.extend_from_slice(&style_data);
        }
        else {
            buffer.extend_from_slice(&0u64.to_le_bytes());
        }
        Ok(buffer)
    }
}
