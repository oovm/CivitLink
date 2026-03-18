//! OAuth2 提供者预设配置

mod dingtalk;
mod feishu;
mod github;
mod google;
mod wechat;

pub use dingtalk::*;
pub use feishu::*;
pub use github::*;
pub use google::*;
pub use wechat::*;

use crate::oauth2::OAuth2ProviderConfig;

/// OAuth2 提供者类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuth2ProviderType {
    /// Google
    Google,
    /// GitHub
    GitHub,
    /// 微信
    WeChat,
    /// 钉钉
    DingTalk,
    /// 飞书
    Feishu,
    /// 自定义
    Custom,
}

/// 创建提供者配置
pub fn create_provider_config(
    provider_type: OAuth2ProviderType,
    client_id: impl Into<String> + Clone,
    client_secret: impl Into<String> + Clone,
    redirect_uri: impl Into<String> + Clone,
) -> OAuth2ProviderConfig {
    match provider_type {
        OAuth2ProviderType::Google => google_config(client_id, client_secret, redirect_uri),
        OAuth2ProviderType::GitHub => github_config(client_id, client_secret, redirect_uri),
        OAuth2ProviderType::WeChat => wechat_config(client_id, client_secret, redirect_uri),
        OAuth2ProviderType::DingTalk => dingtalk_config(client_id, client_secret, redirect_uri),
        OAuth2ProviderType::Feishu => feishu_config(client_id, client_secret, redirect_uri),
        OAuth2ProviderType::Custom => OAuth2ProviderConfig::new("custom", client_id, client_secret, redirect_uri),
    }
}
