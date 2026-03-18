//! JWT Claims 定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 标准 JWT Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// 签发者 (iss)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// 受众 (aud)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    /// 主题 (sub) - 通常是用户 ID
    pub sub: String,

    /// JWT ID (jti) - 唯一标识符
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,

    /// 签发时间 (iat)
    pub iat: i64,

    /// 过期时间 (exp)
    pub exp: i64,

    /// 生效时间 (nbf)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// 自定义声明
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl JwtClaims {
    /// 创建新的 Claims
    ///
    /// # Arguments
    /// * `subject` - 主题，通常是用户 ID
    /// * `expires_in_seconds` - 过期时间（秒）
    pub fn new(subject: impl Into<String>, expires_in_seconds: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            iss: None,
            aud: None,
            sub: subject.into(),
            jti: Some(uuid::Uuid::new_v4().to_string()),
            iat: now,
            exp: now + expires_in_seconds,
            nbf: None,
            custom: HashMap::new(),
        }
    }

    /// 设置签发者
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.iss = Some(issuer.into());
        self
    }

    /// 设置受众
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.aud = Some(audience.into());
        self
    }

    /// 设置生效时间
    pub fn with_not_before(mut self, nbf: i64) -> Self {
        self.nbf = Some(nbf);
        self
    }

    /// 添加自定义声明
    pub fn with_custom_claim(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// 检查令牌是否已过期
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// 检查令牌是否已生效
    pub fn is_valid_now(&self) -> bool {
        let now = Utc::now().timestamp();
        if self.is_expired() {
            return false;
        }
        if let Some(nbf) = self.nbf {
            if now < nbf {
                return false;
            }
        }
        true
    }

    /// 获取过期时间作为 DateTime
    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_default()
    }

    /// 获取签发时间作为 DateTime
    pub fn issued_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.iat, 0).unwrap_or_default()
    }
}

/// 访问令牌 Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    /// 用户 ID
    pub user_id: String,

    /// 用户名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// 角色
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,

    /// 权限
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,

    /// 会话 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl AccessTokenClaims {
    /// 创建新的访问令牌 Claims
    pub fn new(user_id: impl Into<String>) -> Self {
        Self { user_id: user_id.into(), username: None, roles: Vec::new(), permissions: Vec::new(), session_id: None }
    }

    /// 设置用户名
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// 设置角色
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// 设置权限
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// 设置会话 ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

/// 刷新令牌 Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// 用户 ID
    pub user_id: String,

    /// 会话 ID
    pub session_id: String,

    /// 令牌版本（用于撤销）
    pub version: u32,
}

impl RefreshTokenClaims {
    /// 创建新的刷新令牌 Claims
    pub fn new(user_id: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self { user_id: user_id.into(), session_id: session_id.into(), version: 0 }
    }

    /// 设置版本
    pub fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
}
