//! OpenAPI 安全定义

use serde::{Deserialize, Serialize};

/// 安全要求
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityRequirement {
    /// 安全方案名称和范围
    #[serde(flatten)]
    pub schemes: std::collections::BTreeMap<String, Vec<String>>,
}

impl SecurityRequirement {
    /// 创建新的安全要求
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加安全方案
    pub fn scheme(mut self, name: impl Into<String>, scopes: Vec<&str>) -> Self {
        self.schemes.insert(name.into(), scopes.into_iter().map(String::from).collect());
        self
    }
}

/// 安全方案类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SecuritySchemeType {
    /// API Key
    ApiKey,
    /// HTTP 认证
    Http,
    /// OAuth2
    OAuth2,
    /// OpenID Connect
    OpenIdConnect,
}

/// API Key 位置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation {
    /// 查询参数
    Query,
    /// 请求头
    Header,
    /// Cookie
    Cookie,
}

/// 安全方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    /// 安全方案类型
    #[serde(rename = "type")]
    pub scheme_type: SecuritySchemeType,
    /// 安全方案描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// API Key 名称（仅 apiKey 类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// API Key 位置（仅 apiKey 类型）
    #[serde(rename = "in")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ApiKeyLocation>,
    /// HTTP 方案名称（仅 http 类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,
    /// HTTP Bearer 格式（仅 http 类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_format: Option<String>,
    /// OAuth2 流程（仅 oauth2 类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flows: Option<OAuthFlows>,
    /// OpenID Connect URL（仅 openIdConnect 类型）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_id_connect_url: Option<String>,
}

impl SecurityScheme {
    /// 创建 API Key 安全方案
    pub fn api_key(name: impl Into<String>, location: ApiKeyLocation) -> Self {
        Self {
            scheme_type: SecuritySchemeType::ApiKey,
            description: None,
            name: Some(name.into()),
            location: Some(location),
            scheme: None,
            bearer_format: None,
            flows: None,
            open_id_connect_url: None,
        }
    }

    /// 创建 HTTP Basic 安全方案
    pub fn basic() -> Self {
        Self {
            scheme_type: SecuritySchemeType::Http,
            description: None,
            name: None,
            location: None,
            scheme: Some("basic".to_string()),
            bearer_format: None,
            flows: None,
            open_id_connect_url: None,
        }
    }

    /// 创建 HTTP Bearer 安全方案
    pub fn bearer(format: Option<String>) -> Self {
        Self {
            scheme_type: SecuritySchemeType::Http,
            description: None,
            name: None,
            location: None,
            scheme: Some("bearer".to_string()),
            bearer_format: format,
            flows: None,
            open_id_connect_url: None,
        }
    }

    /// 创建 OAuth2 安全方案
    pub fn oauth2(flows: OAuthFlows) -> Self {
        Self {
            scheme_type: SecuritySchemeType::OAuth2,
            description: None,
            name: None,
            location: None,
            scheme: None,
            bearer_format: None,
            flows: Some(flows),
            open_id_connect_url: None,
        }
    }

    /// 创建 OpenID Connect 安全方案
    pub fn open_id_connect(url: impl Into<String>) -> Self {
        Self {
            scheme_type: SecuritySchemeType::OpenIdConnect,
            description: None,
            name: None,
            location: None,
            scheme: None,
            bearer_format: None,
            flows: None,
            open_id_connect_url: Some(url.into()),
        }
    }

    /// 设置描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// OAuth2 流程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlows {
    /// Implicit 流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuthFlow>,
    /// Password 流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuthFlow>,
    /// Client Credentials 流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuthFlow>,
    /// Authorization Code 流程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuthFlow>,
}

/// OAuth2 流程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    /// 授权 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
    /// Token URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
    /// 刷新 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    /// 可用的范围
    pub scopes: std::collections::BTreeMap<String, String>,
}

impl OAuthFlow {
    /// 创建新的 OAuth 流程
    pub fn new() -> Self {
        Self { authorization_url: None, token_url: None, refresh_url: None, scopes: std::collections::BTreeMap::new() }
    }

    /// 设置授权 URL
    pub fn authorization_url(mut self, url: impl Into<String>) -> Self {
        self.authorization_url = Some(url.into());
        self
    }

    /// 设置 Token URL
    pub fn token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = Some(url.into());
        self
    }

    /// 设置刷新 URL
    pub fn refresh_url(mut self, url: impl Into<String>) -> Self {
        self.refresh_url = Some(url.into());
        self
    }

    /// 添加范围
    pub fn scope(mut self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.scopes.insert(name.into(), description.into());
        self
    }
}

impl Default for OAuthFlow {
    fn default() -> Self {
        Self::new()
    }
}
