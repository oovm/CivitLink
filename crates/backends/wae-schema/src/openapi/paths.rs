//! OpenAPI 路径定义

use serde::{Deserialize, Serialize};
use serde_json;

/// 路径项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    /// GET 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    /// POST 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    /// PUT 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    /// DELETE 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    /// PATCH 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    /// HEAD 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    /// OPTIONS 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    /// TRACE 操作
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
    /// 路径描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 路径摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 路径参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterOrReference>>,
    /// 服务器列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<super::Server>>,
}

impl Default for PathItem {
    /// 创建默认的路径项
    fn default() -> Self {
        Self {
            get: None,
            post: None,
            put: None,
            delete: None,
            patch: None,
            head: None,
            options: None,
            trace: None,
            description: None,
            summary: None,
            parameters: None,
            servers: None,
        }
    }
}

impl PathItem {
    /// 创建新的路径项
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 GET 操作
    pub fn get(mut self, op: Operation) -> Self {
        self.get = Some(op);
        self
    }

    /// 设置 POST 操作
    pub fn post(mut self, op: Operation) -> Self {
        self.post = Some(op);
        self
    }

    /// 设置 PUT 操作
    pub fn put(mut self, op: Operation) -> Self {
        self.put = Some(op);
        self
    }

    /// 设置 DELETE 操作
    pub fn delete(mut self, op: Operation) -> Self {
        self.delete = Some(op);
        self
    }

    /// 设置 PATCH 操作
    pub fn patch(mut self, op: Operation) -> Self {
        self.patch = Some(op);
        self
    }

    /// 设置 HEAD 操作
    pub fn head(mut self, op: Operation) -> Self {
        self.head = Some(op);
        self
    }

    /// 设置 OPTIONS 操作
    pub fn options(mut self, op: Operation) -> Self {
        self.options = Some(op);
        self
    }

    /// 设置 TRACE 操作
    pub fn trace(mut self, op: Operation) -> Self {
        self.trace = Some(op);
        self
    }
}

/// 操作定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// 操作标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// 操作摘要
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// 操作描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 操作 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    /// 参数列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterOrReference>>,
    /// 请求体
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<super::RequestBodyOrReference>,
    /// 响应定义
    pub responses: std::collections::BTreeMap<String, super::ResponseOrReference>,
    /// 回调定义
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<std::collections::BTreeMap<String, super::Callback>>,
    /// 弃用标记
    #[serde(default)]
    pub deprecated: bool,
    /// 安全要求
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<super::SecurityRequirement>>,
    /// 服务器列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<super::Server>>,
    /// 外部文档
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<super::ExternalDocumentation>,
}

impl Operation {
    /// 创建新操作
    pub fn new() -> Self {
        Self {
            tags: None,
            summary: None,
            description: None,
            operation_id: None,
            parameters: None,
            request_body: None,
            responses: std::collections::BTreeMap::new(),
            callbacks: None,
            deprecated: false,
            security: None,
            servers: None,
            external_docs: None,
        }
    }

    /// 设置摘要
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 设置操作 ID
    pub fn operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }

    /// 添加标签
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        let tags = self.tags.get_or_insert_with(Vec::new);
        tags.push(tag.into());
        self
    }

    /// 添加参数
    pub fn parameter(mut self, param: ParameterOrReference) -> Self {
        let params = self.parameters.get_or_insert_with(Vec::new);
        params.push(param);
        self
    }

    /// 设置请求体
    pub fn request_body(mut self, body: super::RequestBodyOrReference) -> Self {
        self.request_body = Some(body);
        self
    }

    /// 添加响应
    pub fn response(mut self, status: impl Into<String>, resp: super::ResponseOrReference) -> Self {
        self.responses.insert(status.into(), resp);
        self
    }

    /// 设置弃用
    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = deprecated;
        self
    }
}

impl Default for Operation {
    /// 创建默认的操作
    fn default() -> Self {
        Self::new()
    }
}

/// 参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// 参数名称
    pub name: String,
    /// 参数位置
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    /// 参数描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 参数 Schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<crate::Schema>,
    /// 是否废弃
    #[serde(default)]
    pub deprecated: bool,
    /// 是否允许空值
    #[serde(default)]
    pub allow_empty_value: bool,
    /// 示例值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    /// 多个示例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<std::collections::BTreeMap<String, super::ExampleOrReference>>,
}

/// 参数位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    /// 路径参数
    Path,
    /// 查询参数
    Query,
    /// 请求头参数
    Header,
    /// Cookie 参数
    Cookie,
}

impl Parameter {
    /// 创建路径参数
    pub fn path(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: ParameterLocation::Path,
            description: None,
            required: true,
            schema: None,
            deprecated: false,
            allow_empty_value: false,
            example: None,
            examples: None,
        }
    }

    /// 创建查询参数
    pub fn query(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: ParameterLocation::Query,
            description: None,
            required: false,
            schema: None,
            deprecated: false,
            allow_empty_value: false,
            example: None,
            examples: None,
        }
    }

    /// 创建请求头参数
    pub fn header(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: ParameterLocation::Header,
            description: None,
            required: false,
            schema: None,
            deprecated: false,
            allow_empty_value: false,
            example: None,
            examples: None,
        }
    }

    /// 创建 Cookie 参数
    pub fn cookie(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: ParameterLocation::Cookie,
            description: None,
            required: false,
            schema: None,
            deprecated: false,
            allow_empty_value: false,
            example: None,
            examples: None,
        }
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// 设置必需
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// 设置 Schema
    pub fn schema(mut self, schema: crate::Schema) -> Self {
        self.schema = Some(schema);
        self
    }

    /// 设置废弃
    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = deprecated;
        self
    }

    /// 设置允许空值
    pub fn allow_empty_value(mut self, allow: bool) -> Self {
        self.allow_empty_value = allow;
        self
    }

    /// 设置示例值
    pub fn example(mut self, example: serde_json::Value) -> Self {
        self.example = Some(example);
        self
    }
}

/// 参数或引用
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterOrReference {
    /// 参数对象
    Parameter(Parameter),
    /// 引用对象
    Reference(super::Reference),
}
