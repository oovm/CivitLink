//! 速率限制模块
//!
//! 提供登录/注册等敏感接口的速率限制功能，防范暴力破解攻击。

mod config;
mod error;
mod service;

pub use config::*;
pub use error::*;
pub use service::*;
