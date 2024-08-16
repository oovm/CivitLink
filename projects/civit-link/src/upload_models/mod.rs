use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::client::CivitClient;

impl CivitClient {
    pub fn model_upset() {}
}


#[derive(Serialize, Deserialize)]
struct ModelUpsetRequest {
    pub name: String,
    /// HTML
    pub description: String,
    #[serde(rename = "allowNoCredit")]
    pub allow_no_credit: bool,
    #[serde(rename = "allowCommercialUse")]
    pub allow_commercial_use: Vec<String>,
    #[serde(rename = "allowDerivatives")]
    pub allow_derivatives: bool,
    #[serde(rename = "allowDifferentLicense")]
    pub allow_different_license: bool,
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(rename = "uploadType")]
    pub upload_type: String,
    pub status: String,
    #[serde(rename = "checkpointType")]
    pub checkpoint_type: Option<i64>,
    pub poi: bool,
    pub nsfw: bool,
    pub minor: Option<i64>,
    #[serde(rename = "tagsOnModels")]
    pub tags_on_models: Vec<CivitTags>,
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "bountyId")]
    pub bounty_id: Option<i64>,
    pub authed: bool,
}

impl Default for ModelUpsetRequest {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            description: "".to_string(),
            allow_no_credit: false,
            allow_commercial_use: vec![],
            allow_derivatives: false,
            allow_different_license: false,
            r#type: "".to_string(),
            upload_type: "".to_string(),
            status: "".to_string(),
            checkpoint_type: None,
            poi: false,
            nsfw: false,
            minor: None,
            tags_on_models: vec![],
            template_id: 0,
            bounty_id: None,
            authed: true,
        }
    }
}

impl ModelUpsetRequest {
    pub fn send(&self, client: &CivitClient) -> reqwest::Result<Request> {
        let req = client.get("https://civitai.com/api/trpc/model.upsert", true)?;




    }
}


#[derive(Serialize, Deserialize)]
struct CivitTags {
    pub id: i64,
    pub name: String,
}


#[derive(Serialize, Deserialize)]
struct JsonMetaCompose<T> {
    pub json: T,
    pub meta: Value,
}

#[derive(Serialize, Deserialize)]
struct ModelUpsetResponse {
    pub id: i64,
    #[serde(rename = "nsfwLevel")]
    pub nsfw_level: i64,
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub json: ModelUpsetResponse,
}

#[derive(Serialize, Deserialize)]
struct Result {
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
struct ModelUpsetResponseResult {
    pub result: Result,
}

