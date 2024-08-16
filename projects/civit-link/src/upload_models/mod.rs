use serde::{Deserialize, Serialize};
use crate::client::CivitClient;

impl CivitClient {
    pub fn model_upset() {

    }
}

#[derive(Serialize, Deserialize)]
struct Values {
    pub minor: Vec<String>,
    #[serde(rename = "bountyId")]
    pub bounty_id: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    pub values: Values,
}

#[derive(Serialize, Deserialize)]
struct Struct {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
struct ModelUpsetRequest {
    #[serde(rename = "allowNoCredit")]
    pub allow_no_credit: bool,
    #[serde(rename = "allowCommercialUse")]
    pub allow_commercial_use: Vec<String>,
    #[serde(rename = "allowDerivatives")]
    pub allow_derivatives: bool,
    #[serde(rename = "allowDifferentLicense")]
    pub allow_different_license: bool,
    pub name: String,
    pub description: String,
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
    pub tags_on_models: Vec<Struct>,
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(rename = "bountyId")]
    pub bounty_id: Option<i64>,
    pub authed: bool,
}

#[derive(Serialize, Deserialize)]
struct JsonMetaCompose {
    pub json: ModelUpsetRequest,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize)]
struct Json1 {
    pub id: i64,
    #[serde(rename = "nsfwLevel")]
    pub nsfw_level: i64,
}

#[derive(Serialize, Deserialize)]
struct Data {
    pub json: Json1,
}

#[derive(Serialize, Deserialize)]
struct Result {
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
struct ModelUpsetResponse {
    pub result: Result,
}

