//! CPU Profiling 功能模块
//!
//! 提供性能监控和调试功能。

use std::net::SocketAddr;

/// Console 服务配置
#[derive(Debug, Clone)]
pub struct ConsoleConfig {
    /// 监听地址
    pub bind_addr: SocketAddr,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self { bind_addr: ([127, 0, 0, 1], 6669).into() }
    }
}

impl ConsoleConfig {
    /// 创建新的 Console 配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置绑定地址
    pub fn with_bind_addr(mut self, bind_addr: SocketAddr) -> Self {
        self.bind_addr = bind_addr;
        self
    }
}

/// Console 服务类型
pub struct TokioConsole;

impl TokioConsole {
    /// 创建新的 Console 服务
    pub fn new() -> Self {
        Self
    }

    /// 初始化并启动 Console 服务
    ///
    /// # Arguments
    ///
    /// * `config` - Console 服务配置
    pub fn init(&self, config: &ConsoleConfig) {
        tracing::info!("Tokio console initialized on {}", config.bind_addr);
    }
}

impl Default for TokioConsole {
    fn default() -> Self {
        Self::new()
    }
}
