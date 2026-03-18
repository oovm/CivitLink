//! OpenAPI 相关功能

use serde::{Deserialize, Serialize};

mod components;
mod paths;
mod security;

pub use components::*;
pub use paths::*;
pub use security::*;

/// OpenAPI 文档信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInfo {
    /// API 标题
    pub title: String,
    /// API 版本
    pub version: String,
    /// API 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 服务条款 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    /// 联系信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    /// 许可信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

impl Default for OpenApiInfo {
    /// 创建默认的 OpenAPI 信息
    fn default() -> Self {
        Self {
            title: "API".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
        }
    }
}

/// 联系信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// 联系人姓名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 联系人邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// 联系人 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// 许可信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// 许可证名称
    pub name: String,
    /// 许可证 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// 外部文档定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDocumentation {
    /// 外部文档 URL
    pub url: String,
    /// 外部文档描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 标签定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// 标签名称
    pub name: String,
    /// 标签描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 标签的外部文档
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocumentation>,
}

/// 服务器定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// 服务器 URL
    pub url: String,
    /// 服务器描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 服务器变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<std::collections::BTreeMap<String, ServerVariable>>,
}

/// 服务器变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVariable {
    /// 默认值
    pub default: String,
    /// 枚举值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// OpenAPI 文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiDoc {
    /// OpenAPI 版本
    pub openapi: String,
    /// API 信息
    pub info: OpenApiInfo,
    /// 路径定义
    pub paths: std::collections::BTreeMap<String, PathItem>,
    /// 组件定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,
    /// 服务器列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
    /// 标签列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    /// 外部文档
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocumentation>,
    /// 安全要求
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,
}

impl Default for OpenApiDoc {
    /// 创建默认的 OpenAPI 文档
    fn default() -> Self {
        Self {
            openapi: "3.1.0".to_string(),
            info: OpenApiInfo::default(),
            paths: std::collections::BTreeMap::new(),
            components: None,
            servers: None,
            tags: None,
            external_docs: None,
            security: None,
        }
    }
}

impl OpenApiDoc {
    /// 创建新的 OpenAPI 文档
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self { info: OpenApiInfo { title: title.into(), version: version.into(), ..Default::default() }, ..Default::default() }
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.info.description = Some(desc.into());
        self
    }

    /// 添加路径
    pub fn path(mut self, path: impl Into<String>, item: PathItem) -> Self {
        self.paths.insert(path.into(), item);
        self
    }

    /// 添加 Schema 组件
    pub fn schema(mut self, name: impl Into<String>, schema: crate::Schema) -> Self {
        let components = self.components.get_or_insert_with(Components::default);
        components.schemas.insert(name.into(), schema);
        self
    }

    /// 添加安全方案组件
    pub fn security_scheme(mut self, name: impl Into<String>, scheme: SecurityScheme) -> Self {
        let components = self.components.get_or_insert_with(Components::default);
        components.security_schemes.insert(name.into(), scheme);
        self
    }

    /// 添加响应组件
    pub fn response(mut self, name: impl Into<String>, response: Response) -> Self {
        let components = self.components.get_or_insert_with(Components::default);
        components.responses.insert(name.into(), response);
        self
    }

    /// 添加参数组件
    pub fn parameter(mut self, name: impl Into<String>, parameter: Parameter) -> Self {
        let components = self.components.get_or_insert_with(Components::default);
        components.parameters.insert(name.into(), parameter);
        self
    }

    /// 添加请求体组件
    pub fn request_body(mut self, name: impl Into<String>, body: RequestBody) -> Self {
        let components = self.components.get_or_insert_with(Components::default);
        components.request_bodies.insert(name.into(), body);
        self
    }

    /// 添加服务器
    pub fn server(mut self, url: impl Into<String>, description: Option<String>) -> Self {
        let servers = self.servers.get_or_insert_with(Vec::new);
        servers.push(Server { url: url.into(), description, variables: None });
        self
    }

    /// 添加标签
    pub fn tag(mut self, name: impl Into<String>, description: Option<String>) -> Self {
        let tags = self.tags.get_or_insert_with(Vec::new);
        tags.push(Tag { name: name.into(), description, external_docs: None });
        self
    }

    /// 设置外部文档
    pub fn external_docs(mut self, url: impl Into<String>, description: Option<String>) -> Self {
        self.external_docs = Some(ExternalDocumentation { url: url.into(), description });
        self
    }

    /// 设置安全要求
    pub fn security(mut self, security: SecurityRequirement) -> Self {
        let securities = self.security.get_or_insert_with(Vec::new);
        securities.push(security);
        self
    }

    /// 转换为 JSON
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// 转换为 YAML
    #[cfg(feature = "yaml")]
    pub fn to_yaml(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }
}
