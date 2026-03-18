//! 中心化错误类型定义
//!
//! 提供统一的错误处理机制，支持国际化。

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fmt;

/// WAE 结果类型
pub type WaeResult<T> = Result<T, WaeError>;

/// 中心化错误类型
///
/// 包含错误类型标识，支持国际化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaeError {
    /// 错误类型
    pub kind: Box<WaeErrorKind>,
}

/// 错误分类
///
/// 用于将错误映射到 HTTP 状态码等场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 验证错误 (400)
    Validation,
    /// 认证错误 (401)
    Auth,
    /// 权限错误 (403)
    Permission,
    /// 资源未找到 (404)
    NotFound,
    /// 请求冲突 (409)
    Conflict,
    /// 请求过多 (429)
    RateLimited,
    /// 网络/服务错误 (502/503)
    Network,
    /// 存储错误 (500)
    Storage,
    /// 数据库错误 (500)
    Database,
    /// 缓存错误 (500)
    Cache,
    /// 配置错误 (500)
    Config,
    /// 超时错误 (408/504)
    Timeout,
    /// 内部错误 (500)
    Internal,
}

impl ErrorCategory {
    /// 获取对应的 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            ErrorCategory::Validation => 400,
            ErrorCategory::Auth => 401,
            ErrorCategory::Permission => 403,
            ErrorCategory::NotFound => 404,
            ErrorCategory::Conflict => 409,
            ErrorCategory::RateLimited => 429,
            ErrorCategory::Network => 502,
            ErrorCategory::Storage => 500,
            ErrorCategory::Database => 500,
            ErrorCategory::Cache => 500,
            ErrorCategory::Config => 500,
            ErrorCategory::Timeout => 408,
            ErrorCategory::Internal => 500,
        }
    }
}

/// 统一错误类型枚举
///
/// 整合所有模块的错误变体，每个变体对应一个 i18n key。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaeErrorKind {
    // ========== 验证错误 ==========
    /// 无效格式
    InvalidFormat {
        /// 字段名
        field: String,
        /// 期望格式
        expected: String,
    },
    /// 值超出范围
    OutOfRange {
        /// 字段名
        field: String,
        /// 最小值
        min: Option<String>,
        /// 最大值
        max: Option<String>,
    },
    /// 必填字段缺失
    Required {
        /// 字段名
        field: String,
    },
    /// 值已存在
    AlreadyExists {
        /// 字段名
        field: String,
        /// 值
        value: String,
    },
    /// 值不允许
    NotAllowed {
        /// 字段名
        field: String,
        /// 允许的值
        allowed: Vec<String>,
    },
    /// 无效参数
    InvalidParams {
        /// 参数名
        param: String,
        /// 原因
        reason: String,
    },

    // ========== 认证错误 (Auth) ==========
    /// 无效凭证
    InvalidCredentials,
    /// 账户已锁定
    AccountLocked,
    /// 用户未找到
    UserNotFound {
        /// 用户标识
        identifier: String,
    },
    /// 用户已存在
    UserAlreadyExists {
        /// 用户名
        username: String,
    },
    /// 无效令牌
    InvalidToken {
        /// 原因
        reason: String,
    },
    /// 令牌已过期
    TokenExpired,
    /// 令牌尚未生效
    TokenNotValidYet,
    /// 无效签名
    InvalidSignature,
    /// 无效算法
    InvalidAlgorithm,
    /// 缺少声明
    MissingClaim {
        /// 声明名
        claim: String,
    },
    /// 无效声明
    InvalidClaim {
        /// 声明名
        claim: String,
    },
    /// 无效签发者
    InvalidIssuer {
        /// 期望值
        expected: String,
        /// 实际值
        actual: String,
    },
    /// 无效受众
    InvalidAudience,
    /// 密钥错误
    KeyError,
    /// 编码错误
    EncodingError,
    /// 解码错误
    DecodingError,

    // ========== OAuth2 错误 ==========
    /// 无效授权码
    InvalidAuthorizationCode,
    /// 无效重定向 URI
    InvalidRedirectUri,
    /// 无效客户端 ID
    InvalidClientId,
    /// 无效客户端密钥
    InvalidClientSecret,
    /// 无效授权范围
    InvalidScope {
        /// 范围
        scope: String,
    },
    /// 无效访问令牌
    InvalidAccessToken,
    /// 无效刷新令牌
    InvalidRefreshToken,
    /// 访问被拒绝
    AccessDenied {
        /// 原因
        reason: String,
    },
    /// 不支持的授权类型
    UnsupportedGrantType {
        /// 类型
        grant_type: String,
    },
    /// 不支持的响应类型
    UnsupportedResponseType {
        /// 类型
        response_type: String,
    },
    /// 状态不匹配
    StateMismatch,
    /// OAuth2 提供者错误
    OAuth2ProviderError {
        /// 错误信息
        message: String,
    },

    // ========== SAML 错误 ==========
    /// 无效 SAML 请求
    InvalidSamlRequest {
        /// 原因
        reason: String,
    },
    /// 无效 SAML 响应
    InvalidSamlResponse {
        /// 原因
        reason: String,
    },
    /// 无效断言
    InvalidAssertion {
        /// 原因
        reason: String,
    },
    /// 签名验证失败
    SignatureVerificationFailed {
        /// 原因
        reason: String,
    },
    /// 缺少签名
    MissingSignature,
    /// 证书错误
    CertificateError {
        /// 原因
        reason: String,
    },
    /// XML 解析错误
    XmlParsingError {
        /// 原因
        reason: String,
    },
    /// Base64 解码错误
    Base64DecodeError {
        /// 原因
        reason: String,
    },
    /// 压缩错误
    CompressionError {
        /// 原因
        reason: String,
    },
    /// 断言已过期
    AssertionExpired,
    /// 断言尚未生效
    AssertionNotYetValid,
    /// 受众验证失败
    SamlAudienceValidationFailed {
        /// 期望值
        expected: String,
        /// 实际值
        actual: String,
    },
    /// 发行人验证失败
    SamlIssuerValidationFailed {
        /// 期望值
        expected: String,
        /// 实际值
        actual: String,
    },
    /// 目标验证失败
    DestinationValidationFailed,
    /// 重放攻击检测
    ReplayAttackDetected,
    /// 不支持的绑定
    UnsupportedBinding {
        /// 绑定类型
        binding: String,
    },
    /// 不支持的名称 ID 格式
    UnsupportedNameIdFormat {
        /// 格式
        format: String,
    },

    // ========== TOTP 错误 ==========
    /// 无效密钥
    InvalidSecret {
        /// 原因
        reason: String,
    },
    /// 无体验证码
    InvalidCode,
    /// 验证码已过期
    CodeExpired,
    /// 验证码格式错误
    InvalidCodeFormat {
        /// 期望位数
        expected: usize,
        /// 实际位数
        actual: usize,
    },
    /// 无效时间步长
    InvalidTimeStep,
    /// Base32 错误
    Base32Error {
        /// 原因
        reason: String,
    },
    /// HMAC 错误
    HmacError {
        /// 原因
        reason: String,
    },
    /// QR 码错误
    QrCodeError {
        /// 原因
        reason: String,
    },

    // ========== 权限错误 ==========
    /// 权限拒绝
    PermissionDenied {
        /// 操作
        action: String,
    },
    /// 禁止访问
    Forbidden {
        /// 资源
        resource: String,
    },

    // ========== 资源未找到 ==========
    /// 资源未找到
    ResourceNotFound {
        /// 资源类型
        resource_type: String,
        /// 资源标识
        identifier: String,
    },

    // ========== 冲突错误 ==========
    /// 资源冲突
    ResourceConflict {
        /// 资源
        resource: String,
        /// 原因
        reason: String,
    },

    // ========== 网络错误 ==========
    /// 连接失败
    ConnectionFailed {
        /// 目标
        target: String,
    },
    /// DNS 解析失败
    DnsResolutionFailed {
        /// 主机
        host: String,
    },
    /// TLS 错误
    TlsError {
        /// 原因
        reason: String,
    },
    /// 协议错误
    ProtocolError {
        /// 协议
        protocol: String,
        /// 原因
        reason: String,
    },
    /// 服务不可用
    ServiceUnavailable {
        /// 服务名
        service: String,
    },
    /// 网关错误
    GatewayError {
        /// 网关
        gateway: String,
    },

    // ========== 存储错误 ==========
    /// 读取失败
    StorageReadFailed {
        /// 路径
        path: String,
    },
    /// 写入失败
    StorageWriteFailed {
        /// 路径
        path: String,
    },
    /// 删除失败
    StorageDeleteFailed {
        /// 路径
        path: String,
    },
    /// 容量不足
    InsufficientCapacity {
        /// 需要
        required: u64,
        /// 可用
        available: u64,
    },
    /// 数据损坏
    DataCorruption {
        /// 位置
        location: String,
    },
    /// 文件未找到
    StorageFileNotFound {
        /// 路径
        path: String,
    },

    // ========== 数据库错误 ==========
    /// 数据库连接失败
    DatabaseConnectionFailed {
        /// 原因
        reason: String,
    },
    /// 查询失败
    QueryFailed {
        /// 查询语句
        query: Option<String>,
        /// 原因
        reason: String,
    },
    /// 执行失败
    ExecuteFailed {
        /// 语句
        statement: Option<String>,
        /// 原因
        reason: String,
    },
    /// 事务失败
    TransactionFailed {
        /// 原因
        reason: String,
    },
    /// 迁移失败
    MigrationFailed {
        /// 版本
        version: Option<String>,
        /// 原因
        reason: String,
    },

    // ========== 缓存错误 ==========
    /// 缓存连接失败
    CacheConnectionFailed {
        /// 原因
        reason: String,
    },
    /// 缓存键不存在
    CacheKeyNotFound {
        /// 键
        key: String,
    },
    /// 序列化失败
    SerializationFailed {
        /// 类型
        type_name: String,
    },
    /// 反序列化失败
    DeserializationFailed {
        /// 类型
        type_name: String,
    },

    // ========== 弹性容错错误 ==========
    /// 熔断器开启
    CircuitBreakerOpen {
        /// 服务名
        service: String,
    },
    /// 限流超出
    RateLimitExceeded {
        /// 限制
        limit: u64,
    },
    /// 重试次数耗尽
    MaxRetriesExceeded {
        /// 次数
        attempts: u32,
    },
    /// 舱壁已满
    BulkheadFull {
        /// 最大并发
        max_concurrent: usize,
    },
    /// 舱壁等待超时
    BulkheadWaitTimeout,

    // ========== 调度器错误 ==========
    /// 任务未找到
    TaskNotFound {
        /// 任务 ID
        task_id: String,
    },
    /// 任务已存在
    TaskAlreadyExists {
        /// 任务 ID
        task_id: String,
    },
    /// 无效 Cron 表达式
    InvalidCronExpression {
        /// 表达式
        expression: String,
    },
    /// 任务执行失败
    TaskExecutionFailed {
        /// 任务 ID
        task_id: String,
        /// 原因
        reason: String,
    },
    /// 调度器已关闭
    SchedulerShutdown,

    // ========== 测试错误 ==========
    /// Mock 错误
    MockError {
        /// 原因
        reason: String,
    },
    /// 断言失败
    AssertionFailed {
        /// 消息
        message: String,
    },
    /// Fixture 错误
    FixtureError {
        /// 原因
        reason: String,
    },
    /// 环境错误
    EnvironmentError {
        /// 原因
        reason: String,
    },

    // ========== 超时错误 ==========
    /// 操作超时
    OperationTimeout {
        /// 操作名
        operation: String,
        /// 超时时间（毫秒）
        timeout_ms: u64,
    },

    // ========== 配置错误 ==========
    /// 配置缺失
    ConfigMissing {
        /// 配置键
        key: String,
    },
    /// 配置无效
    ConfigInvalid {
        /// 配置键
        key: String,
        /// 原因
        reason: String,
    },

    // ========== 内部错误 ==========
    /// 内部错误
    InternalError {
        /// 原因
        reason: String,
    },
    /// 未实现
    NotImplemented {
        /// 功能
        feature: String,
    },
    /// IO 错误
    IoError {
        /// 操作
        operation: String,
        /// 原因
        reason: String,
    },
    /// JSON 错误
    JsonError {
        /// 原因
        reason: String,
    },
    /// 解析错误
    ParseError {
        /// 类型
        type_name: String,
        /// 原因
        reason: String,
    },
    /// 请求错误
    RequestError {
        /// URL
        url: String,
        /// 原因
        reason: String,
    },
}

impl WaeErrorKind {
    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            WaeErrorKind::InvalidFormat { .. }
            | WaeErrorKind::OutOfRange { .. }
            | WaeErrorKind::Required { .. }
            | WaeErrorKind::AlreadyExists { .. }
            | WaeErrorKind::NotAllowed { .. }
            | WaeErrorKind::InvalidParams { .. } => ErrorCategory::Validation,

            WaeErrorKind::InvalidCredentials
            | WaeErrorKind::AccountLocked
            | WaeErrorKind::UserNotFound { .. }
            | WaeErrorKind::UserAlreadyExists { .. }
            | WaeErrorKind::InvalidToken { .. }
            | WaeErrorKind::TokenExpired
            | WaeErrorKind::TokenNotValidYet
            | WaeErrorKind::InvalidSignature
            | WaeErrorKind::InvalidAlgorithm
            | WaeErrorKind::MissingClaim { .. }
            | WaeErrorKind::InvalidClaim { .. }
            | WaeErrorKind::InvalidIssuer { .. }
            | WaeErrorKind::InvalidAudience
            | WaeErrorKind::KeyError
            | WaeErrorKind::EncodingError
            | WaeErrorKind::DecodingError
            | WaeErrorKind::InvalidAuthorizationCode
            | WaeErrorKind::InvalidRedirectUri
            | WaeErrorKind::InvalidClientId
            | WaeErrorKind::InvalidClientSecret
            | WaeErrorKind::InvalidScope { .. }
            | WaeErrorKind::InvalidAccessToken
            | WaeErrorKind::InvalidRefreshToken
            | WaeErrorKind::AccessDenied { .. }
            | WaeErrorKind::UnsupportedGrantType { .. }
            | WaeErrorKind::UnsupportedResponseType { .. }
            | WaeErrorKind::StateMismatch
            | WaeErrorKind::OAuth2ProviderError { .. }
            | WaeErrorKind::InvalidSamlRequest { .. }
            | WaeErrorKind::InvalidSamlResponse { .. }
            | WaeErrorKind::InvalidAssertion { .. }
            | WaeErrorKind::SignatureVerificationFailed { .. }
            | WaeErrorKind::MissingSignature
            | WaeErrorKind::CertificateError { .. }
            | WaeErrorKind::XmlParsingError { .. }
            | WaeErrorKind::Base64DecodeError { .. }
            | WaeErrorKind::CompressionError { .. }
            | WaeErrorKind::AssertionExpired
            | WaeErrorKind::AssertionNotYetValid
            | WaeErrorKind::SamlAudienceValidationFailed { .. }
            | WaeErrorKind::SamlIssuerValidationFailed { .. }
            | WaeErrorKind::DestinationValidationFailed
            | WaeErrorKind::ReplayAttackDetected
            | WaeErrorKind::UnsupportedBinding { .. }
            | WaeErrorKind::UnsupportedNameIdFormat { .. }
            | WaeErrorKind::InvalidSecret { .. }
            | WaeErrorKind::InvalidCode
            | WaeErrorKind::CodeExpired
            | WaeErrorKind::InvalidCodeFormat { .. }
            | WaeErrorKind::InvalidTimeStep
            | WaeErrorKind::Base32Error { .. }
            | WaeErrorKind::HmacError { .. }
            | WaeErrorKind::QrCodeError { .. } => ErrorCategory::Auth,

            WaeErrorKind::PermissionDenied { .. } | WaeErrorKind::Forbidden { .. } => ErrorCategory::Permission,

            WaeErrorKind::ResourceNotFound { .. } => ErrorCategory::NotFound,

            WaeErrorKind::ResourceConflict { .. } => ErrorCategory::Conflict,

            WaeErrorKind::RateLimitExceeded { .. } => ErrorCategory::RateLimited,

            WaeErrorKind::ConnectionFailed { .. }
            | WaeErrorKind::DnsResolutionFailed { .. }
            | WaeErrorKind::TlsError { .. }
            | WaeErrorKind::ProtocolError { .. }
            | WaeErrorKind::ServiceUnavailable { .. }
            | WaeErrorKind::GatewayError { .. } => ErrorCategory::Network,

            WaeErrorKind::StorageReadFailed { .. }
            | WaeErrorKind::StorageWriteFailed { .. }
            | WaeErrorKind::StorageDeleteFailed { .. }
            | WaeErrorKind::InsufficientCapacity { .. }
            | WaeErrorKind::DataCorruption { .. }
            | WaeErrorKind::StorageFileNotFound { .. } => ErrorCategory::Storage,

            WaeErrorKind::DatabaseConnectionFailed { .. }
            | WaeErrorKind::QueryFailed { .. }
            | WaeErrorKind::ExecuteFailed { .. }
            | WaeErrorKind::TransactionFailed { .. }
            | WaeErrorKind::MigrationFailed { .. } => ErrorCategory::Database,

            WaeErrorKind::CacheConnectionFailed { .. }
            | WaeErrorKind::CacheKeyNotFound { .. }
            | WaeErrorKind::SerializationFailed { .. }
            | WaeErrorKind::DeserializationFailed { .. } => ErrorCategory::Cache,

            WaeErrorKind::CircuitBreakerOpen { .. } | WaeErrorKind::BulkheadFull { .. } | WaeErrorKind::BulkheadWaitTimeout => {
                ErrorCategory::Network
            }

            WaeErrorKind::MaxRetriesExceeded { .. } => ErrorCategory::Network,

            WaeErrorKind::TaskNotFound { .. }
            | WaeErrorKind::TaskAlreadyExists { .. }
            | WaeErrorKind::InvalidCronExpression { .. }
            | WaeErrorKind::TaskExecutionFailed { .. }
            | WaeErrorKind::SchedulerShutdown => ErrorCategory::Internal,

            WaeErrorKind::MockError { .. }
            | WaeErrorKind::AssertionFailed { .. }
            | WaeErrorKind::FixtureError { .. }
            | WaeErrorKind::EnvironmentError { .. } => ErrorCategory::Internal,

            WaeErrorKind::OperationTimeout { .. } => ErrorCategory::Timeout,

            WaeErrorKind::ConfigMissing { .. } | WaeErrorKind::ConfigInvalid { .. } => ErrorCategory::Config,

            WaeErrorKind::InternalError { .. }
            | WaeErrorKind::NotImplemented { .. }
            | WaeErrorKind::IoError { .. }
            | WaeErrorKind::JsonError { .. }
            | WaeErrorKind::ParseError { .. }
            | WaeErrorKind::RequestError { .. } => ErrorCategory::Internal,
        }
    }

    /// 获取国际化键
    pub fn i18n_key(&self) -> &'static str {
        match self {
            WaeErrorKind::InvalidFormat { .. } => "wae.error.validation.invalid_format",
            WaeErrorKind::OutOfRange { .. } => "wae.error.validation.out_of_range",
            WaeErrorKind::Required { .. } => "wae.error.validation.required",
            WaeErrorKind::AlreadyExists { .. } => "wae.error.validation.already_exists",
            WaeErrorKind::NotAllowed { .. } => "wae.error.validation.not_allowed",
            WaeErrorKind::InvalidParams { .. } => "wae.error.validation.invalid_params",

            WaeErrorKind::InvalidCredentials => "wae.error.auth.invalid_credentials",
            WaeErrorKind::AccountLocked => "wae.error.auth.account_locked",
            WaeErrorKind::UserNotFound { .. } => "wae.error.auth.user_not_found",
            WaeErrorKind::UserAlreadyExists { .. } => "wae.error.auth.user_already_exists",
            WaeErrorKind::InvalidToken { .. } => "wae.error.auth.invalid_token",
            WaeErrorKind::TokenExpired => "wae.error.auth.token_expired",
            WaeErrorKind::TokenNotValidYet => "wae.error.auth.token_not_valid_yet",
            WaeErrorKind::InvalidSignature => "wae.error.auth.invalid_signature",
            WaeErrorKind::InvalidAlgorithm => "wae.error.auth.invalid_algorithm",
            WaeErrorKind::MissingClaim { .. } => "wae.error.auth.missing_claim",
            WaeErrorKind::InvalidClaim { .. } => "wae.error.auth.invalid_claim",
            WaeErrorKind::InvalidIssuer { .. } => "wae.error.auth.invalid_issuer",
            WaeErrorKind::InvalidAudience => "wae.error.auth.invalid_audience",
            WaeErrorKind::KeyError => "wae.error.auth.key_error",
            WaeErrorKind::EncodingError => "wae.error.auth.encoding_error",
            WaeErrorKind::DecodingError => "wae.error.auth.decoding_error",

            WaeErrorKind::InvalidAuthorizationCode => "wae.error.oauth2.invalid_authorization_code",
            WaeErrorKind::InvalidRedirectUri => "wae.error.oauth2.invalid_redirect_uri",
            WaeErrorKind::InvalidClientId => "wae.error.oauth2.invalid_client_id",
            WaeErrorKind::InvalidClientSecret => "wae.error.oauth2.invalid_client_secret",
            WaeErrorKind::InvalidScope { .. } => "wae.error.oauth2.invalid_scope",
            WaeErrorKind::InvalidAccessToken => "wae.error.oauth2.invalid_access_token",
            WaeErrorKind::InvalidRefreshToken => "wae.error.oauth2.invalid_refresh_token",
            WaeErrorKind::AccessDenied { .. } => "wae.error.oauth2.access_denied",
            WaeErrorKind::UnsupportedGrantType { .. } => "wae.error.oauth2.unsupported_grant_type",
            WaeErrorKind::UnsupportedResponseType { .. } => "wae.error.oauth2.unsupported_response_type",
            WaeErrorKind::StateMismatch => "wae.error.oauth2.state_mismatch",
            WaeErrorKind::OAuth2ProviderError { .. } => "wae.error.oauth2.provider_error",

            WaeErrorKind::InvalidSamlRequest { .. } => "wae.error.saml.invalid_request",
            WaeErrorKind::InvalidSamlResponse { .. } => "wae.error.saml.invalid_response",
            WaeErrorKind::InvalidAssertion { .. } => "wae.error.saml.invalid_assertion",
            WaeErrorKind::SignatureVerificationFailed { .. } => "wae.error.saml.signature_verification_failed",
            WaeErrorKind::MissingSignature => "wae.error.saml.missing_signature",
            WaeErrorKind::CertificateError { .. } => "wae.error.saml.certificate_error",
            WaeErrorKind::XmlParsingError { .. } => "wae.error.saml.xml_parsing_error",
            WaeErrorKind::Base64DecodeError { .. } => "wae.error.saml.base64_decode_error",
            WaeErrorKind::CompressionError { .. } => "wae.error.saml.compression_error",
            WaeErrorKind::AssertionExpired => "wae.error.saml.assertion_expired",
            WaeErrorKind::AssertionNotYetValid => "wae.error.saml.assertion_not_yet_valid",
            WaeErrorKind::SamlAudienceValidationFailed { .. } => "wae.error.saml.audience_validation_failed",
            WaeErrorKind::SamlIssuerValidationFailed { .. } => "wae.error.saml.issuer_validation_failed",
            WaeErrorKind::DestinationValidationFailed => "wae.error.saml.destination_validation_failed",
            WaeErrorKind::ReplayAttackDetected => "wae.error.saml.replay_attack_detected",
            WaeErrorKind::UnsupportedBinding { .. } => "wae.error.saml.unsupported_binding",
            WaeErrorKind::UnsupportedNameIdFormat { .. } => "wae.error.saml.unsupported_name_id_format",

            WaeErrorKind::InvalidSecret { .. } => "wae.error.totp.invalid_secret",
            WaeErrorKind::InvalidCode => "wae.error.totp.invalid_code",
            WaeErrorKind::CodeExpired => "wae.error.totp.code_expired",
            WaeErrorKind::InvalidCodeFormat { .. } => "wae.error.totp.invalid_code_format",
            WaeErrorKind::InvalidTimeStep => "wae.error.totp.invalid_time_step",
            WaeErrorKind::Base32Error { .. } => "wae.error.totp.base32_error",
            WaeErrorKind::HmacError { .. } => "wae.error.totp.hmac_error",
            WaeErrorKind::QrCodeError { .. } => "wae.error.totp.qr_code_error",

            WaeErrorKind::PermissionDenied { .. } => "wae.error.permission.denied",
            WaeErrorKind::Forbidden { .. } => "wae.error.permission.forbidden",

            WaeErrorKind::ResourceNotFound { .. } => "wae.error.not_found.resource",

            WaeErrorKind::ResourceConflict { .. } => "wae.error.conflict.resource",

            WaeErrorKind::RateLimitExceeded { .. } => "wae.error.rate_limited",

            WaeErrorKind::ConnectionFailed { .. } => "wae.error.network.connection_failed",
            WaeErrorKind::DnsResolutionFailed { .. } => "wae.error.network.dns_resolution_failed",
            WaeErrorKind::TlsError { .. } => "wae.error.network.tls_error",
            WaeErrorKind::ProtocolError { .. } => "wae.error.network.protocol_error",
            WaeErrorKind::ServiceUnavailable { .. } => "wae.error.network.service_unavailable",
            WaeErrorKind::GatewayError { .. } => "wae.error.network.gateway_error",

            WaeErrorKind::StorageReadFailed { .. } => "wae.error.storage.read_failed",
            WaeErrorKind::StorageWriteFailed { .. } => "wae.error.storage.write_failed",
            WaeErrorKind::StorageDeleteFailed { .. } => "wae.error.storage.delete_failed",
            WaeErrorKind::InsufficientCapacity { .. } => "wae.error.storage.insufficient_capacity",
            WaeErrorKind::DataCorruption { .. } => "wae.error.storage.data_corruption",
            WaeErrorKind::StorageFileNotFound { .. } => "wae.error.storage.file_not_found",

            WaeErrorKind::DatabaseConnectionFailed { .. } => "wae.error.database.connection_failed",
            WaeErrorKind::QueryFailed { .. } => "wae.error.database.query_failed",
            WaeErrorKind::ExecuteFailed { .. } => "wae.error.database.execute_failed",
            WaeErrorKind::TransactionFailed { .. } => "wae.error.database.transaction_failed",
            WaeErrorKind::MigrationFailed { .. } => "wae.error.database.migration_failed",

            WaeErrorKind::CacheConnectionFailed { .. } => "wae.error.cache.connection_failed",
            WaeErrorKind::CacheKeyNotFound { .. } => "wae.error.cache.key_not_found",
            WaeErrorKind::SerializationFailed { .. } => "wae.error.cache.serialization_failed",
            WaeErrorKind::DeserializationFailed { .. } => "wae.error.cache.deserialization_failed",

            WaeErrorKind::CircuitBreakerOpen { .. } => "wae.error.resilience.circuit_breaker_open",
            WaeErrorKind::MaxRetriesExceeded { .. } => "wae.error.resilience.max_retries_exceeded",
            WaeErrorKind::BulkheadFull { .. } => "wae.error.resilience.bulkhead_full",
            WaeErrorKind::BulkheadWaitTimeout => "wae.error.resilience.bulkhead_wait_timeout",

            WaeErrorKind::TaskNotFound { .. } => "wae.error.scheduler.task_not_found",
            WaeErrorKind::TaskAlreadyExists { .. } => "wae.error.scheduler.task_already_exists",
            WaeErrorKind::InvalidCronExpression { .. } => "wae.error.scheduler.invalid_cron_expression",
            WaeErrorKind::TaskExecutionFailed { .. } => "wae.error.scheduler.task_execution_failed",
            WaeErrorKind::SchedulerShutdown => "wae.error.scheduler.shutdown",

            WaeErrorKind::MockError { .. } => "wae.error.testing.mock_error",
            WaeErrorKind::AssertionFailed { .. } => "wae.error.testing.assertion_failed",
            WaeErrorKind::FixtureError { .. } => "wae.error.testing.fixture_error",
            WaeErrorKind::EnvironmentError { .. } => "wae.error.testing.environment_error",

            WaeErrorKind::OperationTimeout { .. } => "wae.error.timeout.operation",

            WaeErrorKind::ConfigMissing { .. } => "wae.error.config.missing",
            WaeErrorKind::ConfigInvalid { .. } => "wae.error.config.invalid",

            WaeErrorKind::InternalError { .. } => "wae.error.internal.error",
            WaeErrorKind::NotImplemented { .. } => "wae.error.internal.not_implemented",
            WaeErrorKind::IoError { .. } => "wae.error.internal.io_error",
            WaeErrorKind::JsonError { .. } => "wae.error.internal.json_error",
            WaeErrorKind::ParseError { .. } => "wae.error.internal.parse_error",
            WaeErrorKind::RequestError { .. } => "wae.error.internal.request_error",
        }
    }

    /// 获取国际化数据
    pub fn i18n_data(&self) -> Value {
        match self {
            WaeErrorKind::InvalidFormat { field, expected } => {
                json!({ "field": field, "expected": expected })
            }
            WaeErrorKind::OutOfRange { field, min, max } => {
                json!({ "field": field, "min": min, "max": max })
            }
            WaeErrorKind::Required { field } => json!({ "field": field }),
            WaeErrorKind::AlreadyExists { field, value } => json!({ "field": field, "value": value }),
            WaeErrorKind::NotAllowed { field, allowed } => json!({ "field": field, "allowed": allowed }),
            WaeErrorKind::InvalidParams { param, reason } => json!({ "param": param, "reason": reason }),

            WaeErrorKind::InvalidCredentials => json!({}),
            WaeErrorKind::AccountLocked => json!({}),
            WaeErrorKind::UserNotFound { identifier } => json!({ "identifier": identifier }),
            WaeErrorKind::UserAlreadyExists { username } => json!({ "username": username }),
            WaeErrorKind::InvalidToken { reason } => json!({ "reason": reason }),
            WaeErrorKind::TokenExpired => json!({}),
            WaeErrorKind::TokenNotValidYet => json!({}),
            WaeErrorKind::InvalidSignature => json!({}),
            WaeErrorKind::InvalidAlgorithm => json!({}),
            WaeErrorKind::MissingClaim { claim } => json!({ "claim": claim }),
            WaeErrorKind::InvalidClaim { claim } => json!({ "claim": claim }),
            WaeErrorKind::InvalidIssuer { expected, actual } => json!({ "expected": expected, "actual": actual }),
            WaeErrorKind::InvalidAudience => json!({}),
            WaeErrorKind::KeyError => json!({}),
            WaeErrorKind::EncodingError => json!({}),
            WaeErrorKind::DecodingError => json!({}),

            WaeErrorKind::InvalidAuthorizationCode => json!({}),
            WaeErrorKind::InvalidRedirectUri => json!({}),
            WaeErrorKind::InvalidClientId => json!({}),
            WaeErrorKind::InvalidClientSecret => json!({}),
            WaeErrorKind::InvalidScope { scope } => json!({ "scope": scope }),
            WaeErrorKind::InvalidAccessToken => json!({}),
            WaeErrorKind::InvalidRefreshToken => json!({}),
            WaeErrorKind::AccessDenied { reason } => json!({ "reason": reason }),
            WaeErrorKind::UnsupportedGrantType { grant_type } => json!({ "grant_type": grant_type }),
            WaeErrorKind::UnsupportedResponseType { response_type } => json!({ "response_type": response_type }),
            WaeErrorKind::StateMismatch => json!({}),
            WaeErrorKind::OAuth2ProviderError { message } => json!({ "message": message }),

            WaeErrorKind::InvalidSamlRequest { reason } => json!({ "reason": reason }),
            WaeErrorKind::InvalidSamlResponse { reason } => json!({ "reason": reason }),
            WaeErrorKind::InvalidAssertion { reason } => json!({ "reason": reason }),
            WaeErrorKind::SignatureVerificationFailed { reason } => json!({ "reason": reason }),
            WaeErrorKind::MissingSignature => json!({}),
            WaeErrorKind::CertificateError { reason } => json!({ "reason": reason }),
            WaeErrorKind::XmlParsingError { reason } => json!({ "reason": reason }),
            WaeErrorKind::Base64DecodeError { reason } => json!({ "reason": reason }),
            WaeErrorKind::CompressionError { reason } => json!({ "reason": reason }),
            WaeErrorKind::AssertionExpired => json!({}),
            WaeErrorKind::AssertionNotYetValid => json!({}),
            WaeErrorKind::SamlAudienceValidationFailed { expected, actual } => {
                json!({ "expected": expected, "actual": actual })
            }
            WaeErrorKind::SamlIssuerValidationFailed { expected, actual } => {
                json!({ "expected": expected, "actual": actual })
            }
            WaeErrorKind::DestinationValidationFailed => json!({}),
            WaeErrorKind::ReplayAttackDetected => json!({}),
            WaeErrorKind::UnsupportedBinding { binding } => json!({ "binding": binding }),
            WaeErrorKind::UnsupportedNameIdFormat { format } => json!({ "format": format }),

            WaeErrorKind::InvalidSecret { reason } => json!({ "reason": reason }),
            WaeErrorKind::InvalidCode => json!({}),
            WaeErrorKind::CodeExpired => json!({}),
            WaeErrorKind::InvalidCodeFormat { expected, actual } => json!({ "expected": expected, "actual": actual }),
            WaeErrorKind::InvalidTimeStep => json!({}),
            WaeErrorKind::Base32Error { reason } => json!({ "reason": reason }),
            WaeErrorKind::HmacError { reason } => json!({ "reason": reason }),
            WaeErrorKind::QrCodeError { reason } => json!({ "reason": reason }),

            WaeErrorKind::PermissionDenied { action } => json!({ "action": action }),
            WaeErrorKind::Forbidden { resource } => json!({ "resource": resource }),

            WaeErrorKind::ResourceNotFound { resource_type, identifier } => {
                json!({ "resource_type": resource_type, "identifier": identifier })
            }

            WaeErrorKind::ResourceConflict { resource, reason } => json!({ "resource": resource, "reason": reason }),

            WaeErrorKind::RateLimitExceeded { limit } => json!({ "limit": limit }),

            WaeErrorKind::ConnectionFailed { target } => json!({ "target": target }),
            WaeErrorKind::DnsResolutionFailed { host } => json!({ "host": host }),
            WaeErrorKind::TlsError { reason } => json!({ "reason": reason }),
            WaeErrorKind::ProtocolError { protocol, reason } => json!({ "protocol": protocol, "reason": reason }),
            WaeErrorKind::ServiceUnavailable { service } => json!({ "service": service }),
            WaeErrorKind::GatewayError { gateway } => json!({ "gateway": gateway }),

            WaeErrorKind::StorageReadFailed { path } => json!({ "path": path }),
            WaeErrorKind::StorageWriteFailed { path } => json!({ "path": path }),
            WaeErrorKind::StorageDeleteFailed { path } => json!({ "path": path }),
            WaeErrorKind::InsufficientCapacity { required, available } => {
                json!({ "required": required, "available": available })
            }
            WaeErrorKind::DataCorruption { location } => json!({ "location": location }),
            WaeErrorKind::StorageFileNotFound { path } => json!({ "path": path }),

            WaeErrorKind::DatabaseConnectionFailed { reason } => json!({ "reason": reason }),
            WaeErrorKind::QueryFailed { query, reason } => json!({ "query": query, "reason": reason }),
            WaeErrorKind::ExecuteFailed { statement, reason } => json!({ "statement": statement, "reason": reason }),
            WaeErrorKind::TransactionFailed { reason } => json!({ "reason": reason }),
            WaeErrorKind::MigrationFailed { version, reason } => json!({ "version": version, "reason": reason }),

            WaeErrorKind::CacheConnectionFailed { reason } => json!({ "reason": reason }),
            WaeErrorKind::CacheKeyNotFound { key } => json!({ "key": key }),
            WaeErrorKind::SerializationFailed { type_name } => json!({ "type": type_name }),
            WaeErrorKind::DeserializationFailed { type_name } => json!({ "type": type_name }),

            WaeErrorKind::CircuitBreakerOpen { service } => json!({ "service": service }),
            WaeErrorKind::MaxRetriesExceeded { attempts } => json!({ "attempts": attempts }),
            WaeErrorKind::BulkheadFull { max_concurrent } => json!({ "max_concurrent": max_concurrent }),
            WaeErrorKind::BulkheadWaitTimeout => json!({}),

            WaeErrorKind::TaskNotFound { task_id } => json!({ "task_id": task_id }),
            WaeErrorKind::TaskAlreadyExists { task_id } => json!({ "task_id": task_id }),
            WaeErrorKind::InvalidCronExpression { expression } => json!({ "expression": expression }),
            WaeErrorKind::TaskExecutionFailed { task_id, reason } => json!({ "task_id": task_id, "reason": reason }),
            WaeErrorKind::SchedulerShutdown => json!({}),

            WaeErrorKind::MockError { reason } => json!({ "reason": reason }),
            WaeErrorKind::AssertionFailed { message } => json!({ "message": message }),
            WaeErrorKind::FixtureError { reason } => json!({ "reason": reason }),
            WaeErrorKind::EnvironmentError { reason } => json!({ "reason": reason }),

            WaeErrorKind::OperationTimeout { operation, timeout_ms } => {
                json!({ "operation": operation, "timeout_ms": timeout_ms })
            }

            WaeErrorKind::ConfigMissing { key } => json!({ "key": key }),
            WaeErrorKind::ConfigInvalid { key, reason } => json!({ "key": key, "reason": reason }),

            WaeErrorKind::InternalError { reason } => json!({ "reason": reason }),
            WaeErrorKind::NotImplemented { feature } => json!({ "feature": feature }),
            WaeErrorKind::IoError { operation, reason } => json!({ "operation": operation, "reason": reason }),
            WaeErrorKind::JsonError { reason } => json!({ "reason": reason }),
            WaeErrorKind::ParseError { type_name, reason } => json!({ "type": type_name, "reason": reason }),
            WaeErrorKind::RequestError { url, reason } => json!({ "url": url, "reason": reason }),
        }
    }
}

impl fmt::Display for WaeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.i18n_key())
    }
}

impl WaeError {
    /// 创建新的错误
    pub fn new(kind: WaeErrorKind) -> Self {
        Self { kind: Box::new(kind) }
    }

    /// 获取国际化键
    pub fn i18n_key(&self) -> &'static str {
        self.kind.i18n_key()
    }

    /// 获取国际化数据
    pub fn i18n_data(&self) -> Value {
        self.kind.i18n_data()
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        self.kind.category()
    }

    /// 获取 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        self.category().http_status()
    }

    // ========== 验证错误便捷方法 ==========

    /// 创建无效格式错误
    pub fn invalid_format(field: impl Into<String>, expected: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InvalidFormat { field: field.into(), expected: expected.into() })
    }

    /// 创建超出范围错误
    pub fn out_of_range(field: impl Into<String>, min: Option<String>, max: Option<String>) -> Self {
        Self::new(WaeErrorKind::OutOfRange { field: field.into(), min, max })
    }

    /// 创建必填字段缺失错误
    pub fn required(field: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::Required { field: field.into() })
    }

    /// 创建已存在错误
    pub fn already_exists(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::AlreadyExists { field: field.into(), value: value.into() })
    }

    /// 创建不允许错误
    pub fn not_allowed(field: impl Into<String>, allowed: Vec<String>) -> Self {
        Self::new(WaeErrorKind::NotAllowed { field: field.into(), allowed })
    }

    /// 创建无效参数错误
    pub fn invalid_params(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InvalidParams { param: param.into(), reason: reason.into() })
    }

    // ========== 认证错误便捷方法 ==========

    /// 创建无效凭证错误
    pub fn invalid_credentials() -> Self {
        Self::new(WaeErrorKind::InvalidCredentials)
    }

    /// 创建账户已锁定错误
    pub fn account_locked() -> Self {
        Self::new(WaeErrorKind::AccountLocked)
    }

    /// 创建用户未找到错误
    pub fn user_not_found(identifier: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::UserNotFound { identifier: identifier.into() })
    }

    /// 创建用户已存在错误
    pub fn user_already_exists(username: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::UserAlreadyExists { username: username.into() })
    }

    /// 创建无效令牌错误
    pub fn invalid_token(reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InvalidToken { reason: reason.into() })
    }

    /// 创建令牌过期错误
    pub fn token_expired() -> Self {
        Self::new(WaeErrorKind::TokenExpired)
    }

    /// 创建令牌尚未生效错误
    pub fn token_not_valid_yet() -> Self {
        Self::new(WaeErrorKind::TokenNotValidYet)
    }

    /// 创建无效签名错误
    pub fn invalid_signature() -> Self {
        Self::new(WaeErrorKind::InvalidSignature)
    }

    /// 创建缺少声明错误
    pub fn missing_claim(claim: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::MissingClaim { claim: claim.into() })
    }

    /// 创建无效声明错误
    pub fn invalid_claim(claim: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InvalidClaim { claim: claim.into() })
    }

    /// 创建无效签发者错误
    pub fn invalid_issuer(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InvalidIssuer { expected: expected.into(), actual: actual.into() })
    }

    /// 创建无效受众错误
    pub fn invalid_audience() -> Self {
        Self::new(WaeErrorKind::InvalidAudience)
    }

    // ========== 权限错误便捷方法 ==========

    /// 创建权限拒绝错误
    pub fn permission_denied(action: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::PermissionDenied { action: action.into() })
    }

    /// 创建禁止访问错误
    pub fn forbidden(resource: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::Forbidden { resource: resource.into() })
    }

    // ========== 资源未找到便捷方法 ==========

    /// 创建资源未找到错误
    pub fn not_found(resource_type: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ResourceNotFound { resource_type: resource_type.into(), identifier: identifier.into() })
    }

    // ========== 网络错误便捷方法 ==========

    /// 创建连接失败错误
    pub fn connection_failed(target: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ConnectionFailed { target: target.into() })
    }

    /// 创建服务不可用错误
    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ServiceUnavailable { service: service.into() })
    }

    // ========== 存储错误便捷方法 ==========

    /// 创建存储读取失败错误
    pub fn storage_read_failed(path: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::StorageReadFailed { path: path.into() })
    }

    /// 创建存储写入失败错误
    pub fn storage_write_failed(path: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::StorageWriteFailed { path: path.into() })
    }

    /// 创建存储文件未找到错误
    pub fn storage_file_not_found(path: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::StorageFileNotFound { path: path.into() })
    }

    // ========== 数据库错误便捷方法 ==========

    /// 创建数据库错误
    pub fn database(kind: WaeErrorKind) -> Self {
        Self::new(kind)
    }

    /// 创建数据库连接失败错误
    pub fn database_connection_failed(reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::DatabaseConnectionFailed { reason: reason.into() })
    }

    /// 创建查询失败错误
    pub fn query_failed(query: Option<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::QueryFailed { query, reason: reason.into() })
    }

    /// 创建执行失败错误
    pub fn execute_failed(statement: Option<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ExecuteFailed { statement, reason: reason.into() })
    }

    /// 创建事务失败错误
    pub fn transaction_failed(reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::TransactionFailed { reason: reason.into() })
    }

    // ========== 缓存错误便捷方法 ==========

    /// 创建缓存键未找到错误
    pub fn cache_key_not_found(key: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::CacheKeyNotFound { key: key.into() })
    }

    /// 创建序列化失败错误
    pub fn serialization_failed(type_name: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::SerializationFailed { type_name: type_name.into() })
    }

    /// 创建反序列化失败错误
    pub fn deserialization_failed(type_name: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::DeserializationFailed { type_name: type_name.into() })
    }

    // ========== 弹性容错错误便捷方法 ==========

    /// 创建熔断器开启错误
    pub fn circuit_breaker_open(service: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::CircuitBreakerOpen { service: service.into() })
    }

    /// 创建限流超出错误
    pub fn rate_limit_exceeded(limit: u64) -> Self {
        Self::new(WaeErrorKind::RateLimitExceeded { limit })
    }

    /// 创建重试次数耗尽错误
    pub fn max_retries_exceeded(attempts: u32) -> Self {
        Self::new(WaeErrorKind::MaxRetriesExceeded { attempts })
    }

    /// 创建舱壁已满错误
    pub fn bulkhead_full(max_concurrent: usize) -> Self {
        Self::new(WaeErrorKind::BulkheadFull { max_concurrent })
    }

    // ========== 调度器错误便捷方法 ==========

    /// 创建任务未找到错误
    pub fn task_not_found(task_id: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::TaskNotFound { task_id: task_id.into() })
    }

    /// 创建无效 Cron 表达式错误
    pub fn invalid_cron_expression(expression: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InvalidCronExpression { expression: expression.into() })
    }

    /// 创建任务执行失败错误
    pub fn task_execution_failed(task_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::TaskExecutionFailed { task_id: task_id.into(), reason: reason.into() })
    }

    /// 创建调度器已关闭错误
    pub fn scheduler_shutdown() -> Self {
        Self::new(WaeErrorKind::SchedulerShutdown)
    }

    // ========== 超时错误便捷方法 ==========

    /// 创建操作超时错误
    pub fn operation_timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::new(WaeErrorKind::OperationTimeout { operation: operation.into(), timeout_ms })
    }

    // ========== 配置错误便捷方法 ==========

    /// 创建配置缺失错误
    pub fn config_missing(key: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ConfigMissing { key: key.into() })
    }

    /// 创建配置无效错误
    pub fn config_invalid(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ConfigInvalid { key: key.into(), reason: reason.into() })
    }

    // ========== 内部错误便捷方法 ==========

    /// 创建内部错误
    pub fn internal(reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::InternalError { reason: reason.into() })
    }

    /// 创建未实现错误
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::NotImplemented { feature: feature.into() })
    }

    /// 创建 IO 错误
    pub fn io_error(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::IoError { operation: operation.into(), reason: reason.into() })
    }

    /// 创建 JSON 错误
    pub fn json_error(reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::JsonError { reason: reason.into() })
    }

    /// 创建解析错误
    pub fn parse_error(type_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::ParseError { type_name: type_name.into(), reason: reason.into() })
    }

    /// 创建请求错误
    pub fn request_error(url: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(WaeErrorKind::RequestError { url: url.into(), reason: reason.into() })
    }
}

impl fmt::Display for WaeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind.category(), self.kind.i18n_key())
    }
}

impl std::error::Error for WaeError {}

impl From<std::io::Error> for WaeError {
    fn from(err: std::io::Error) -> Self {
        Self::io_error("unknown", err.to_string())
    }
}

impl From<serde_json::Error> for WaeError {
    fn from(err: serde_json::Error) -> Self {
        Self::json_error(err.to_string())
    }
}
