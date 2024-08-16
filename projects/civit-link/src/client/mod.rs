use std::env::VarError;
use reqwest::{Client, Request};
use reqwest::header::{CONTENT_TYPE, COOKIE, SET_COOKIE, USER_AGENT};

mod tokens;


pub struct CivitClient {
    __secure_civitai_token: String,
}


impl CivitClient {
    pub fn new() -> CivitClient {
        let token = match CivitClient::env_civit_token() {
            Ok(o) => {
                o
            }
            Err(_) => {
                String::new()
            }
        };
        Self {
            __secure_civitai_token: token,
        }
    }
    pub fn env_civit_token() -> Result<String, VarError> {
        std::env::var("__Secure-civitai-token")
    }
    pub fn get(&self, url: &str, authorize: bool) -> reqwest::Result<Request> {
        let mut client = Client::new()
            .get(url)
            .header(USER_AGENT, format!("CivitLink/{}", env!("CARGO_PKG_VERSION")))
            .header(CONTENT_TYPE, "application/json")
            ;
        if authorize {
            client = client.header(SET_COOKIE, self.__secure_civitai_token.as_ref())
        }
        client.build()
    }
}


