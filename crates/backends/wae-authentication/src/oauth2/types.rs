//! OAuth2 类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OAuth2 访问令牌响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    /// 访问令牌
    pub access_token: String,

    /// 令牌类型
    pub token_type: String,

    /// 过期时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,

    /// 刷新令牌
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,

    /// 授权范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// ID 令牌 (OpenID Connect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

impl TokenResponse {
    /// 创建新的令牌响应
    pub fn new(access_token: impl Into<String>, token_type: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            token_type: token_type.into(),
            expires_in: None,
            refresh_token: None,
            scope: None,
            id_token: None,
        }
    }

    /// 设置过期时间
    pub fn with_expires_in(mut self, expires_in: u64) -> Self {
        self.expires_in = Some(expires_in);
        self
    }

    /// 设置刷新令牌
    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }

    /// 设置授权范围
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// 设置 ID 令牌
    pub fn with_id_token(mut self, id_token: impl Into<String>) -> Self {
        self.id_token = Some(id_token.into());
        self
    }
}

/// OAuth2 授权码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    /// 授权码
    pub code: String,

    /// 状态参数
    pub state: Option<String>,
}

/// OAuth2 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// 用户唯一标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    /// 用户名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// 昵称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,

    /// 姓名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// 姓
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,

    /// 头像 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,

    /// 邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// 邮箱是否已验证
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,

    /// 手机号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,

    /// 手机号是否已验证
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number_verified: Option<bool>,

    /// 地区/语言
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// 时区
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoneinfo: Option<String>,

    /// 其他字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl UserInfo {
    /// 创建新的用户信息
    pub fn new() -> Self {
        Self {
            sub: None,
            username: None,
            nickname: None,
            name: None,
            given_name: None,
            family_name: None,
            picture: None,
            email: None,
            email_verified: None,
            phone_number: None,
            phone_number_verified: None,
            locale: None,
            zoneinfo: None,
            extra: HashMap::new(),
        }
    }

    /// 获取用户 ID
    pub fn user_id(&self) -> Option<&str> {
        self.sub.as_deref()
    }

    /// 获取显示名称
    pub fn display_name(&self) -> Option<&str> {
        self.nickname.as_deref().or(self.name.as_deref()).or(self.username.as_deref())
    }
}

impl Default for UserInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// OAuth2 授权 URL 构建结果
#[derive(Debug, Clone)]
pub struct AuthorizationUrl {
    /// 授权 URL
    pub url: String,

    /// 状态参数
    pub state: String,

    /// PKCE code verifier
    pub code_verifier: Option<String>,
}

/// OAuth2 令牌刷新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    /// 刷新令牌
    pub refresh_token: String,

    /// 授权范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl RefreshTokenRequest {
    /// 创建新的刷新请求
    pub fn new(refresh_token: impl Into<String>) -> Self {
        Self { refresh_token: refresh_token.into(), scope: None }
    }

    /// 设置授权范围
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }
}

/// OAuth2 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 错误代码
    pub error: String,

    /// 错误描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,

    /// 错误 URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

impl ErrorResponse {
    /// 创建新的错误响应
    pub fn new(error: impl Into<String>) -> Self {
        Self { error: error.into(), error_description: None, error_uri: None }
    }

    /// 设置错误描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.error_description = Some(description.into());
        self
    }

    /// 设置错误 URI
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.error_uri = Some(uri.into());
        self
    }
}
