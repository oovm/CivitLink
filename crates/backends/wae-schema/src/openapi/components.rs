//! OpenAPI 组件定义

use serde::{Deserialize, Serialize};
use serde_json;
use super::{Parameter, SecurityScheme};

/// 组件定义
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Components {
    /// Schema 定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub schemas: std::collections::BTreeMap<String, crate::Schema>,
    /// 响应定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub responses: std::collections::BTreeMap<String, Response>,
    /// 参数定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub parameters: std::collections::BTreeMap<String, Parameter>,
    /// 示例定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub examples: std::collections::BTreeMap<String, Example>,
    /// 请求体定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub request_bodies: std::collections::BTreeMap<String, RequestBody>,
    /// 头部定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub headers: std::collections::BTreeMap<String, Header>,
    /// 安全方案定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub security_schemes: std::collections::BTreeMap<String, SecurityScheme>,
    /// 链接定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub links: std::collections::BTreeMap<String, Link>,
    /// 回调定义
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub callbacks: std::collections::BTreeMap<String, Callback>,
}

/// 响应定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// 响应描述
    pub description: String,
    /// 内容定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<std::collections::BTreeMap<String, MediaType>>,
    /// 响应头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::BTreeMap<String, HeaderOrReference>>,
    /// 链接
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<std::collections::BTreeMap<String, LinkOrReference>>,
}

impl Response {
    /// 创建新响应
    pub fn new(description: impl Into<String>) -> Self {
        Self { description: description.into(), content: None, headers: None, links: None }
    }

    /// 添加 JSON 响应
    pub fn json(mut self, schema: crate::Schema) -> Self {
        let content = self.content.get_or_insert_with(std::collections::BTreeMap::new);
        content.insert(
            "application/json".to_string(),
            MediaType { schema: Some(schema), example: None, examples: None, encoding: None },
        );
        self
    }
}

/// 请求体定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// 请求体描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 内容定义
    pub content: std::collections::BTreeMap<String, MediaType>,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
}

impl RequestBody {
    /// 创建 JSON 请求体
    pub fn json(schema: crate::Schema) -> Self {
        let mut content = std::collections::BTreeMap::new();
        content.insert(
            "application/json".to_string(),
            MediaType { schema: Some(schema), example: None, examples: None, encoding: None },
        );
        Self { description: None, content, required: true }
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 设置是否必需
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

/// 媒体类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    /// Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::Schema>,
    /// 示例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    /// 多个示例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<std::collections::BTreeMap<String, ExampleOrReference>>,
    /// 编码定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<std::collections::BTreeMap<String, Encoding>>,
}

/// 编码定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encoding {
    /// 内容类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::BTreeMap<String, HeaderOrReference>>,
    /// 样式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// 是否展开
    #[serde(default)]
    pub explode: bool,
    /// 是否允许保留字符
    #[serde(default)]
    pub allow_reserved: bool,
}

/// 头部定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    /// 头部描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::Schema>,
    /// 是否废弃
    #[serde(default)]
    pub deprecated: bool,
    /// 示例值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    /// 多个示例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<std::collections::BTreeMap<String, ExampleOrReference>>,
}

/// 示例定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// 示例摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 示例描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 示例值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    /// 外部示例值 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_value: Option<String>,
}

/// 链接定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// 链接操作引用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_ref: Option<String>,
    /// 链接操作 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// 链接参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::BTreeMap<String, serde_json::Value>>,
    /// 链接请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<serde_json::Value>,
    /// 链接描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 链接服务器
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<super::Server>,
}

/// 回调定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Callback {
    /// 回调路径项
    #[serde(flatten)]
    pub paths: std::collections::BTreeMap<String, super::PathItem>,
}

/// 引用对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// 引用地址
    #[serde(rename = "$ref")]
    pub ref_path: String,
}

/// 头部或引用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HeaderOrReference {
    /// 头部对象
    Header(Header),
    /// 引用对象
    Reference(Reference),
}

/// 示例或引用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExampleOrReference {
    /// 示例对象
    Example(Example),
    /// 引用对象
    Reference(Reference),
}

/// 链接或引用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LinkOrReference {
    /// 链接对象
    Link(Link),
    /// 引用对象
    Reference(Reference),
}

/// 响应或引用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseOrReference {
    /// 响应对象
    Response(Response),
    /// 引用对象
    Reference(Reference),
}

/// 请求体或引用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestBodyOrReference {
    /// 请求体对象
    RequestBody(RequestBody),
    /// 引用对象
    Reference(Reference),
}
