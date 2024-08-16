use std::env::VarError;

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
}


