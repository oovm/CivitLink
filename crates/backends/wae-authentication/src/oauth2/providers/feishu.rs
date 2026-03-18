//! 飞书 OAuth2 提供者配置

use crate::oauth2::OAuth2ProviderConfig;

/// 飞书 OAuth2 端点
const FEISHU_AUTH_URL: &str = "https://open.feishu.cn/open-apis/authen/v1/index";
const FEISHU_TOKEN_URL: &str = "https://open.feishu.cn/open-apis/auth/v3/app_access_token/internal";
const FEISHU_USERINFO_URL: &str = "https://open.feishu.cn/open-apis/authen/v1/user_info";

/// 创建飞书 OAuth2 配置
///
/// # Arguments
/// * `client_id` - 飞书 App ID
/// * `client_secret` - 飞书 App Secret
/// * `redirect_uri` - 重定向 URI
pub fn feishu_config(
    client_id: impl Into<String>,
    client_secret: impl Into<String>,
    redirect_uri: impl Into<String>,
) -> OAuth2ProviderConfig {
    OAuth2ProviderConfig::new("feishu", client_id, client_secret, redirect_uri)
        .with_authorization_url(FEISHU_AUTH_URL)
        .with_token_url(FEISHU_TOKEN_URL)
        .with_userinfo_url(FEISHU_USERINFO_URL)
        .with_scopes(vec!["user:profile".to_string()])
}

/// 飞书 OAuth2 默认范围
pub fn feishu_default_scopes() -> Vec<String> {
    vec!["user:profile".to_string()]
}
