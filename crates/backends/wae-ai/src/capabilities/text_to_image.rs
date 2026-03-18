use crate::{AiConfig, AiResult};

use serde::{Deserialize, Serialize};

/// 文生图参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextToImageParams {
    /// 提示词
    pub prompt: String,
    /// 负向提示词
    pub negative_prompt: Option<String>,
    /// 宽度
    pub width: Option<u32>,
    /// 高度
    pub height: Option<u32>,
    /// 生成张数
    pub num_images: Option<u32>,
    /// 随机种子
    pub seed: Option<i64>,
}

/// 文生图能力 trait
pub trait TextToImageCapability: Send + Sync {
    /// 执行文生图
    #[allow(async_fn_in_trait)]
    async fn text_to_image(&self, params: &TextToImageParams, config: &AiConfig) -> AiResult<Vec<String>>;
}
