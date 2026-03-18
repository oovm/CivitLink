//! WAE AI - AI 服务抽象层
//!
//! 提供统一的 AI 能力抽象，支持多种 AI 服务商。
//!
//! 深度融合 tokio 运行时，所有 API 都是异步优先设计。
//! 微服务架构友好，支持服务发现、链路追踪等特性。
#![warn(missing_docs)]

/// AI 能力模块
pub mod capabilities;
/// AI 服务提供商模块
pub mod providers;

pub use capabilities::{
    ai_image::{AiImageCapability, AiImageInput, AiImageOutput, AiImageTask, LoRAConfig},
    chat::{ChatCapability, ChatMessage},
    text_to_image::{TextToImageCapability, TextToImageParams},
};

use serde::{Deserialize, Serialize};
pub use wae_types::{WaeError, WaeResult};

/// AI 操作结果类型
pub type AiResult<T> = WaeResult<T>;
/// AI 错误类型
pub type AiError = WaeError;

/// AI 服务配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiConfig {
    /// 密钥 ID
    pub secret_id: String,
    /// 密钥
    pub secret_key: String,
    /// 服务端点
    pub endpoint: Option<String>,
    /// 区域
    pub region: Option<String>,
}
