//! OAuth2 客户端实现

use crate::oauth2::{AuthorizationUrl, OAuth2ClientConfig, TokenResponse, UserInfo};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use wae_request::{HttpClient, HttpClientConfig};
use wae_types::{WaeError, WaeErrorKind};

/// OAuth2 结果类型
pub type OAuth2Result<T> = Result<T, WaeError>;

/// OAuth2 客户端
#[derive(Debug, Clone)]
pub struct OAuth2Client {
    config: OAuth2ClientConfig,
    http_client: HttpClient,
}

impl OAuth2Client {
    /// 创建新的 OAuth2 客户端
    ///
    /// # Arguments
    /// * `config` - 客户端配置
    pub fn new(config: OAuth2ClientConfig) -> OAuth2Result<Self> {
        let http_config = HttpClientConfig {
            timeout: std::time::Duration::from_millis(config.timeout_ms),
            connect_timeout: std::time::Duration::from_secs(10),
            user_agent: "wae-oauth2/0.1.0".to_string(),
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(1000),
            default_headers: HashMap::new(),
        };

        let http_client = HttpClient::new(http_config);

        Ok(Self { config, http_client })
    }

    /// 生成授权 URL
    ///
    /// # Returns
    /// 返回授权 URL、状态参数和 PKCE code verifier
    pub fn authorization_url(&self) -> OAuth2Result<AuthorizationUrl> {
        let state = self.generate_state();
        let mut params: Vec<(String, String)> = vec![
            ("client_id".to_string(), self.config.provider.client_id.clone()),
            ("redirect_uri".to_string(), self.config.provider.redirect_uri.clone()),
            ("response_type".to_string(), "code".to_string()),
            ("state".to_string(), state.clone()),
        ];

        if !self.config.provider.scopes.is_empty() {
            params.push(("scope".to_string(), self.config.provider.scopes.join(" ")));
        }

        let code_verifier = if self.config.use_pkce {
            let verifier = self.generate_code_verifier();
            let challenge = self.generate_code_challenge(&verifier);
            params.push(("code_challenge".to_string(), challenge));
            params.push(("code_challenge_method".to_string(), "S256".to_string()));
            Some(verifier)
        }
        else {
            None
        };

        for (key, value) in &self.config.provider.extra_params {
            params.push((key.clone(), value.clone()));
        }

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", wae_types::url_encode(k), wae_types::url_encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}?{}", self.config.provider.authorization_url, query);

        Ok(AuthorizationUrl { url, state, code_verifier })
    }

    /// 使用授权码交换令牌
    ///
    /// # Arguments
    /// * `code` - 授权码
    /// * `code_verifier` - PKCE code verifier (如果启用了 PKCE)
    pub async fn exchange_code(&self, code: &str, code_verifier: Option<&str>) -> OAuth2Result<TokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code".to_string());
        params.insert("code", code.to_string());
        params.insert("redirect_uri", self.config.provider.redirect_uri.clone());
        params.insert("client_id", self.config.provider.client_id.clone());
        params.insert("client_secret", self.config.provider.client_secret.clone());

        if let Some(verifier) = code_verifier {
            params.insert("code_verifier", verifier.to_string());
        }

        let form_body = self.encode_form_data(&params);

        let response = self
            .http_client
            .post_with_headers(&self.config.provider.token_url, form_body.into_bytes(), self.form_headers())
            .await
            .map_err(WaeError::from)?;

        if !response.is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(WaeError::new(WaeErrorKind::OAuth2ProviderError { message: error_text }));
        }

        let token_response: TokenResponse = response.json().map_err(WaeError::from)?;
        Ok(token_response)
    }

    /// 刷新访问令牌
    ///
    /// # Arguments
    /// * `refresh_token` - 刷新令牌
    pub async fn refresh_token(&self, refresh_token: &str) -> OAuth2Result<TokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token".to_string());
        params.insert("refresh_token", refresh_token.to_string());
        params.insert("client_id", self.config.provider.client_id.clone());
        params.insert("client_secret", self.config.provider.client_secret.clone());

        let form_body = self.encode_form_data(&params);

        let response = self
            .http_client
            .post_with_headers(&self.config.provider.token_url, form_body.into_bytes(), self.form_headers())
            .await
            .map_err(WaeError::from)?;

        if !response.is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(WaeError::new(WaeErrorKind::OAuth2ProviderError { message: error_text }));
        }

        let token_response: TokenResponse = response.json().map_err(WaeError::from)?;
        Ok(token_response)
    }

    /// 获取用户信息
    ///
    /// # Arguments
    /// * `access_token` - 访问令牌
    pub async fn get_user_info(&self, access_token: &str) -> OAuth2Result<UserInfo> {
        let userinfo_url = self
            .config
            .provider
            .userinfo_url
            .as_ref()
            .ok_or_else(|| WaeError::config_invalid("userinfo_url", "not configured"))?;

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));

        let response = self.http_client.get_with_headers(userinfo_url, headers).await.map_err(WaeError::from)?;

        if !response.is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(WaeError::new(WaeErrorKind::OAuth2ProviderError { message: error_text }));
        }

        let user_info: UserInfo = response.json().map_err(WaeError::from)?;
        Ok(user_info)
    }

    /// 撤销令牌
    ///
    /// # Arguments
    /// * `token` - 要撤销的令牌
    /// * `token_type_hint` - 令牌类型提示 (access_token 或 refresh_token)
    pub async fn revoke_token(&self, token: &str, token_type_hint: Option<&str>) -> OAuth2Result<()> {
        let revocation_url = self
            .config
            .provider
            .revocation_url
            .as_ref()
            .ok_or_else(|| WaeError::config_invalid("revocation_url", "not configured"))?;

        let mut params = HashMap::new();
        params.insert("token", token.to_string());
        params.insert("client_id", self.config.provider.client_id.clone());
        params.insert("client_secret", self.config.provider.client_secret.clone());

        if let Some(hint) = token_type_hint {
            params.insert("token_type_hint", hint.to_string());
        }

        let form_body = self.encode_form_data(&params);

        let response = self
            .http_client
            .post_with_headers(revocation_url, form_body.into_bytes(), self.form_headers())
            .await
            .map_err(WaeError::from)?;

        if !response.is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(WaeError::new(WaeErrorKind::OAuth2ProviderError { message: error_text }));
        }

        Ok(())
    }

    /// 验证状态参数
    ///
    /// # Arguments
    /// * `expected` - 期望的状态值
    /// * `received` - 接收到的状态值
    pub fn validate_state(&self, expected: &str, received: &str) -> OAuth2Result<()> {
        if !self.config.use_state {
            return Ok(());
        }

        if expected == received { Ok(()) } else { Err(WaeError::new(WaeErrorKind::StateMismatch)) }
    }

    fn generate_state(&self) -> String {
        uuid::Uuid::new_v4().to_string().replace('-', "")
    }

    fn generate_code_verifier(&self) -> String {
        let random_bytes: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        URL_SAFE_NO_PAD.encode(&random_bytes)
    }

    fn generate_code_challenge(&self, verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        URL_SAFE_NO_PAD.encode(&hash)
    }

    fn encode_form_data(&self, params: &HashMap<&str, String>) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    fn form_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string());
        headers
    }

    /// 获取提供者名称
    pub fn provider_name(&self) -> &str {
        &self.config.provider.name
    }

    /// 获取配置
    pub fn config(&self) -> &OAuth2ClientConfig {
        &self.config
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
