//! SAML 响应定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::assertion::SamlAssertion;

/// SAML 响应状态码
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamlStatusCode {
    /// 成功
    Success,
    /// 请求者错误
    Requester,
    /// 响应者错误
    Responder,
    /// 版本不匹配
    VersionMismatch,
    /// 自定义状态码
    Custom(String),
}

impl SamlStatusCode {
    /// 获取状态码 URI
    pub fn uri(&self) -> &str {
        match self {
            SamlStatusCode::Success => "urn:oasis:names:tc:SAML:2.0:status:Success",
            SamlStatusCode::Requester => "urn:oasis:names:tc:SAML:2.0:status:Requester",
            SamlStatusCode::Responder => "urn:oasis:names:tc:SAML:2.0:status:Responder",
            SamlStatusCode::VersionMismatch => "urn:oasis:names:tc:SAML:2.0:status:VersionMismatch",
            SamlStatusCode::Custom(uri) => uri,
        }
    }

    /// 从 URI 解析
    pub fn from_uri(uri: &str) -> Self {
        match uri {
            "urn:oasis:names:tc:SAML:2.0:status:Success" => SamlStatusCode::Success,
            "urn:oasis:names:tc:SAML:2.0:status:Requester" => SamlStatusCode::Requester,
            "urn:oasis:names:tc:SAML:2.0:status:Responder" => SamlStatusCode::Responder,
            "urn:oasis:names:tc:SAML:2.0:status:VersionMismatch" => SamlStatusCode::VersionMismatch,
            _ => SamlStatusCode::Custom(uri.to_string()),
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, SamlStatusCode::Success)
    }
}

/// SAML 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlResponse {
    /// 响应 ID
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

    /// 响应请求 ID
    #[serde(rename = "@InResponseTo")]
    pub in_response_to: Option<String>,

    /// 发行人
    #[serde(rename = "saml:Issuer")]
    pub issuer: String,

    /// 状态
    #[serde(rename = "samlp:Status")]
    pub status: SamlStatus,

    /// 断言
    #[serde(rename = "saml:Assertion")]
    pub assertion: Option<SamlAssertion>,

    /// 加密断言
    #[serde(rename = "saml:EncryptedAssertion")]
    pub encrypted_assertion: Option<String>,

    /// 签名
    #[serde(rename = "ds:Signature")]
    pub signature: Option<String>,
}

impl SamlResponse {
    /// 检查响应是否成功
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// 获取断言
    pub fn assertion(&self) -> Option<&SamlAssertion> {
        self.assertion.as_ref()
    }

    /// 获取用户 ID (从断言中提取)
    pub fn user_id(&self) -> Option<&str> {
        self.assertion.as_ref().and_then(|a| a.subject.as_ref()).map(|s| s.name_id.value.as_str())
    }

    /// 获取属性
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.assertion.as_ref().and_then(|a| a.attribute_statement.as_ref()).and_then(|s| s.get_attribute_value(name))
    }
}

/// SAML 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlStatus {
    /// 状态码
    #[serde(rename = "samlp:StatusCode")]
    pub status_code: SamlStatusCodeElement,

    /// 状态消息
    #[serde(rename = "samlp:StatusMessage")]
    pub status_message: Option<String>,

    /// 状态详情
    #[serde(rename = "samlp:StatusDetail")]
    pub status_detail: Option<String>,
}

impl SamlStatus {
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status_code.is_success()
    }

    /// 获取状态码
    pub fn code(&self) -> SamlStatusCode {
        SamlStatusCode::from_uri(&self.status_code.value)
    }
}

/// SAML 状态码元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlStatusCodeElement {
    /// 状态码值
    #[serde(rename = "@Value")]
    pub value: String,

    /// 子状态码
    #[serde(rename = "samlp:StatusCode")]
    pub sub_code: Option<Box<SamlStatusCodeElement>>,
}

impl SamlStatusCodeElement {
    /// 创建成功状态码
    pub fn success() -> Self {
        Self { value: SamlStatusCode::Success.uri().to_string(), sub_code: None }
    }

    /// 创建请求者错误状态码
    pub fn requester() -> Self {
        Self { value: SamlStatusCode::Requester.uri().to_string(), sub_code: None }
    }

    /// 创建响应者错误状态码
    pub fn responder() -> Self {
        Self { value: SamlStatusCode::Responder.uri().to_string(), sub_code: None }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.value == SamlStatusCode::Success.uri()
    }
}

/// SAML 登出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlLogoutResponse {
    /// 响应 ID
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

    /// 响应请求 ID
    #[serde(rename = "@InResponseTo")]
    pub in_response_to: Option<String>,

    /// 发行人
    #[serde(rename = "saml:Issuer")]
    pub issuer: String,

    /// 状态
    #[serde(rename = "samlp:Status")]
    pub status: SamlStatus,

    /// 签名
    #[serde(rename = "ds:Signature")]
    pub signature: Option<String>,
}

impl SamlLogoutResponse {
    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }
}
