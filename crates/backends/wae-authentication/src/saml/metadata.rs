//! SAML 元数据定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::config::{NameIdFormat, SamlBinding};

/// SAML 实体描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDescriptor {
    /// 实体 ID
    #[serde(rename = "@entityID")]
    pub entity_id: String,

    /// 有效期
    #[serde(rename = "@validUntil")]
    pub valid_until: Option<DateTime<Utc>>,

    /// 缓存时长
    #[serde(rename = "@cacheDuration")]
    pub cache_duration: Option<String>,

    /// SP 描述符
    #[serde(rename = "md:SPSSODescriptor")]
    pub sp_sso_descriptor: Option<SPSSODescriptor>,

    /// IdP 描述符
    #[serde(rename = "md:IDPSSODescriptor")]
    pub idp_sso_descriptor: Option<IDPSSODescriptor>,
}

/// SP SSO 描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPSSODescriptor {
    /// 是否需要签名断言
    #[serde(rename = "@WantAssertionsSigned")]
    pub want_assertions_signed: Option<bool>,

    /// 是否需要签名响应
    #[serde(rename = "@AuthnRequestsSigned")]
    pub authn_requests_signed: Option<bool>,

    /// 协议支持枚举
    #[serde(rename = "@protocolSupportEnumeration")]
    pub protocol_support_enumeration: String,

    /// 名称 ID 格式
    #[serde(rename = "md:NameIDFormat")]
    pub name_id_format: Vec<String>,

    /// 断言消费者服务
    #[serde(rename = "md:AssertionConsumerService")]
    pub assertion_consumer_service: Vec<AssertionConsumerService>,

    /// 单点登出服务
    #[serde(rename = "md:SingleLogoutService")]
    pub single_logout_service: Option<Vec<SingleLogoutService>>,

    /// 签名密钥描述符
    #[serde(rename = "md:KeyDescriptor")]
    pub key_descriptor: Option<Vec<KeyDescriptor>>,
}

/// IdP SSO 描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDPSSODescriptor {
    /// 是否需要签名请求
    #[serde(rename = "@WantAuthnRequestsSigned")]
    pub want_authn_requests_signed: Option<bool>,

    /// 协议支持枚举
    #[serde(rename = "@protocolSupportEnumeration")]
    pub protocol_support_enumeration: String,

    /// 名称 ID 格式
    #[serde(rename = "md:NameIDFormat")]
    pub name_id_format: Vec<String>,

    /// 单点登录服务
    #[serde(rename = "md:SingleSignOnService")]
    pub single_sign_on_service: Vec<SingleSignOnService>,

    /// 单点登出服务
    #[serde(rename = "md:SingleLogoutService")]
    pub single_logout_service: Option<Vec<SingleLogoutService>>,

    /// 签名密钥描述符
    #[serde(rename = "md:KeyDescriptor")]
    pub key_descriptor: Option<Vec<KeyDescriptor>>,
}

/// 断言消费者服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionConsumerService {
    /// 索引
    #[serde(rename = "@index")]
    pub index: u32,

    /// 是否默认
    #[serde(rename = "@isDefault")]
    pub is_default: Option<bool>,

    /// 绑定
    #[serde(rename = "@Binding")]
    pub binding: String,

    /// 位置
    #[serde(rename = "@Location")]
    pub location: String,
}

/// 单点登录服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleSignOnService {
    /// 绑定
    #[serde(rename = "@Binding")]
    pub binding: String,

    /// 位置
    #[serde(rename = "@Location")]
    pub location: String,
}

/// 单点登出服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleLogoutService {
    /// 绑定
    #[serde(rename = "@Binding")]
    pub binding: String,

    /// 位置
    #[serde(rename = "@Location")]
    pub location: String,

    /// 响应位置
    #[serde(rename = "@ResponseLocation")]
    pub response_location: Option<String>,
}

/// 密钥描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDescriptor {
    /// 用途
    #[serde(rename = "@use")]
    pub use_: Option<String>,

    /// 证书
    #[serde(rename = "ds:KeyInfo")]
    pub key_info: KeyInfo,
}

/// 密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// X509 数据
    #[serde(rename = "ds:X509Data")]
    pub x509_data: X509Data,
}

/// X509 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct X509Data {
    /// 证书
    #[serde(rename = "ds:X509Certificate")]
    pub certificate: String,
}

/// SP 元数据构建器
#[derive(Debug, Clone)]
pub struct SpMetadataBuilder {
    entity_id: String,
    acs_url: String,
    slo_url: Option<String>,
    name_id_formats: Vec<NameIdFormat>,
    want_assertions_signed: bool,
    authn_requests_signed: bool,
    certificate: Option<String>,
}

impl SpMetadataBuilder {
    /// 创建新的 SP 元数据构建器
    pub fn new(entity_id: impl Into<String>, acs_url: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            acs_url: acs_url.into(),
            slo_url: None,
            name_id_formats: vec![NameIdFormat::EmailAddress],
            want_assertions_signed: true,
            authn_requests_signed: false,
            certificate: None,
        }
    }

    /// 设置 SLO URL
    pub fn with_slo_url(mut self, url: impl Into<String>) -> Self {
        self.slo_url = Some(url.into());
        self
    }

    /// 设置名称 ID 格式
    pub fn with_name_id_formats(mut self, formats: Vec<NameIdFormat>) -> Self {
        self.name_id_formats = formats;
        self
    }

    /// 设置是否需要签名断言
    pub fn with_want_assertions_signed(mut self, want: bool) -> Self {
        self.want_assertions_signed = want;
        self
    }

    /// 设置是否签名认证请求
    pub fn with_authn_requests_signed(mut self, signed: bool) -> Self {
        self.authn_requests_signed = signed;
        self
    }

    /// 设置证书
    pub fn with_certificate(mut self, cert: impl Into<String>) -> Self {
        self.certificate = Some(cert.into());
        self
    }

    /// 构建元数据
    pub fn build(self) -> EntityDescriptor {
        let acs = AssertionConsumerService {
            index: 0,
            is_default: Some(true),
            binding: SamlBinding::HttpPost.uri().to_string(),
            location: self.acs_url,
        };

        let slo_services = self.slo_url.map(|url| {
            vec![SingleLogoutService {
                binding: SamlBinding::HttpRedirect.uri().to_string(),
                location: url,
                response_location: None,
            }]
        });

        let key_descriptors = self.certificate.map(|cert| {
            vec![KeyDescriptor {
                use_: Some("signing".to_string()),
                key_info: KeyInfo { x509_data: X509Data { certificate: cert } },
            }]
        });

        EntityDescriptor {
            entity_id: self.entity_id,
            valid_until: None,
            cache_duration: None,
            sp_sso_descriptor: Some(SPSSODescriptor {
                want_assertions_signed: Some(self.want_assertions_signed),
                authn_requests_signed: Some(self.authn_requests_signed),
                protocol_support_enumeration: "urn:oasis:names:tc:SAML:2.0:protocol".to_string(),
                name_id_format: self.name_id_formats.iter().map(|f| f.uri().to_string()).collect(),
                assertion_consumer_service: vec![acs],
                single_logout_service: slo_services,
                key_descriptor: key_descriptors,
            }),
            idp_sso_descriptor: None,
        }
    }
}
