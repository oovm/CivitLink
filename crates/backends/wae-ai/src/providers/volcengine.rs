use crate::{
    AiConfig, AiResult,
    capabilities::{
        ai_image::{AiImageCapability, AiImageInput, AiImageOutput, AiImageTask},
        chat::{ChatCapability, ChatParams},
        text_to_image::{TextToImageCapability, TextToImageParams},
        video::{AiVideoOutput, AiVideoTask, VideoGenerationCapability},
    },
};

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use wae_request::HttpClient;
use wae_types::{BillingDimensions, WaeError, hex_encode};

/// 火山引擎服务提供商
///
/// 注意：火山引擎不同服务使用不同的认证方式：
/// 1. 豆包 (Ark/Chat): 使用单一 API Key，对应 AiConfig.secret_id，无需 secret_key。
/// 2. 视觉服务 (CV/Image): 使用 Access Key ID (secret_id) 和 Secret Access Key (secret_key)。
pub struct VolcengineProvider {
    http_client: HttpClient,
}

impl Default for VolcengineProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl VolcengineProvider {
    /// 创建新的火山引擎提供商实例
    pub fn new() -> Self {
        Self { http_client: HttpClient::default() }
    }

    /// 使用自定义 HTTP 客户端创建
    pub fn with_client(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// 实现火山引擎 V4 签名算法
    #[allow(clippy::too_many_arguments)]
    fn generate_v4_signature(
        &self,
        config: &AiConfig,
        service: &str,
        method: &str,
        host: &str,
        path: &str,
        query: &str,
        payload: &[u8],
        now: DateTime<Utc>,
    ) -> String {
        let date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_only = now.format("%Y%m%d").to_string();
        let region = config.region.as_deref().unwrap_or("cn-beijing");

        let payload_hash = hex_encode(&Sha256::digest(payload));
        let canonical_headers =
            format!("content-type:application/json\nhost:{}\nx-content-sha256:{}\nx-date:{}\n", host, payload_hash, date);
        let signed_headers = "content-type;host;x-content-sha256;x-date";
        let canonical_request =
            format!("{}\n{}\n{}\n{}\n{}\n{}", method, path, query, canonical_headers, signed_headers, payload_hash);

        let credential_scope = format!("{}/{}/{}/request", date_only, region, service);
        let string_to_sign = format!(
            "HMAC-SHA256\n{}\n{}\n{}",
            date,
            credential_scope,
            hex_encode(&Sha256::digest(canonical_request.as_bytes()))
        );

        let k_date = self.hmac_sha256(config.secret_key.as_bytes(), date_only.as_bytes());
        let k_region = self.hmac_sha256(&k_date, region.as_bytes());
        let k_service = self.hmac_sha256(&k_region, service.as_bytes());
        let k_signing = self.hmac_sha256(&k_service, b"request");

        let signature = hex_encode(&self.hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        format!(
            "HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            config.secret_id, credential_scope, "content-type;host;x-content-sha256;x-date", signature
        )
    }

    fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }
}

impl ChatCapability for VolcengineProvider {
    async fn chat(&self, params: &ChatParams, config: &AiConfig) -> AiResult<String> {
        let url = config.endpoint.as_deref().unwrap_or("https://ark.cn-beijing.volces.com/api/v3/chat/completions");

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", config.secret_id));
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = serde_json::json!({
            "model": config.endpoint.as_deref().unwrap_or("doubao-pro-4k"),
            "messages": params.messages,
            "stream": false
        });

        let response = self
            .http_client
            .post_with_headers(url, serde_json::to_vec(&body).unwrap(), headers)
            .await
            .map_err(|e| WaeError::internal(e.to_string()))?;

        if !response.is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(WaeError::internal(format!("Volcengine Doubao API error: HTTP {} - {}", response.status, error_text)));
        }

        let result: serde_json::Value =
            response.json().map_err(|e| WaeError::internal(format!("Failed to parse response: {}", e)))?;

        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| WaeError::internal("Failed to parse Doubao response".to_string()))?;

        Ok(content.to_string())
    }
}

impl AiImageCapability for VolcengineProvider {
    async fn generate_image(&self, task: &AiImageTask, config: &AiConfig) -> AiResult<AiImageOutput> {
        let prompt = task
            .inputs
            .iter()
            .filter_map(|input| if let AiImageInput::Text(t) = input { Some(t.clone()) } else { None })
            .collect::<Vec<_>>()
            .join(", ");

        let now = Utc::now();
        let service = "cv";
        let method = "POST";
        let path = "/";
        let query = "Action=CVProcess&Version=2022-08-31";

        let payload = serde_json::json!({
            "req_key": "high_flux",
            "prompt": prompt,
            "width": task.width.unwrap_or(1024),
            "height": task.height.unwrap_or(1024),
            "image_num": task.num_images.unwrap_or(1),
        });
        let payload_bytes = serde_json::to_vec(&payload).unwrap();

        let host = "visual.volcengineapi.com";
        let signature = self.generate_v4_signature(config, service, method, host, path, query, &payload_bytes, now);

        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), signature);
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Date".to_string(), now.format("%Y%m%dT%H%M%SZ").to_string());
        headers.insert("X-Content-Sha256".to_string(), hex_encode(&Sha256::digest(&payload_bytes)));

        let url = format!("https://visual.volcengineapi.com{}?{}", path, query);
        let response = self
            .http_client
            .post_with_headers(&url, payload_bytes, headers)
            .await
            .map_err(|e| WaeError::internal(e.to_string()))?;

        if !response.is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(WaeError::internal(format!("Volcengine Image V3 error: HTTP {} - {}", response.status, error_text)));
        }

        let result: serde_json::Value =
            response.json().map_err(|e| WaeError::internal(format!("Failed to parse response: {}", e)))?;

        let mut image_urls = Vec::new();

        if let Some(arr) = result["data"]["binary_data_base64"].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    image_urls.push(format!("data:image/png;base64,{}", s));
                }
            }
        }

        if image_urls.is_empty()
            && let Some(arr) = result["data"]["image_urls"].as_array()
        {
            for v in arr {
                if let Some(s) = v.as_str() {
                    image_urls.push(s.to_string());
                }
            }
        }

        if image_urls.is_empty() {
            return Err(WaeError::internal(format!("Volcengine Image V3 returned no images. Response: {}", result)));
        }

        Ok(AiImageOutput {
            images: image_urls,
            text: None,
            usage: BillingDimensions {
                input_text: 0,
                output_text: 0,
                input_pixels: 0,
                output_pixels: (task.width.unwrap_or(1024) * task.height.unwrap_or(1024) * task.num_images.unwrap_or(1)) as u64,
            },
        })
    }
}

impl VideoGenerationCapability for VolcengineProvider {
    async fn generate_video(&self, _task: &AiVideoTask, _config: &AiConfig) -> AiResult<AiVideoOutput> {
        Err(WaeError::internal("Volcengine video generation not implemented yet".to_string()))
    }

    async fn get_video_task_status(&self, _task_id: &str, _config: &AiConfig) -> AiResult<AiVideoOutput> {
        Err(WaeError::internal("Volcengine video status check not implemented yet".to_string()))
    }
}

impl TextToImageCapability for VolcengineProvider {
    async fn text_to_image(&self, params: &TextToImageParams, config: &AiConfig) -> AiResult<Vec<String>> {
        let task = AiImageTask {
            inputs: vec![AiImageInput::Text(params.prompt.clone())],
            width: params.width,
            height: params.height,
            num_images: params.num_images,
            negative_prompt: params.negative_prompt.clone(),
            seed: params.seed,
            loras: vec![],
            extra: None,
        };
        let output = self.generate_image(&task, config).await?;
        Ok(output.images)
    }
}
