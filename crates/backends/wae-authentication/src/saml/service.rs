//! SAML 服务实现

use crate::saml::{
    AuthnContextComparison, SamlAuthnRequest, SamlConfig, SamlLogoutRequest, SamlLogoutResponse, SamlNameIdPolicy,
    SamlRequestedAuthnContext, SamlResponse, SpMetadataBuilder,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use chrono::Utc;
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use std::io::{Read, Write};
use uuid::Uuid;
use wae_types::{WaeError, WaeErrorKind};

/// SAML 结果类型
pub type SamlResult<T> = Result<T, WaeError>;

/// SAML 服务
#[derive(Debug, Clone)]
pub struct SamlService {
    config: SamlConfig,
}

impl SamlService {
    /// 创建新的 SAML 服务
    pub fn new(config: SamlConfig) -> Self {
        Self { config }
    }

    /// 创建认证请求 URL (HTTP Redirect 绑定)
    pub fn create_authn_request_url(&self) -> SamlResult<String> {
        let request = self.create_authn_request()?;
        let xml = self.serialize_authn_request(&request)?;
        let encoded = self.encode_redirect_request(&xml)?;

        let mut url = self.config.idp.sso_url.clone();
        url.push_str("?SAMLRequest=");
        url.push_str(&encoded);

        Ok(url)
    }

    /// 创建认证请求
    pub fn create_authn_request(&self) -> SamlResult<SamlAuthnRequest> {
        let id = format!("id{}", Uuid::new_v4().simple());
        let name_id_policy = SamlNameIdPolicy::new().with_format(self.config.sp.name_id_format.clone()).with_allow_create(true);

        let authn_context = SamlRequestedAuthnContext::new(vec![
            "urn:oasis:names:tc:SAML:2.0:ac:classes:PasswordProtectedTransport".to_string(),
        ])
        .with_comparison(AuthnContextComparison::Minimum);

        Ok(SamlAuthnRequest::new(id, &self.config.sp.entity_id)
            .with_destination(&self.config.idp.sso_url)
            .with_protocol_binding(self.config.idp.sso_binding)
            .with_acs_url(&self.config.sp.acs_url)
            .with_name_id_policy(name_id_policy)
            .with_authn_context(authn_context))
    }

    /// 处理认证响应
    pub fn process_authn_response(&self, saml_response: &str) -> SamlResult<SamlResponse> {
        let decoded = BASE64_STANDARD
            .decode(saml_response)
            .map_err(|e| WaeError::new(WaeErrorKind::Base64DecodeError { reason: e.to_string() }))?;

        let xml =
            String::from_utf8(decoded).map_err(|e| WaeError::new(WaeErrorKind::XmlParsingError { reason: e.to_string() }))?;

        let response: SamlResponse = quick_xml::de::from_str(&xml)
            .map_err(|e| WaeError::new(WaeErrorKind::XmlParsingError { reason: e.to_string() }))?;

        self.validate_response(&response)?;

        Ok(response)
    }

    /// 验证响应
    fn validate_response(&self, response: &SamlResponse) -> SamlResult<()> {
        if !response.is_success() {
            return Err(WaeError::new(WaeErrorKind::InvalidSamlResponse {
                reason: format!("SAML response status: {:?}", response.status.code()),
            }));
        }

        if self.config.validate_issuer {
            if response.issuer != self.config.idp.entity_id {
                return Err(WaeError::new(WaeErrorKind::SamlIssuerValidationFailed {
                    expected: self.config.idp.entity_id.clone(),
                    actual: response.issuer.clone(),
                }));
            }
        }

        if let Some(ref assertion) = response.assertion {
            self.validate_assertion(assertion)?;
        }

        Ok(())
    }

    /// 验证断言
    fn validate_assertion(&self, assertion: &crate::saml::SamlAssertion) -> SamlResult<()> {
        let now = Utc::now().timestamp();

        if let Some(ref conditions) = assertion.conditions {
            if let Some(not_before) = conditions.not_before {
                if now < not_before.timestamp() - self.config.clock_skew_seconds {
                    return Err(WaeError::new(WaeErrorKind::AssertionNotYetValid));
                }
            }

            if let Some(not_on_or_after) = conditions.not_on_or_after {
                if now >= not_on_or_after.timestamp() + self.config.clock_skew_seconds {
                    return Err(WaeError::new(WaeErrorKind::AssertionExpired));
                }
            }

            if self.config.validate_audience {
                if let Some(ref restriction) = conditions.audience_restriction {
                    if !restriction.audience.contains(&self.config.sp.entity_id) {
                        return Err(WaeError::new(WaeErrorKind::SamlAudienceValidationFailed {
                            expected: self.config.sp.entity_id.clone(),
                            actual: restriction.audience.first().cloned().unwrap_or_default(),
                        }));
                    }
                }
            }
        }

        Ok(())
    }

    /// 创建登出请求 URL
    pub fn create_logout_request_url(&self, name_id: &str, session_index: Option<&str>) -> SamlResult<String> {
        let slo_url =
            self.config.idp.slo_url.as_ref().ok_or_else(|| WaeError::config_invalid("slo_url", "SLO URL not configured"))?;

        let request = self.create_logout_request(name_id, session_index)?;
        let xml = self.serialize_logout_request(&request)?;
        let encoded = self.encode_redirect_request(&xml)?;

        let mut url = slo_url.clone();
        url.push_str("?SAMLRequest=");
        url.push_str(&encoded);

        Ok(url)
    }

    /// 创建登出请求
    pub fn create_logout_request(&self, name_id: &str, session_index: Option<&str>) -> SamlResult<SamlLogoutRequest> {
        let id = format!("id{}", Uuid::new_v4().simple());
        let slo_url =
            self.config.idp.slo_url.as_ref().ok_or_else(|| WaeError::config_invalid("slo_url", "SLO URL not configured"))?;

        let mut request = SamlLogoutRequest::new(id, &self.config.sp.entity_id)
            .with_destination(slo_url)
            .with_name_id(crate::saml::SamlNameId::new(name_id));

        if let Some(idx) = session_index {
            request = request.with_session_index(idx);
        }

        Ok(request)
    }

    /// 处理登出响应
    pub fn process_logout_response(&self, saml_response: &str) -> SamlResult<SamlLogoutResponse> {
        let decoded = BASE64_STANDARD
            .decode(saml_response)
            .map_err(|e| WaeError::new(WaeErrorKind::Base64DecodeError { reason: e.to_string() }))?;

        let xml =
            String::from_utf8(decoded).map_err(|e| WaeError::new(WaeErrorKind::XmlParsingError { reason: e.to_string() }))?;

        let response: SamlLogoutResponse = quick_xml::de::from_str(&xml)
            .map_err(|e| WaeError::new(WaeErrorKind::XmlParsingError { reason: e.to_string() }))?;

        Ok(response)
    }

    /// 生成 SP 元数据
    pub fn generate_sp_metadata(&self) -> String {
        let builder = SpMetadataBuilder::new(&self.config.sp.entity_id, &self.config.sp.acs_url)
            .with_want_assertions_signed(self.config.sp.want_assertions_signed)
            .with_authn_requests_signed(self.config.sp.want_response_signed);

        let metadata = if let Some(ref slo_url) = self.config.sp.slo_url { builder.with_slo_url(slo_url) } else { builder };

        let entity = metadata.build();

        quick_xml::se::to_string(&entity).unwrap_or_default()
    }

    /// 序列化认证请求
    fn serialize_authn_request(&self, request: &SamlAuthnRequest) -> SamlResult<String> {
        quick_xml::se::to_string(request).map_err(|e| WaeError::new(WaeErrorKind::XmlParsingError { reason: e.to_string() }))
    }

    /// 序列化登出请求
    fn serialize_logout_request(&self, request: &SamlLogoutRequest) -> SamlResult<String> {
        quick_xml::se::to_string(request).map_err(|e| WaeError::new(WaeErrorKind::XmlParsingError { reason: e.to_string() }))
    }

    /// 编码重定向请求 (Deflate + Base64)
    fn encode_redirect_request(&self, xml: &str) -> SamlResult<String> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(xml.as_bytes())
            .map_err(|e| WaeError::new(WaeErrorKind::CompressionError { reason: e.to_string() }))?;
        let compressed =
            encoder.finish().map_err(|e| WaeError::new(WaeErrorKind::CompressionError { reason: e.to_string() }))?;

        Ok(BASE64_STANDARD.encode(&compressed))
    }

    /// 解码重定向请求
    pub fn decode_redirect_request(&self, encoded: &str) -> SamlResult<String> {
        let decoded = BASE64_STANDARD
            .decode(encoded)
            .map_err(|e| WaeError::new(WaeErrorKind::Base64DecodeError { reason: e.to_string() }))?;

        let mut decoder = DeflateDecoder::new(&decoded[..]);
        let mut decompressed = String::new();
        decoder
            .read_to_string(&mut decompressed)
            .map_err(|e| WaeError::new(WaeErrorKind::CompressionError { reason: e.to_string() }))?;

        Ok(decompressed)
    }

    /// 获取配置
    pub fn config(&self) -> &SamlConfig {
        &self.config
    }
}

/// 便捷函数：创建 SAML 服务
pub fn create_saml_service(
    sp_entity_id: impl Into<String>,
    sp_acs_url: impl Into<String>,
    idp_entity_id: impl Into<String>,
    idp_sso_url: impl Into<String>,
    idp_certificate: impl Into<String>,
) -> SamlService {
    use crate::saml::{IdentityProviderConfig, ServiceProviderConfig};

    let sp = ServiceProviderConfig::new(sp_entity_id, sp_acs_url);
    let idp = IdentityProviderConfig::new(idp_entity_id, idp_sso_url, idp_certificate);
    let config = SamlConfig::new(sp, idp);

    SamlService::new(config)
}
