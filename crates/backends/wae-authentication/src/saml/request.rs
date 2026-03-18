//! SAML 请求定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::config::{NameIdFormat, SamlBinding};

/// SAML 认证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAuthnRequest {
    /// 请求 ID
    #[serde(rename = "@ID")]
    pub id: String,

    /// 版本
    #[serde(rename = "@Version")]
    pub version: String,

    /// 发行时间
    #[serde(rename = "@IssueInstant")]
    pub issue_instant: DateTime<Utc>,

    /// 目标
    #[serde(rename = "@Destination")]
    pub destination: Option<String>,

    /// 协议绑定
    #[serde(rename = "@ProtocolBinding")]
    pub protocol_binding: Option<String>,

    /// 断言消费者服务 URL
    #[serde(rename = "@AssertionConsumerServiceURL")]
    pub assertion_consumer_service_url: Option<String>,

    /// 发行人
    #[serde(rename = "saml:Issuer")]
    pub issuer: String,

    /// 名称 ID 策略
    #[serde(rename = "samlp:NameIDPolicy")]
    pub name_id_policy: Option<SamlNameIdPolicy>,

    /// 请求认证上下文
    #[serde(rename = "samlp:RequestedAuthnContext")]
    pub requested_authn_context: Option<SamlRequestedAuthnContext>,

    /// 签名
    #[serde(rename = "ds:Signature")]
    pub signature: Option<String>,
}

impl SamlAuthnRequest {
    /// 创建新的认证请求
    pub fn new(id: impl Into<String>, issuer: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: "2.0".to_string(),
            issue_instant: Utc::now(),
            destination: None,
            protocol_binding: None,
            assertion_consumer_service_url: None,
            issuer: issuer.into(),
            name_id_policy: None,
            requested_authn_context: None,
            signature: None,
        }
    }

    /// 设置目标
    pub fn with_destination(mut self, destination: impl Into<String>) -> Self {
        self.destination = Some(destination.into());
        self
    }

    /// 设置协议绑定
    pub fn with_protocol_binding(mut self, binding: SamlBinding) -> Self {
        self.protocol_binding = Some(binding.uri().to_string());
        self
    }

    /// 设置 ACS URL
    pub fn with_acs_url(mut self, url: impl Into<String>) -> Self {
        self.assertion_consumer_service_url = Some(url.into());
        self
    }

    /// 设置名称 ID 策略
    pub fn with_name_id_policy(mut self, policy: SamlNameIdPolicy) -> Self {
        self.name_id_policy = Some(policy);
        self
    }

    /// 设置请求认证上下文
    pub fn with_authn_context(mut self, context: SamlRequestedAuthnContext) -> Self {
        self.requested_authn_context = Some(context);
        self
    }
}

/// SAML 名称 ID 策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlNameIdPolicy {
    /// 格式
    #[serde(rename = "@Format")]
    pub format: Option<String>,

    /// 允许创建
    #[serde(rename = "@AllowCreate")]
    pub allow_create: Option<bool>,

    /// SP 名称限定符
    #[serde(rename = "@SPNameQualifier")]
    pub sp_name_qualifier: Option<String>,
}

impl SamlNameIdPolicy {
    /// 创建新的名称 ID 策略
    pub fn new() -> Self {
        Self { format: None, allow_create: None, sp_name_qualifier: None }
    }

    /// 设置格式
    pub fn with_format(mut self, format: NameIdFormat) -> Self {
        self.format = Some(format.uri().to_string());
        self
    }

    /// 设置允许创建
    pub fn with_allow_create(mut self, allow: bool) -> Self {
        self.allow_create = Some(allow);
        self
    }
}

impl Default for SamlNameIdPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// SAML 请求认证上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlRequestedAuthnContext {
    /// 比较方式
    #[serde(rename = "@Comparison")]
    pub comparison: Option<String>,

    /// 认证上下文类引用
    #[serde(rename = "saml:AuthnContextClassRef")]
    pub class_ref: Vec<String>,
}

impl SamlRequestedAuthnContext {
    /// 创建新的请求认证上下文
    pub fn new(class_refs: Vec<String>) -> Self {
        Self { comparison: None, class_ref: class_refs }
    }

    /// 设置比较方式
    pub fn with_comparison(mut self, comparison: AuthnContextComparison) -> Self {
        self.comparison = Some(comparison.as_str().to_string());
        self
    }
}

/// 认证上下文比较方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthnContextComparison {
    /// 精确匹配
    Exact,
    /// 更强
    Better,
    /// 更强或相等
    Maximum,
    /// 更弱或相等
    Minimum,
}

impl AuthnContextComparison {
    /// 获取比较方式字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthnContextComparison::Exact => "exact",
            AuthnContextComparison::Better => "better",
            AuthnContextComparison::Maximum => "maximum",
            AuthnContextComparison::Minimum => "minimum",
        }
    }
}

/// SAML 登出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlLogoutRequest {
    /// 请求 ID
    #[serde(rename = "@ID")]
    pub id: String,

    /// 版本
    #[serde(rename = "@Version")]
    pub version: String,

    /// 发行时间
    #[serde(rename = "@IssueInstant")]
    pub issue_instant: DateTime<Utc>,

    /// 目标
    #[serde(rename = "@Destination")]
    pub destination: Option<String>,

    /// 发行人
    #[serde(rename = "saml:Issuer")]
    pub issuer: String,

    /// 名称 ID
    #[serde(rename = "saml:NameID")]
    pub name_id: Option<super::assertion::SamlNameId>,

    /// 会话索引
    #[serde(rename = "samlp:SessionIndex")]
    pub session_index: Option<String>,

    /// 签名
    #[serde(rename = "ds:Signature")]
    pub signature: Option<String>,
}

impl SamlLogoutRequest {
    /// 创建新的登出请求
    pub fn new(id: impl Into<String>, issuer: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: "2.0".to_string(),
            issue_instant: Utc::now(),
            destination: None,
            issuer: issuer.into(),
            name_id: None,
            session_index: None,
            signature: None,
        }
    }

    /// 设置目标
    pub fn with_destination(mut self, destination: impl Into<String>) -> Self {
        self.destination = Some(destination.into());
        self
    }

    /// 设置名称 ID
    pub fn with_name_id(mut self, name_id: super::assertion::SamlNameId) -> Self {
        self.name_id = Some(name_id);
        self
    }

    /// 设置会话索引
    pub fn with_session_index(mut self, index: impl Into<String>) -> Self {
        self.session_index = Some(index.into());
        self
    }
}
