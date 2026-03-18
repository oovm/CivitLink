//! 测试环境配置模块

use std::{collections::HashMap, time::Duration};

/// 测试环境配置
#[derive(Debug, Clone)]
pub struct TestEnvConfig {
    /// 环境名称
    pub name: String,
    /// 是否启用日志
    pub enable_logging: bool,
    /// 是否启用追踪
    pub enable_tracing: bool,
    /// 超时时间
    pub default_timeout: Duration,
    /// 自定义配置
    pub custom: HashMap<String, String>,
}

impl Default for TestEnvConfig {
    fn default() -> Self {
        Self {
            name: "test".to_string(),
            enable_logging: true,
            enable_tracing: false,
            default_timeout: Duration::from_secs(30),
            custom: HashMap::new(),
        }
    }
}

impl TestEnvConfig {
    /// 创建新的测试环境配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置环境名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 启用日志
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// 启用追踪
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }

    /// 设置超时时间
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// 添加自定义配置
    pub fn custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

/// 测试服务配置
///
/// 定义测试环境中需要启动的服务配置，支持多服务集成测试。
#[derive(Debug, Clone)]
pub struct TestServiceConfig {
    /// 服务名称
    pub name: String,
    /// 服务是否启用
    pub enabled: bool,
    /// 服务启动超时时间
    pub startup_timeout: Duration,
    /// 服务特定配置
    pub config: HashMap<String, String>,
}

impl TestServiceConfig {
    /// 创建新的测试服务配置
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), enabled: true, startup_timeout: Duration::from_secs(30), config: HashMap::new() }
    }

    /// 设置服务启用状态
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// 设置服务启动超时时间
    pub fn startup_timeout(mut self, timeout: Duration) -> Self {
        self.startup_timeout = timeout;
        self
    }

    /// 添加服务配置项
    pub fn config(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}
