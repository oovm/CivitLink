//! SAML 断言定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::config::NameIdFormat;

/// SAML 断言
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAssertion {
    /// 断言 ID
    #[serde(rename = "@ID")]
    pub id: String,

    /// 发行时间
    #[serde(rename = "@IssueInstant")]
    pub issue_instant: DateTime<Utc>,

    /// 版本
    #[serde(rename = "@Version")]
    pub version: String,

    /// 发行人
    #[serde(rename = "saml:Issuer")]
    pub issuer: String,

    /// 主题
    #[serde(rename = "saml:Subject")]
    pub subject: Option<SamlSubject>,

    /// 条件
    #[serde(rename = "saml:Conditions")]
    pub conditions: Option<SamlConditions>,

    /// 认证声明
    #[serde(rename = "saml:AuthnStatement")]
    pub authn_statement: Option<SamlAuthnStatement>,

    /// 属性声明
    #[serde(rename = "saml:AttributeStatement")]
    pub attribute_statement: Option<SamlAttributeStatement>,

    /// 签名
    #[serde(rename = "ds:Signature")]
    pub signature: Option<String>,
}

/// SAML 主题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlSubject {
    /// 名称 ID
    #[serde(rename = "saml:NameID")]
    pub name_id: SamlNameId,

    /// 主题确认
    #[serde(rename = "saml:SubjectConfirmation")]
    pub subject_confirmation: Option<SamlSubjectConfirmation>,
}

/// SAML 名称 ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlNameId {
    /// 格式
    #[serde(rename = "@Format")]
    pub format: Option<String>,

    /// 名称限定符
    #[serde(rename = "@NameQualifier")]
    pub name_qualifier: Option<String>,

    /// 值
    #[serde(rename = "$value")]
    pub value: String,
}

impl SamlNameId {
    /// 创建新的名称 ID
    pub fn new(value: impl Into<String>) -> Self {
        Self { format: None, name_qualifier: None, value: value.into() }
    }

    /// 设置格式
    pub fn with_format(mut self, format: NameIdFormat) -> Self {
        self.format = Some(format.uri().to_string());
        self
    }

    /// 获取名称 ID 格式
    pub fn name_id_format(&self) -> Option<NameIdFormat> {
        self.format.as_ref().map(|f| NameIdFormat::from_uri(f))
    }
}

/// SAML 主题确认
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlSubjectConfirmation {
    /// 方法
    #[serde(rename = "@Method")]
    pub method: String,

    /// 主题确认数据
    #[serde(rename = "saml:SubjectConfirmationData")]
    pub data: Option<SamlSubjectConfirmationData>,
}

/// SAML 主题确认数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlSubjectConfirmationData {
    /// 接收者
    #[serde(rename = "@Recipient")]
    pub recipient: Option<String>,

    /// 响应时间
    #[serde(rename = "@NotOnOrAfter")]
    pub not_on_or_after: Option<DateTime<Utc>>,

    /// 生效时间
    #[serde(rename = "@NotBefore")]
    pub not_before: Option<DateTime<Utc>>,

    /// 请求 ID
    #[serde(rename = "@InResponseTo")]
    pub in_response_to: Option<String>,
}

/// SAML 条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConditions {
    /// 生效时间
    #[serde(rename = "@NotBefore")]
    pub not_before: Option<DateTime<Utc>>,

    /// 过期时间
    #[serde(rename = "@NotOnOrAfter")]
    pub not_on_or_after: Option<DateTime<Utc>>,

    /// 受众限制
    #[serde(rename = "saml:AudienceRestriction")]
    pub audience_restriction: Option<SamlAudienceRestriction>,
}

/// SAML 受众限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAudienceRestriction {
    /// 受众列表
    #[serde(rename = "saml:Audience")]
    pub audience: Vec<String>,
}

/// SAML 认证声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAuthnStatement {
    /// 认证时间
    #[serde(rename = "@AuthnInstant")]
    pub authn_instant: DateTime<Utc>,

    /// 会话索引
    #[serde(rename = "@SessionIndex")]
    pub session_index: Option<String>,

    /// 会话过期时间
    #[serde(rename = "@SessionNotOnOrAfter")]
    pub session_not_on_or_after: Option<DateTime<Utc>>,

    /// 认证上下文
    #[serde(rename = "saml:AuthnContext")]
    pub authn_context: SamlAuthnContext,
}

/// SAML 认证上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAuthnContext {
    /// 认证上下文类引用
    #[serde(rename = "saml:AuthnContextClassRef")]
    pub class_ref: Option<String>,
}

/// SAML 属性声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAttributeStatement {
    /// 属性列表
    #[serde(rename = "saml:Attribute")]
    pub attributes: Vec<SamlAttribute>,
}

/// SAML 属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAttribute {
    /// 属性名
    #[serde(rename = "@Name")]
    pub name: String,

    /// 属性名格式
    #[serde(rename = "@NameFormat")]
    pub name_format: Option<String>,

    /// 友好名称
    #[serde(rename = "@FriendlyName")]
    pub friendly_name: Option<String>,

    /// 属性值
    #[serde(rename = "saml:AttributeValue")]
    pub values: Vec<String>,
}

impl SamlAttributeStatement {
    /// 获取属性值
    pub fn get_attribute(&self, name: &str) -> Option<&SamlAttribute> {
        self.attributes.iter().find(|a| a.name == name)
    }

    /// 获取第一个属性值
    pub fn get_attribute_value(&self, name: &str) -> Option<&str> {
        self.get_attribute(name).and_then(|a| a.values.first().map(|s| s.as_str()))
    }

    /// 获取所有属性值
    pub fn get_attribute_values(&self, name: &str) -> Vec<&str> {
        self.get_attribute(name).map(|a| a.values.iter().map(|s| s.as_str()).collect()).unwrap_or_default()
    }

    /// 转换为 HashMap
    pub fn to_map(&self) -> HashMap<String, Vec<String>> {
        self.attributes.iter().map(|a| (a.name.clone(), a.values.clone())).collect()
    }
}

/// 认证上下文类引用
pub mod authn_context_class {
    /// 未指定
    pub const UNSPECIFIED: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:unspecified";

    /// 密码
    pub const PASSWORD: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:Password";

    /// 密码保护传输
    pub const PASSWORD_PROTECTED_TRANSPORT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport";

    /// X.509
    pub const X509: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:X509";

    /// Kerberos
    pub const KERBEROS: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:Kerberos";

    /// TLS 客户端
    pub const TLS_CLIENT: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:TLSClient";

    /// 之前会话
    pub const PREVIOUS_SESSION: &str = "urn:oasis:names:tc:SAML:2.0:ac:classes:PreviousSession";
}
