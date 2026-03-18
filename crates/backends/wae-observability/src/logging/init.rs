//! 日志初始化模块
//!
//! 提供 JSON 日志初始化功能。

use crate::logging::json::JsonFormatterConfig;
use ::tracing::Level;

/// 日志输出目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogOutput {
    /// 输出到标准输出
    #[default]
    Stdout,
    /// 输出到标准错误
    Stderr,
}

/// JSON 日志初始化配置
#[derive(Debug, Clone)]
pub struct JsonLoggerConfig {
    /// 日志级别
    pub level: Level,
    /// 日志输出目标
    pub output: LogOutput,
    /// JSON 格式化器配置
    pub formatter_config: JsonFormatterConfig,
}

impl Default for JsonLoggerConfig {
    fn default() -> Self {
        Self { level: Level::INFO, output: LogOutput::Stdout, formatter_config: JsonFormatterConfig::default() }
    }
}

impl JsonLoggerConfig {
    /// 创建新的 JSON 日志配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置日志级别
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// 设置日志输出目标
    pub fn with_output(mut self, output: LogOutput) -> Self {
        self.output = output;
        self
    }

    /// 设置 JSON 格式化器配置
    pub fn with_formatter_config(mut self, formatter_config: JsonFormatterConfig) -> Self {
        self.formatter_config = formatter_config;
        self
    }
}

/// 初始化 JSON 日志记录器
///
/// # Examples
///
/// ```
/// use tracing::Level;
/// use wae_observability::logging::*;
///
/// init_json_logger(JsonLoggerConfig::default());
/// ```
pub fn init_json_logger(_config: JsonLoggerConfig) {
    tracing::info!("JSON logger initialized (lightweight implementation)");
}

/// 使用默认配置初始化 JSON 日志记录器
///
/// # Examples
///
/// ```
/// use wae_observability::logging::*;
///
/// init_default_json_logger();
/// ```
pub fn init_default_json_logger() {
    init_json_logger(JsonLoggerConfig::default());
}
