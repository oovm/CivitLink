//! SAML 配置模块

use std::collections::HashMap;

/// SAML 绑定类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamlBinding {
    /// HTTP Redirect 绑定
    HttpRedirect,
    /// HTTP POST 绑定
    HttpPost,
    /// HTTP Artifact 绑定
    HttpArtifact,
    /// SOAP 绑定
    Soap,
}

impl SamlBinding {
    /// 获取绑定 URI
    pub fn uri(&self) -> &'static str {
        match self {
            SamlBinding::HttpRedirect => "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect",
            SamlBinding::HttpPost => "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST",
            SamlBinding::HttpArtifact => "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Artifact",
            SamlBinding::Soap => "urn:oasis:names:tc:SAML:2.0:bindings:SOAP",
        }
    }

    /// 从 URI 解析
    pub fn from_uri(uri: &str) -> Option<Self> {
        match uri {
            "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" => Some(SamlBinding::HttpRedirect),
            "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" => Some(SamlBinding::HttpPost),
            "urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Artifact" => Some(SamlBinding::HttpArtifact),
            "urn:oasis:names:tc:SAML:2.0:bindings:SOAP" => Some(SamlBinding::Soap),
            _ => None,
        }
    }
}

/// 名称 ID 格式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameIdFormat {
    /// 未指定
    Unspecified,
    /// 电子邮箱
    EmailAddress,
    /// X.509 主题名称
    X509SubjectName,
    /// Windows 域限定名
    WindowsDomainQualifiedName,
    /// Kerberos 主体名称
    KerberosPrincipalName,
    /// 实体标识符
    EntityIdentifier,
    /// 持久化标识符
    Persistent,
    /// 临时标识符
    Transient,
    /// 自定义格式
    Custom(String),
}

impl NameIdFormat {
    /// 获取格式 URI
    pub fn uri(&self) -> &str {
        match self {
            NameIdFormat::Unspecified => "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified",
            NameIdFormat::EmailAddress => "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
            NameIdFormat::X509SubjectName => "urn:oasis:names:tc:SAML:1.1:nameid-format:X509SubjectName",
            NameIdFormat::WindowsDomainQualifiedName => "urn:oasis:names:tc:SAML:1.1:nameid-format:WindowsDomainQualifiedName",
            NameIdFormat::KerberosPrincipalName => "urn:oasis:names:tc:SAML:2.0:nameid-format:kerberos",
            NameIdFormat::EntityIdentifier => "urn:oasis:names:tc:SAML:2.0:nameid-format:entity",
            NameIdFormat::Persistent => "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
            NameIdFormat::Transient => "urn:oasis:names:tc:SAML:2.0:nameid-format:transient",
            NameIdFormat::Custom(uri) => uri,
        }
    }

    /// 从 URI 解析
    pub fn from_uri(uri: &str) -> Self {
        match uri {
            "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified" => NameIdFormat::Unspecified,
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" => NameIdFormat::EmailAddress,
            "urn:oasis:names:tc:SAML:1.1:nameid-format:X509SubjectName" => NameIdFormat::X509SubjectName,
            "urn:oasis:names:tc:SAML:1.1:nameid-format:WindowsDomainQualifiedName" => NameIdFormat::WindowsDomainQualifiedName,
            "urn:oasis:names:tc:SAML:2.0:nameid-format:kerberos" => NameIdFormat::KerberosPrincipalName,
            "urn:oasis:names:tc:SAML:2.0:nameid-format:entity" => NameIdFormat::EntityIdentifier,
            "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent" => NameIdFormat::Persistent,
            "urn:oasis:names:tc:SAML:2.0:nameid-format:transient" => NameIdFormat::Transient,
            _ => NameIdFormat::Custom(uri.to_string()),
        }
    }
}

/// SAML 服务提供者 (SP) 配置
#[derive(Debug, Clone)]
pub struct ServiceProviderConfig {
    /// 实体 ID
    pub entity_id: String,

    /// ACS (Assertion Consumer Service) URL
    pub acs_url: String,

    /// SLO (Single Logout) URL
    pub slo_url: Option<String>,

    /// 名称 ID 格式
    pub name_id_format: NameIdFormat,

    /// 需要的属性
    pub required_attributes: Vec<String>,

    /// 可选属性
    pub optional_attributes: Vec<String>,

    /// 是否需要签名
    pub want_assertions_signed: bool,

    /// 是否需要响应签名
    pub want_response_signed: bool,

    /// 私钥 (用于签名请求)
    pub private_key: Option<String>,

    /// 证书
    pub certificate: Option<String>,
}

impl ServiceProviderConfig {
    /// 创建新的 SP 配置
    pub fn new(entity_id: impl Into<String>, acs_url: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            acs_url: acs_url.into(),
            slo_url: None,
            name_id_format: NameIdFormat::EmailAddress,
            required_attributes: Vec::new(),
            optional_attributes: Vec::new(),
            want_assertions_signed: true,
            want_response_signed: true,
            private_key: None,
            certificate: None,
        }
    }

    /// 设置 SLO URL
    pub fn with_slo_url(mut self, url: impl Into<String>) -> Self {
        self.slo_url = Some(url.into());
        self
    }

    /// 设置名称 ID 格式
    pub fn with_name_id_format(mut self, format: NameIdFormat) -> Self {
        self.name_id_format = format;
        self
    }

    /// 添加需要的属性
    pub fn add_required_attribute(mut self, attr: impl Into<String>) -> Self {
        self.required_attributes.push(attr.into());
        self
    }

    /// 添加可选属性
    pub fn add_optional_attribute(mut self, attr: impl Into<String>) -> Self {
        self.optional_attributes.push(attr.into());
        self
    }

    /// 设置签名密钥
    pub fn with_signing_key(mut self, private_key: impl Into<String>, certificate: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self.certificate = Some(certificate.into());
        self
    }
}

/// SAML 身份提供者 (IdP) 配置
#[derive(Debug, Clone)]
pub struct IdentityProviderConfig {
    /// 实体 ID
    pub entity_id: String,

    /// SSO URL
    pub sso_url: String,

    /// SLO URL
    pub slo_url: Option<String>,

    /// SSO 绑定方式
    pub sso_binding: SamlBinding,

    /// SLO 绑定方式
    pub slo_binding: SamlBinding,

    /// 签名证书
    pub signing_certificate: String,

    /// 加密证书
    pub encryption_certificate: Option<String>,

    /// 额外配置
    pub extra: HashMap<String, String>,
}

impl IdentityProviderConfig {
    /// 创建新的 IdP 配置
    pub fn new(entity_id: impl Into<String>, sso_url: impl Into<String>, certificate: impl Into<String>) -> Self {
        Self {
            entity_id: entity_id.into(),
            sso_url: sso_url.into(),
            slo_url: None,
            sso_binding: SamlBinding::HttpRedirect,
            slo_binding: SamlBinding::HttpRedirect,
            signing_certificate: certificate.into(),
            encryption_certificate: None,
            extra: HashMap::new(),
        }
    }

    /// 设置 SLO URL
    pub fn with_slo_url(mut self, url: impl Into<String>) -> Self {
        self.slo_url = Some(url.into());
        self
    }

    /// 设置 SSO 绑定方式
    pub fn with_sso_binding(mut self, binding: SamlBinding) -> Self {
        self.sso_binding = binding;
        self
    }

    /// 设置 SLO 绑定方式
    pub fn with_slo_binding(mut self, binding: SamlBinding) -> Self {
        self.slo_binding = binding;
        self
    }

    /// 设置加密证书
    pub fn with_encryption_certificate(mut self, cert: impl Into<String>) -> Self {
        self.encryption_certificate = Some(cert.into());
        self
    }
}

/// SAML 配置
#[derive(Debug, Clone)]
pub struct SamlConfig {
    /// 服务提供者配置
    pub sp: ServiceProviderConfig,

    /// 身份提供者配置
    pub idp: IdentityProviderConfig,

    /// 时钟偏移容忍度 (秒)
    pub clock_skew_seconds: i64,

    /// 断言有效期 (秒)
    pub assertion_validity_seconds: i64,

    /// 是否验证受众
    pub validate_audience: bool,

    /// 是否验证发行人
    pub validate_issuer: bool,

    /// 是否验证目标
    pub validate_destination: bool,

    /// 是否检测重放攻击
    pub detect_replay: bool,
}

impl SamlConfig {
    /// 创建新的 SAML 配置
    pub fn new(sp: ServiceProviderConfig, idp: IdentityProviderConfig) -> Self {
        Self {
            sp,
            idp,
            clock_skew_seconds: 60,
            assertion_validity_seconds: 300,
            validate_audience: true,
            validate_issuer: true,
            validate_destination: true,
            detect_replay: true,
        }
    }

    /// 设置时钟偏移容忍度
    pub fn with_clock_skew(mut self, seconds: i64) -> Self {
        self.clock_skew_seconds = seconds;
        self
    }

    /// 设置断言有效期
    pub fn with_assertion_validity(mut self, seconds: i64) -> Self {
        self.assertion_validity_seconds = seconds;
        self
    }
}
