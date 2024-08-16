use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
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
    pub rank: Rank,
    pub creator: Creator,
    pub tags: Vec<String>,
    pub model_versions: Vec<ModelVersion>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rank {
    pub download_count_all_time: i64,
    pub comment_count_all_time: i64,
    pub favorite_count_all_time: i64,
    pub rating_count_all_time: i64,
    pub rating_all_time: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub username: String,
    pub image: Value,
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
    pub description: Value,
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
    pub size_kb: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub format: String,
    pub pickle_scan_result: String,
    pub pickle_scan_message: String,
    pub virus_scan_result: String,
    pub scanned_at: String,
    pub hashes: Hashes,
    pub download_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hashes {
    #[serde(rename = "AutoV1")]
    pub auto_v1: String,
    #[serde(rename = "AutoV2")]
    pub auto_v2: String,
    #[serde(rename = "SHA256")]
    pub sha256: String,
    #[serde(rename = "CRC32")]
    pub crc32: String,
    #[serde(rename = "BLAKE3")]
    pub blake3: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub url: String,
    pub nsfw: bool,
    pub width: i64,
    pub height: i64,
    pub hash: String,
    pub meta: Value,
    pub generation_process: Value,
    pub needs_review: bool,
    pub tags: Vec<Tag>,
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
