//! 微信 OAuth2 提供者配置

use crate::oauth2::OAuth2ProviderConfig;

/// 微信 OAuth2 端点
const WECHAT_AUTH_URL: &str = "https://open.weixin.qq.com/connect/qrconnect";
const WECHAT_TOKEN_URL: &str = "https://api.weixin.qq.com/sns/oauth2/access_token";
const WECHAT_USERINFO_URL: &str = "https://api.weixin.qq.com/sns/userinfo";
const WECHAT_REFRESH_TOKEN_URL: &str = "https://api.weixin.qq.com/sns/oauth2/refresh_token";

/// 创建微信 OAuth2 配置
///
/// # Arguments
/// * `client_id` - 微信 AppID
/// * `client_secret` - 微信 AppSecret
/// * `redirect_uri` - 重定向 URI
pub fn wechat_config(
    client_id: impl Into<String> + Clone,
    client_secret: impl Into<String>,
    redirect_uri: impl Into<String>,
) -> OAuth2ProviderConfig {
    OAuth2ProviderConfig::new("wechat", client_id.clone(), client_secret, redirect_uri)
        .with_authorization_url(WECHAT_AUTH_URL)
        .with_token_url(WECHAT_TOKEN_URL)
        .with_userinfo_url(WECHAT_USERINFO_URL)
        .with_scopes(vec!["snsapi_login".to_string()])
        .add_extra_param("appid", client_id.into())
}

/// 创建微信网页授权配置 (用于微信内网页)
///
/// # Arguments
/// * `client_id` - 微信 AppID
/// * `client_secret` - 微信 AppSecret
/// * `redirect_uri` - 重定向 URI
pub fn wechat_web_config(
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    redirect_uri: impl Into<String>,
) -> OAuth2ProviderConfig {
    OAuth2ProviderConfig::new("wechat_web", client_id, client_secret, redirect_uri)
        .with_authorization_url("https://open.weixin.qq.com/connect/oauth2/authorize")
        .with_token_url(WECHAT_TOKEN_URL)
        .with_userinfo_url(WECHAT_USERINFO_URL)
        .with_scopes(vec!["snsapi_userinfo".to_string()])
}

/// 微信 OAuth2 默认范围 (扫码登录)
pub fn wechat_default_scopes() -> Vec<String> {
    vec!["snsapi_login".to_string()]
}

/// 微信 OAuth2 静默授权范围
pub fn wechat_snsapi_base_scopes() -> Vec<String> {
    vec!["snsapi_base".to_string()]
}

/// 微信 OAuth2 用户信息授权范围
pub fn wechat_snsapi_userinfo_scopes() -> Vec<String> {
    vec!["snsapi_userinfo".to_string()]
}
