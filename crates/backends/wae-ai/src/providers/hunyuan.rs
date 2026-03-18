use crate::{
    AiConfig, AiResult,
    capabilities::{
        ai_image::{AiImageCapability, AiImageInput, AiImageOutput, AiImageTask},
        chat::{ChatCapability, ChatParams},
        text_to_image::{TextToImageCapability, TextToImageParams},
        video::{AiVideoOutput, AiVideoTask, VideoGenerationCapability},
    },
};

use chrono::Utc;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::{Digest, Sha256};
use wae_request::HttpClient;
use wae_types::{BillingDimensions, WaeError, hex_encode};

/// 腾讯混元 AI 服务提供商
pub struct HunyuanProvider {
    http_client: HttpClient,
}

impl Default for HunyuanProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl HunyuanProvider {
    /// 创建新的混元提供商实例
    pub fn new() -> Self {
        Self { http_client: HttpClient::default() }
    }

    #[allow(clippy::too_many_arguments)]
    fn _generate_signature(
        secret_key: &str,
        service: &str,
        _host: &str,
        _region: &str,
        action: &str,
        _version: &str,
        payload: &str,
        timestamp: i64,
    ) -> String {
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let host = _host;

        let http_request_method = "POST";
        let canonical_uri = "/";
        let canonical_query_string = "";
        let canonical_headers =
            format!("content-type:application/json; charset=utf-8\nhost:{}\nx-tc-action:{}\n", host, action.to_lowercase());
        let signed_headers = "content-type;host;x-tc-action";

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let hashed_request_payload = hex_encode(&hasher.finalize());

        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            http_request_method,
            canonical_uri,
            canonical_query_string,
            canonical_headers,
            signed_headers,
            hashed_request_payload
        );

        let credential_scope = format!("{}/{}/{}/tc3_request", date, service, "tc3_request");
        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let hashed_canonical_request = hex_encode(&hasher.finalize());

        let string_to_sign = format!("TC3-HMAC-SHA256\n{}\n{}\n{}", timestamp, credential_scope, hashed_canonical_request);

        let k_date = _hmac_sha256(format!("TC3{}", secret_key).as_bytes(), date.as_bytes());
        let k_service = _hmac_sha256(&k_date, service.as_bytes());
        let k_signing = _hmac_sha256(&k_service, b"tc3_request");

        let signature = hex_encode(&_hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        format!(
            "TC3-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            "AKID...", credential_scope, signed_headers, signature
        )
    }
}

fn _hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC error");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

impl VideoGenerationCapability for HunyuanProvider {
    async fn generate_video(&self, _task: &AiVideoTask, _config: &AiConfig) -> AiResult<AiVideoOutput> {
        Err(WaeError::internal("Hunyuan video generation not implemented yet".to_string()))
    }

    async fn get_video_task_status(&self, _task_id: &str, _config: &AiConfig) -> AiResult<AiVideoOutput> {
        Err(WaeError::internal("Hunyuan video status check not implemented yet".to_string()))
    }
}

impl TextToImageCapability for HunyuanProvider {
    async fn text_to_image(&self, params: &TextToImageParams, config: &AiConfig) -> AiResult<Vec<String>> {
        let task = AiImageTask {
            inputs: vec![AiImageInput::Text(params.prompt.clone())],
            loras: vec![],
            width: params.width,
            height: params.height,
            num_images: params.num_images,
            negative_prompt: params.negative_prompt.clone(),
            seed: params.seed,
            extra: None,
        };

        let output = self.generate_image(&task, config).await?;
        Ok(output.images)
    }
}

impl AiImageCapability for HunyuanProvider {
    async fn generate_image(&self, task: &AiImageTask, _config: &AiConfig) -> AiResult<AiImageOutput> {
        let _client = &self.http_client;

        let prompt = task
            .inputs
            .iter()
            .find_map(|input| if let AiImageInput::Text(t) = input { Some(t.clone()) } else { None })
            .ok_or_else(|| WaeError::invalid_params("prompt", "Prompt is required"))?;

        let _body = json!({
            "Prompt": prompt,
            "NegativePrompt": task.negative_prompt,
            "Width": task.width.unwrap_or(1024),
            "Height": task.height.unwrap_or(1024),
            "ReqNum": task.num_images.unwrap_or(1),
            "Seed": task.seed,
        });

        Ok(AiImageOutput {
            images: vec![format!("https://mock-cos.com/{}.png", uuid::Uuid::new_v4())],
            text: None,
            usage: BillingDimensions {
                input_text: prompt.len() as u64,
                output_pixels: (task.width.unwrap_or(1024) * task.height.unwrap_or(1024) * task.num_images.unwrap_or(1)) as u64,
                ..Default::default()
            },
        })
    }
}

impl ChatCapability for HunyuanProvider {
    async fn chat(&self, _params: &ChatParams, _config: &AiConfig) -> AiResult<String> {
        Ok("Hunyuan Chat Response".to_string())
    }
}
