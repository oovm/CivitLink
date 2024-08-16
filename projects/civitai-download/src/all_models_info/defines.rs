use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllModels {
    pub metadata: Metadata,
    pub items: Vec<Item>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub poi: bool,
    pub nsfw: bool,
    pub allow_no_credit: bool,
    pub allow_commercial_use: String,
    pub allow_derivatives: bool,
    pub allow_different_license: bool,
    pub stats: Stats,
    pub creator: Creator,
    pub tags: Vec<String>,
    pub model_versions: Vec<ModelVersion>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub download_count: i64,
    pub favorite_count: i64,
    pub comment_count: i64,
    pub rating_count: i64,
    pub rating: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub username: String,
    pub image: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersion {
    pub id: i64,
    pub model_id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub trained_words: Vec<String>,
    pub base_model: String,
    pub early_access_time_frame: i64,
    pub description: Option<String>,
    pub files: Vec<File>,
    pub images: Vec<Image>,
    pub download_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub name: String,
    pub id: i64,
    #[serde(rename = "sizeKB")]
    pub size_kb: f64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
    pub pickle_scan_result: String,
    pub pickle_scan_message: String,
    pub virus_scan_result: String,
    pub scanned_at: String,
    pub hashes: Hashes,
    pub download_url: String,
    pub primary: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashes {
    #[serde(rename = "SHA256")]
    pub sha256: Option<String>,
    #[serde(rename = "AutoV1")]
    pub auto_v1: Option<String>,
    #[serde(rename = "AutoV2")]
    pub auto_v2: Option<String>,
    #[serde(rename = "BLAKE3")]
    pub blake3: Option<String>,
    #[serde(rename = "CRC32")]
    pub crc32: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub url: String,
    pub nsfw: bool,
    pub width: i64,
    pub height: i64,
    pub hash: String,
    pub meta: Option<Meta>,
    pub generation_process: Option<String>,
    pub needs_review: bool,
    pub tags: Vec<Tag>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "ENSD")]
    pub ensd: Option<String>,
    #[serde(rename = "Size")]
    pub size: Option<String>,
    pub seed: Option<i64>,
    #[serde(rename = "Model")]
    pub model: Option<String>,
    pub steps: Option<i64>,
    pub prompt: String,
    pub sampler: Option<String>,
    pub cfg_scale: Option<f64>,
    #[serde(default)]
    pub resources: Vec<Resource>,
    #[serde(rename = "Model hash")]
    pub model_hash: Option<String>,
    #[serde(rename = "Hires upscale")]
    pub hires_upscale: Option<String>,
    #[serde(rename = "Hires upscaler")]
    pub hires_upscaler: Option<String>,
    pub negative_prompt: Option<String>,
    #[serde(rename = "Denoising strength")]
    pub denoising_strength: Option<String>,
    #[serde(rename = "Hires steps")]
    pub hires_steps: Option<String>,
    #[serde(rename = "Negative Template")]
    pub negative_template: Option<String>,
    #[serde(rename = "Hires resize")]
    pub hires_resize: Option<String>,
    #[serde(rename = "First pass size")]
    pub first_pass_size: Option<String>,
    #[serde(rename = "Clip skip")]
    pub clip_skip: Option<String>,
    #[serde(rename = "Mask blur")]
    pub mask_blur: Option<String>,
    #[serde(rename = "Batch pos")]
    pub batch_pos: Option<String>,
    #[serde(rename = "Batch size")]
    pub batch_size: Option<String>,
    #[serde(rename = "AddNet Enabled")]
    pub add_net_enabled: Option<String>,
    #[serde(rename = "AddNet Model 1")]
    pub add_net_model_1: Option<String>,
    #[serde(rename = "AddNet Model 2")]
    pub add_net_model_2: Option<String>,
    #[serde(rename = "AddNet Module 1")]
    pub add_net_module_1: Option<String>,
    #[serde(rename = "AddNet Module 2")]
    pub add_net_module_2: Option<String>,
    #[serde(rename = "AddNet Weight A 1")]
    pub add_net_weight_a_1: Option<String>,
    #[serde(rename = "AddNet Weight A 2")]
    pub add_net_weight_a_2: Option<String>,
    #[serde(rename = "AddNet Weight B 1")]
    pub add_net_weight_b_1: Option<String>,
    #[serde(rename = "AddNet Weight B 2")]
    pub add_net_weight_b_2: Option<String>,
    #[serde(rename = "Conditional mask weight")]
    pub conditional_mask_weight: Option<String>,
    pub hashes: Option<Hashes2>,
    #[serde(rename = "Mimic scale")]
    pub mimic_scale: Option<String>,
    #[serde(rename = "Threshold percentile")]
    pub threshold_percentile: Option<String>,
    #[serde(rename = "Dynamic thresholding enabled")]
    pub dynamic_thresholding_enabled: Option<String>,
    #[serde(rename = "ControlNet Model")]
    pub control_net_model: Option<String>,
    #[serde(rename = "ControlNet Module")]
    pub control_net_module: Option<String>,
    #[serde(rename = "ControlNet Weight")]
    pub control_net_weight: Option<String>,
    #[serde(rename = "ControlNet Enabled")]
    pub control_net_enabled: Option<String>,
    #[serde(rename = "CFG mode")]
    pub cfg_mode: Option<String>,
    #[serde(rename = "Mimic mode")]
    pub mimic_mode: Option<String>,
    #[serde(rename = "CFG scale minimum")]
    pub cfg_scale_minimum: Option<String>,
    #[serde(rename = "Mimic scale minimum")]
    pub mimic_scale_minimum: Option<String>,
    #[serde(rename = "Eta")]
    pub eta: Option<String>,
    #[serde(rename = "SD upscale overlap")]
    pub sd_upscale_overlap: Option<String>,
    #[serde(rename = "SD upscale upscaler")]
    pub sd_upscale_upscaler: Option<String>,
    #[serde(rename = "Image CFG scale")]
    pub image_cfg_scale: Option<String>,
    #[serde(rename = "Face restoration")]
    pub face_restoration: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub hash: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub weight: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashes2 {
    pub vae: String,
    pub model: String,
    #[serde(rename = "lora:shuimobysimV3")]
    pub lora_shuimobysim_v3: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub tag: Tag2,
    pub automated: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag2 {
    pub id: i64,
    pub name: String,
    pub is_category: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub total_items: i64,
    pub current_page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub next_page: String,
}
