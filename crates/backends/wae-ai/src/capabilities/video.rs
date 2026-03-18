use crate::AiConfig;

use serde::{Deserialize, Serialize};
use wae_types::{BillingDimensions, WaeResult};

/// AI 视频输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiVideoInput {
    /// 文本输入
    Text(String),
    /// 图片输入 (URL 或 base64)
    Image(String),
}

/// AI 视频任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVideoTask {
    /// 输入序列
    pub inputs: Vec<AiVideoInput>,
    /// 时长 (秒)
    pub duration: Option<u32>,
    /// 宽度
    pub width: Option<u32>,
    /// 高度
    pub height: Option<u32>,
    /// 帧率
    pub fps: Option<u32>,
    /// 厂商特定的扩展参数
    pub extra: Option<serde_json::Value>,
}

/// AI 视频输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVideoOutput {
    /// 视频 URL
    pub video_url: String,
    /// 封面 URL
    pub cover_url: Option<String>,
    /// 计费维度
    pub usage: BillingDimensions,
}

/// 视频生成能力 trait
pub trait VideoGenerationCapability: Send + Sync {
    /// 生成视频
    #[allow(async_fn_in_trait)]
    async fn generate_video(&self, task: &AiVideoTask, config: &AiConfig) -> WaeResult<AiVideoOutput>;

    /// 查询视频任务状态 (某些视频生成是异步的)
    #[allow(async_fn_in_trait)]
    async fn get_video_task_status(&self, task_id: &str, config: &AiConfig) -> WaeResult<AiVideoOutput>;
}
