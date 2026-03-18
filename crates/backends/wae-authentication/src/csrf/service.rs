//! CSRF 服务实现

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::csrf::{CsrfConfig, CsrfError, CsrfResult};

/// CSRF 令牌记录
#[derive(Debug, Clone)]
struct CsrfTokenRecord {
    /// 令牌值（已哈希）
    token_hash: String,
    /// 过期时间戳
    expires_at: i64,
    /// 会话 ID
    session_id: String,
}

/// CSRF 令牌信息
#[derive(Debug, Clone)]
pub struct CsrfToken {
    /// 令牌值（用于传输）
    pub token: String,
    /// 过期时间戳
    pub expires_at: i64,
}

/// CSRF 服务
#[derive(Debug, Clone)]
pub struct CsrfService {
    config: CsrfConfig,
    tokens: Arc<RwLock<HashMap<String, CsrfTokenRecord>>>,
}

impl CsrfService {
    /// 创建新的 CSRF 服务
    ///
    /// # Arguments
    /// * `config` - CSRF 配置
    pub fn new(config: CsrfConfig) -> Self {
        Self { config, tokens: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// 使用默认配置创建 CSRF 服务
    pub fn default() -> Self {
        Self::new(CsrfConfig::default())
    }

    /// 生成 CSRF 令牌
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    pub async fn generate_token(&self, session_id: &str) -> CsrfResult<CsrfToken> {
        let token_bytes = uuid::Uuid::new_v4().into_bytes();
        let token = URL_SAFE_NO_PAD.encode(&token_bytes[..self.config.token_length.min(16)]);
        let token_hash = Self::hash_token(&token);
        let expires_at = Utc::now().timestamp() + self.config.token_ttl as i64;

        let record = CsrfTokenRecord { token_hash, expires_at, session_id: session_id.to_string() };

        self.tokens.write().await.insert(session_id.to_string(), record);

        self.cleanup_expired_tokens().await;

        Ok(CsrfToken { token, expires_at })
    }

    /// 验证 CSRF 令牌
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    /// * `token` - CSRF 令牌
    pub async fn validate_token(&self, session_id: &str, token: &str) -> CsrfResult<bool> {
        let tokens = self.tokens.read().await;

        let record = tokens.get(session_id).ok_or(CsrfError::InvalidToken)?;

        if record.expires_at < Utc::now().timestamp() {
            return Err(CsrfError::TokenExpired);
        }

        let token_hash = Self::hash_token(token);

        if record.token_hash != token_hash {
            return Err(CsrfError::InvalidToken);
        }

        Ok(true)
    }

    /// 撤销 CSRF 令牌
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    pub async fn revoke_token(&self, session_id: &str) {
        self.tokens.write().await.remove(session_id);
    }

    /// 哈希令牌（用于存储）
    fn hash_token(token: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }

    /// 清理过期令牌
    async fn cleanup_expired_tokens(&self) {
        let now = Utc::now().timestamp();
        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, record| record.expires_at > now);
    }

    /// 获取配置
    pub fn config(&self) -> &CsrfConfig {
        &self.config
    }
}

impl Default for CsrfService {
    fn default() -> Self {
        Self::new(CsrfConfig::default())
    }
}
