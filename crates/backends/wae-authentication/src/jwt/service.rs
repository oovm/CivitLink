//! JWT 服务实现

use crate::jwt::{
    AccessTokenClaims, JwtAlgorithm, JwtClaims, JwtConfig, JwtHeader, RefreshTokenClaims, decode_jwt, encode_jwt,
};
use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use wae_types::WaeError;

/// JWT 结果类型
pub type JwtResult<T> = Result<T, WaeError>;

/// JWT 服务
#[derive(Debug, Clone)]
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    /// 创建新的 JWT 服务
    ///
    /// # Arguments
    /// * `config` - JWT 配置
    pub fn new(config: JwtConfig) -> JwtResult<Self> {
        Ok(Self { config })
    }

    /// 获取算法名称
    fn algorithm_name(&self) -> &'static str {
        match self.config.algorithm {
            JwtAlgorithm::HS256 => "HS256",
            JwtAlgorithm::HS384 => "HS384",
            JwtAlgorithm::HS512 => "HS512",
            _ => unimplemented!("Only HS256, HS384, HS512 are supported in custom implementation"),
        }
    }

    /// 生成令牌
    ///
    /// # Arguments
    /// * `claims` - JWT Claims
    pub fn generate_token<T: Serialize>(&self, claims: &T) -> JwtResult<String> {
        let header = JwtHeader::new(self.algorithm_name());
        let secret = self.config.secret.as_bytes();
        encode_jwt(&header, claims, secret).map_err(|e| e.into())
    }

    /// 验证令牌
    ///
    /// # Arguments
    /// * `token` - JWT 令牌
    pub fn verify_token<T: DeserializeOwned + 'static>(&self, token: &str) -> JwtResult<T> {
        let secret = self.config.secret.as_bytes();
        let claims: T = decode_jwt(token, secret, true)?;

        let now = Utc::now().timestamp();
        let leeway = self.config.leeway_seconds;

        if let Some(validated_claims) = (&claims as &dyn std::any::Any).downcast_ref::<JwtClaims>() {
            if validated_claims.exp + leeway < now {
                return Err(WaeError::token_expired());
            }
            if let Some(nbf) = validated_claims.nbf {
                if nbf - leeway > now {
                    return Err(WaeError::token_not_valid_yet());
                }
            }
            if self.config.validate_issuer {
                if let Some(ref issuer) = self.config.issuer {
                    if validated_claims.iss.as_ref() != Some(issuer) {
                        return Err(WaeError::invalid_claim("issuer"));
                    }
                }
            }
            if self.config.validate_audience {
                if let Some(ref audience) = self.config.audience {
                    if validated_claims.aud.as_ref() != Some(audience) {
                        return Err(WaeError::invalid_audience());
                    }
                }
            }
        }

        Ok(claims)
    }

    /// 生成访问令牌
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `claims` - 访问令牌 Claims
    pub fn generate_access_token(&self, user_id: &str, claims: AccessTokenClaims) -> JwtResult<String> {
        let mut jwt_claims = JwtClaims::new(user_id, self.config.access_token_expires_in());

        if let Some(ref issuer) = self.config.issuer {
            jwt_claims = jwt_claims.with_issuer(issuer.clone());
        }

        if let Some(ref audience) = self.config.audience {
            jwt_claims = jwt_claims.with_audience(audience.clone());
        }

        jwt_claims.custom.insert("user_id".to_string(), serde_json::to_value(&claims.user_id).unwrap());

        if let Some(ref username) = claims.username {
            jwt_claims.custom.insert("username".to_string(), serde_json::to_value(username).unwrap());
        }

        if !claims.roles.is_empty() {
            jwt_claims.custom.insert("roles".to_string(), serde_json::to_value(&claims.roles).unwrap());
        }

        if !claims.permissions.is_empty() {
            jwt_claims.custom.insert("permissions".to_string(), serde_json::to_value(&claims.permissions).unwrap());
        }

        if let Some(ref session_id) = claims.session_id {
            jwt_claims.custom.insert("session_id".to_string(), serde_json::to_value(session_id).unwrap());
        }

        self.generate_token(&jwt_claims)
    }

    /// 验证访问令牌
    ///
    /// # Arguments
    /// * `token` - 访问令牌
    pub fn verify_access_token(&self, token: &str) -> JwtResult<JwtClaims> {
        self.verify_token(token)
    }

    /// 生成刷新令牌
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `session_id` - 会话 ID
    /// * `version` - 令牌版本
    pub fn generate_refresh_token(&self, user_id: &str, session_id: &str, version: u32) -> JwtResult<String> {
        let mut jwt_claims = JwtClaims::new(user_id, self.config.refresh_token_expires_in());

        if let Some(ref issuer) = self.config.issuer {
            jwt_claims = jwt_claims.with_issuer(issuer.clone());
        }

        jwt_claims.custom.insert("session_id".to_string(), serde_json::to_value(session_id).unwrap());
        jwt_claims.custom.insert("version".to_string(), serde_json::to_value(version).unwrap());
        jwt_claims.custom.insert("token_type".to_string(), serde_json::to_value("refresh").unwrap());

        self.generate_token(&jwt_claims)
    }

    /// 验证刷新令牌
    ///
    /// # Arguments
    /// * `token` - 刷新令牌
    pub fn verify_refresh_token(&self, token: &str) -> JwtResult<RefreshTokenClaims> {
        let claims: JwtClaims = self.verify_token(token)?;

        let token_type: Option<String> = claims.custom.get("token_type").and_then(|v| serde_json::from_value(v.clone()).ok());

        if token_type.as_deref() != Some("refresh") {
            return Err(WaeError::invalid_token("not a refresh token"));
        }

        let session_id: String = claims
            .custom
            .get("session_id")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| WaeError::missing_claim("session_id"))?;

        let version: u32 = claims.custom.get("version").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or(0);

        Ok(RefreshTokenClaims { user_id: claims.sub, session_id, version })
    }

    /// 解码令牌（不验证签名）
    ///
    /// # Arguments
    /// * `token` - JWT 令牌
    pub fn decode_unchecked<T: DeserializeOwned>(&self, token: &str) -> JwtResult<T> {
        let secret = self.config.secret.as_bytes();
        decode_jwt(token, secret, false).map_err(|e| e.into())
    }

    /// 获取令牌剩余有效期
    ///
    /// # Arguments
    /// * `token` - JWT 令牌
    pub fn get_remaining_ttl(&self, token: &str) -> JwtResult<i64> {
        let claims: JwtClaims = self.verify_token(token)?;
        let now = Utc::now().timestamp();
        let remaining = claims.exp - now;
        Ok(remaining.max(0))
    }

    /// 检查令牌是否即将过期
    ///
    /// # Arguments
    /// * `token` - JWT 令牌
    /// * `threshold_seconds` - 阈值（秒）
    pub fn is_token_expiring_soon(&self, token: &str, threshold_seconds: i64) -> JwtResult<bool> {
        let remaining = self.get_remaining_ttl(token)?;
        Ok(remaining < threshold_seconds)
    }

    /// 获取配置
    pub fn config(&self) -> &JwtConfig {
        &self.config
    }
}

/// 令牌对（访问令牌 + 刷新令牌）
#[derive(Debug, Clone)]
pub struct TokenPair {
    /// 访问令牌
    pub access_token: String,

    /// 刷新令牌
    pub refresh_token: String,

    /// 令牌类型
    pub token_type: String,

    /// 访问令牌过期时间（秒）
    pub expires_in: i64,

    /// 刷新令牌过期时间（秒）
    pub refresh_expires_in: i64,
}

impl JwtService {
    /// 生成令牌对
    ///
    /// # Arguments
    /// * `user_id` - 用户 ID
    /// * `access_claims` - 访问令牌 Claims
    /// * `session_id` - 会话 ID
    pub fn generate_token_pair(
        &self,
        user_id: &str,
        access_claims: AccessTokenClaims,
        session_id: &str,
    ) -> JwtResult<TokenPair> {
        let access_token = self.generate_access_token(user_id, access_claims)?;
        let refresh_token = self.generate_refresh_token(user_id, session_id, 0)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_expires_in(),
            refresh_expires_in: self.config.refresh_token_expires_in(),
        })
    }

    /// 轮换刷新令牌
    ///
    /// 使用旧的刷新令牌生成新的令牌对，并递增版本号
    ///
    /// # Arguments
    /// * `refresh_token` - 旧的刷新令牌
    /// * `new_access_claims` - 新的访问令牌 Claims（可选）
    pub fn rotate_token_pair(&self, refresh_token: &str, new_access_claims: Option<AccessTokenClaims>) -> JwtResult<TokenPair> {
        let refresh_claims = self.verify_refresh_token(refresh_token)?;

        let new_version = refresh_claims.version + 1;
        let access_claims = new_access_claims.unwrap_or_else(|| {
            AccessTokenClaims::new(refresh_claims.user_id.clone()).with_session_id(refresh_claims.session_id.clone())
        });

        let access_token = self.generate_access_token(&refresh_claims.user_id, access_claims)?;
        let new_refresh_token =
            self.generate_refresh_token(&refresh_claims.user_id, &refresh_claims.session_id, new_version)?;

        Ok(TokenPair {
            access_token,
            refresh_token: new_refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.config.access_token_expires_in(),
            refresh_expires_in: self.config.refresh_token_expires_in(),
        })
    }
}

/// 便捷函数：创建默认 JWT 服务
pub fn default_jwt_service() -> JwtResult<JwtService> {
    JwtService::new(JwtConfig::default())
}

/// 便捷函数：使用密钥创建 JWT 服务
pub fn jwt_service_with_secret(secret: impl Into<String>) -> JwtResult<JwtService> {
    JwtService::new(JwtConfig::new(secret))
}
