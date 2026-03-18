//! SAML 2.0 (Security Assertion Markup Language) 模块
//!
//! 提供企业级单点登录 (SSO) 功能，支持 SAML 2.0 协议。

mod assertion;
mod config;
mod metadata;
mod request;
mod response;
mod service;

pub use assertion::*;
pub use config::*;
pub use metadata::*;
pub use request::*;
pub use response::*;
pub use service::*;
