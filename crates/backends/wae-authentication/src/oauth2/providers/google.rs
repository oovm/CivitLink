//! Google OAuth2 提供者配置

use crate::oauth2::OAuth2ProviderConfig;

/// Google OAuth2 端点
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

/// 创建 Google OAuth2 配置
///
/// # Arguments
/// * `client_id` - Google 客户端 ID
/// * `client_secret` - Google 客户端密钥
/// * `redirect_uri` - 重定向 URI
pub fn google_config(
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    redirect_uri: impl Into<String>,
) -> OAuth2ProviderConfig {
    OAuth2ProviderConfig::new("google", client_id, client_secret, redirect_uri)
        .with_authorization_url(GOOGLE_AUTH_URL)
        .with_token_url(GOOGLE_TOKEN_URL)
        .with_userinfo_url(GOOGLE_USERINFO_URL)
        .with_scopes(vec!["openid".to_string(), "email".to_string(), "profile".to_string()])
}

/// Google OAuth2 默认范围
pub fn google_default_scopes() -> Vec<String> {
    vec!["openid".to_string(), "email".to_string(), "profile".to_string()]
}
