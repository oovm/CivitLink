use crate::{AiConfig, AiResult};

use serde::{Deserialize, Serialize};
pub use wae_types::{BillingDimensions, Decimal};

/// 输入模态：文本或图片
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AiImageInput {
    /// 文本输入
    #[serde(rename = "text")]
    Text(String),
    /// 图片输入 (URL 或 base64)
    #[serde(rename = "image")]
    Image(String),
}

/// LoRA 模型配置 (通常作为扩展，不参与基础计费)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoRAConfig {
    /// 模型 ID
    pub model_id: String,
    /// 权重
    pub weight: f32,
}

/// 统一的生图任务描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiImageTask {
    /// 输入序列，支持文本、参考图以及对话上下文
    pub inputs: Vec<AiImageInput>,
    /// LoRA 配置
    pub loras: Vec<LoRAConfig>,
    /// 宽度
    pub width: Option<u32>,
    /// 高度
    pub height: Option<u32>,
    /// 生成张数
    pub num_images: Option<u32>,
    /// 负向提示词
    pub negative_prompt: Option<String>,
    /// 随机种子
    pub seed: Option<i64>,
    /// 厂商特定的扩展参数
    pub extra: Option<serde_json::Value>,
}

/// 生图任务输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiImageOutput {
    /// 生成的图片 URL 或 Base64 列表
    pub images: Vec<String>,
    /// 可能存在的说明性文字（如 DALL-E 3 的修改后提示词或对话回复）
    pub text: Option<String>,
    /// 实际消耗的计费维度
    pub usage: BillingDimensions,
}

/// 图像生成能力抽象
pub trait AiImageCapability: Send + Sync {
    /// 执行生图任务 (文生图、图生图、对话生图等)
    #[allow(async_fn_in_trait)]
    async fn generate_image(&self, task: &AiImageTask, config: &AiConfig) -> AiResult<AiImageOutput>;

    /// 预估计费维度 (某些厂商支持同步预估)
    fn estimate_usage(&self, task: &AiImageTask) -> AiResult<BillingDimensions> {
        let output_pixels =
            (task.width.unwrap_or(1024) as u64) * (task.height.unwrap_or(1024) as u64) * (task.num_images.unwrap_or(1) as u64);

        let mut input_text = 0;
        let mut input_pixels = 0;

        for input in &task.inputs {
            match input {
                AiImageInput::Text(t) => input_text += t.len() as u64,
                AiImageInput::Image(_) => input_pixels += 1024 * 1024,
            }
        }

        Ok(BillingDimensions { input_text, output_text: 0, input_pixels, output_pixels })
    }
}
