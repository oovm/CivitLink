use std::str::FromStr;

use crate::CivitResult;
use reqwest::Url;

pub use self::defines::AllModels;

mod defines;

/// Get the next page of all_models_info
///
/// <https://github.com/civitai/civitai/wiki/REST-API-Reference#get-apiv1models>
#[derive(Clone, Debug)]
pub struct RequestAllModels {
    /// The page from which to start fetching models
    pub page: usize,
}

impl Default for RequestAllModels {
    fn default() -> Self {
        Self { page: 1 }
    }
}

impl RequestAllModels {
    pub fn new(page: usize) -> Self {
        Self { page, ..Default::default() }
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }
    /// Send the request
    pub async fn send(&self) -> CivitResult<AllModels> {
        Ok(reqwest::get(self.url()).await?.json().await?)
    }
    /// Get the url for the request
    pub fn url(&self) -> String {
        format!("https://civitai.com/api/v1/models?page={page}", page = self.page)
    }
}

impl FromStr for RequestAllModels {
    type Err = trauma::Error;

    fn from_str(s: &str) -> CivitResult<Self> {
        let mut out = RequestAllModels::default();
        match Url::parse(s) {
            Ok(o) => {
                for (key, value) in o.query_pairs() {
                    match key.as_ref() {
                        "page" => match value.parse() {
                            Ok(o) => out.page = o,
                            Err(_) => {}
                        },
                        _ => {
                            println!("Unknown key: {} => {}", key, value);
                        }
                    }
                }
            }
            Err(e) => Err(trauma::Error::InvalidUrl(e.to_string()))?,
        }
        Ok(out)
    }
}

impl AllModels {
    /// Get the next page of all_models_info
    pub fn next_page(&self) -> CivitResult<RequestAllModels> {
        RequestAllModels::from_str(&self.metadata.next_page)
    }
}
