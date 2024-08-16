mod defines;
use serde_derive::{Deserialize, Serialize};

use crate::CivitResult;

/// Get model by id or hash
///
/// - <https://github.com/civitai/civitai/wiki/REST-API-Reference#get-apiv1models-versionsmodelversionid>
/// - <https://github.com/civitai/civitai/wiki/REST-API-Reference#get-apiv1models-versionsby-hashhash>
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestModelVersion {
    pub id: usize,
    pub hash: String,
}

impl RequestModelVersion {
    pub fn new(id: usize) -> Self {
        Self { id, ..Default::default() }
    }
    pub fn with_hash(mut self, hash: String) -> Self {
        self.hash = hash;
        self
    }
    pub async fn send(&self) -> CivitResult<()> {
        Ok(reqwest::get(self.url()).await?.json().await?)
    }
    pub fn url(&self) -> String {
        if self.hash.is_empty() {
            format!("https://civitai.com/api/v1/models/versions/{}", self.id)
        }
        else {
            format!("https://civitai.com/api/v1/models/versions/by-hash/{}", self.hash)
        }
    }
}
