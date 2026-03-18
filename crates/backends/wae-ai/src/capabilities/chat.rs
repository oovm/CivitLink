use crate::{AiConfig, AiResult};

use serde::{Deserialize, Serialize};

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色
    pub role: String,
    /// 内容
    pub content: String,
}

/// 聊天参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatParams {
    /// 消息列表
    pub messages: Vec<ChatMessage>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
}

/// 聊天能力 trait
pub trait ChatCapability: Send + Sync {
    /// 执行聊天
    #[allow(async_fn_in_trait)]
    async fn chat(&self, params: &ChatParams, config: &AiConfig) -> AiResult<String>;
}
