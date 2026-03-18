//! OAuth2 配置模块

use std::collections::HashSet;

/// OAuth2 提供者配置
#[derive(Debug, Clone)]
pub struct OAuth2ProviderConfig {
    /// 提供者名称
    pub name: String,

    /// 客户端 ID
    pub client_id: String,

    /// 客户端密钥
    pub client_secret: String,

    /// 授权端点 URL
    pub authorization_url: String,

    /// 令牌端点 URL
    pub token_url: String,

    /// 用户信息端点 URL
    pub userinfo_url: Option<String>,

    /// 撤销端点 URL
    pub revocation_url: Option<String>,

    /// 重定向 URI
    pub redirect_uri: String,

    /// 授权范围
    pub scopes: Vec<String>,

    /// 额外参数
    pub extra_params: Vec<(String, String)>,
}

impl OAuth2ProviderConfig {
    /// 创建新的提供者配置
    pub fn new(
        name: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            authorization_url: String::new(),
            token_url: String::new(),
            userinfo_url: None,
            revocation_url: None,
            redirect_uri: redirect_uri.into(),
            scopes: Vec::new(),
            extra_params: Vec::new(),
        }
    }

    /// 设置授权端点 URL
    pub fn with_authorization_url(mut self, url: impl Into<String>) -> Self {
        self.authorization_url = url.into();
        self
    }

    /// 设置令牌端点 URL
    pub fn with_token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = url.into();
        self
    }

    /// 设置用户信息端点 URL
    pub fn with_userinfo_url(mut self, url: impl Into<String>) -> Self {
        self.userinfo_url = Some(url.into());
        self
    }

    /// 设置撤销端点 URL
    pub fn with_revocation_url(mut self, url: impl Into<String>) -> Self {
        self.revocation_url = Some(url.into());
        self
    }

    /// 设置授权范围
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// 添加授权范围
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// 添加额外参数
    pub fn add_extra_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_params.push((key.into(), value.into()));
        self
    }
}

/// OAuth2 客户端配置
#[derive(Debug, Clone)]
pub struct OAuth2ClientConfig {
    /// 提供者配置
    pub provider: OAuth2ProviderConfig,

    /// 是否启用 PKCE
    pub use_pkce: bool,

    /// 是否启用状态验证
    pub use_state: bool,

    /// 是否验证 scope
    pub validate_scope: bool,

    /// 允许的 scope 集合
    pub allowed_scopes: HashSet<String>,

    /// 请求超时（毫秒）
    pub timeout_ms: u64,
}

impl OAuth2ClientConfig {
    /// 创建新的客户端配置
    pub fn new(provider: OAuth2ProviderConfig) -> Self {
        Self {
            provider,
            use_pkce: true,
            use_state: true,
            validate_scope: false,
            allowed_scopes: HashSet::new(),
            timeout_ms: 30000,
        }
    }

    /// 设置是否启用 PKCE
    pub fn with_pkce(mut self, use_pkce: bool) -> Self {
        self.use_pkce = use_pkce;
        self
    }

    /// 设置是否启用状态验证
    pub fn with_state_validation(mut self, use_state: bool) -> Self {
        self.use_state = use_state;
        self
    }

    /// 设置是否验证 scope
    pub fn with_scope_validation(mut self, validate: bool) -> Self {
        self.validate_scope = validate;
        self
    }

    /// 设置允许的 scope
    pub fn with_allowed_scopes(mut self, scopes: HashSet<String>) -> Self {
        self.allowed_scopes = scopes;
        self
    }

    /// 设置请求超时
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// OAuth2 授权请求配置
#[derive(Debug, Clone)]
pub struct AuthorizationRequest {
    /// 重定向 URI
    pub redirect_uri: String,

    /// 授权范围
    pub scopes: Vec<String>,

    /// 状态参数
    pub state: Option<String>,

    /// PKCE code challenge
    pub code_challenge: Option<String>,

    /// PKCE code challenge method
    pub code_challenge_method: Option<String>,

    /// 额外参数
    pub extra_params: Vec<(String, String)>,
}

impl AuthorizationRequest {
    /// 创建新的授权请求
    pub fn new(redirect_uri: impl Into<String>) -> Self {
        Self {
            redirect_uri: redirect_uri.into(),
            scopes: Vec::new(),
            state: None,
            code_challenge: None,
            code_challenge_method: None,
            extra_params: Vec::new(),
        }
    }

    /// 设置授权范围
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// 设置状态参数
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// 设置 PKCE challenge
    pub fn with_pkce(mut self, challenge: impl Into<String>, method: impl Into<String>) -> Self {
        self.code_challenge = Some(challenge.into());
        self.code_challenge_method = Some(method.into());
        self
    }

    /// 添加额外参数
    pub fn add_extra_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_params.push((key.into(), value.into()));
        self
    }
}
