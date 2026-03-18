//! GitHub OAuth2 提供者配置

use crate::oauth2::OAuth2ProviderConfig;

/// GitHub OAuth2 端点
const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USERINFO_URL: &str = "https://api.github.com/user";

/// 创建 GitHub OAuth2 配置
///
/// # Arguments
/// * `client_id` - GitHub 客户端 ID
/// * `client_secret` - GitHub 客户端密钥
/// * `redirect_uri` - 重定向 URI
pub fn github_config(
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    redirect_uri: impl Into<String>,
) -> OAuth2ProviderConfig {
    OAuth2ProviderConfig::new("github", client_id, client_secret, redirect_uri)
        .with_authorization_url(GITHUB_AUTH_URL)
        .with_token_url(GITHUB_TOKEN_URL)
        .with_userinfo_url(GITHUB_USERINFO_URL)
        .with_scopes(vec!["user:email".to_string(), "read:user".to_string()])
}

/// GitHub OAuth2 默认范围
pub fn github_default_scopes() -> Vec<String> {
    vec!["user:email".to_string(), "read:user".to_string()]
}

/// GitHub OAuth2 只读范围
pub fn github_readonly_scopes() -> Vec<String> {
    vec!["read:user".to_string(), "user:email".to_string()]
}

/// GitHub OAuth2 完整范围
pub fn github_full_scopes() -> Vec<String> {
    vec!["user".to_string(), "user:email".to_string(), "repo".to_string(), "gist".to_string()]
}
