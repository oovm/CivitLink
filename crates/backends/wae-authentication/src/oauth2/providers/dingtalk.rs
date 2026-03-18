//! 钉钉 OAuth2 提供者配置

use crate::oauth2::OAuth2ProviderConfig;

/// 钉钉 OAuth2 端点
const DINGTALK_AUTH_URL: &str = "https://oapi.dingtalk.com/connect/oauth2/sns_authorize";
const DINGTALK_TOKEN_URL: &str = "https://oapi.dingtalk.com/sns/gettoken";
const DINGTALK_USERINFO_URL: &str = "https://oapi.dingtalk.com/sns/getuserinfo";

/// 创建钉钉 OAuth2 配置
///
/// # Arguments
/// * `client_id` - 钉钉 AppKey
/// * `client_secret` - 钉钉 AppSecret
/// * `redirect_uri` - 重定向 URI
pub fn dingtalk_config(
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    redirect_uri: impl Into<String>,
) -> OAuth2ProviderConfig {
    OAuth2ProviderConfig::new("dingtalk", client_id, client_secret, redirect_uri)
        .with_authorization_url(DINGTALK_AUTH_URL)
        .with_token_url(DINGTALK_TOKEN_URL)
        .with_userinfo_url(DINGTALK_USERINFO_URL)
        .with_scopes(vec!["snsapi_login".to_string()])
}

/// 钉钉 OAuth2 默认范围
pub fn dingtalk_default_scopes() -> Vec<String> {
    vec!["snsapi_login".to_string()]
}
