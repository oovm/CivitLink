//! JSON 日志格式化器
//!
//! 提供 JSON 格式的日志记录，包含 timestamp、level、message、trace_id、span_id 等字段。

/// 自定义 JSON 字段配置
#[derive(Debug, Clone)]
pub struct JsonFormatterConfig {
    /// 是否包含时间戳
    pub include_timestamp: bool,
    /// 是否包含日志级别
    pub include_level: bool,
    /// 是否包含消息
    pub include_message: bool,
    /// 是否包含 trace_id
    pub include_trace_id: bool,
    /// 是否包含 span_id
    pub include_span_id: bool,
    /// 是否包含目标
    pub include_target: bool,
    /// 是否包含文件名
    pub include_file: bool,
    /// 是否包含行号
    pub include_line: bool,
}

impl Default for JsonFormatterConfig {
    fn default() -> Self {
        Self {
            include_timestamp: true,
            include_level: true,
            include_message: true,
            include_trace_id: true,
            include_span_id: true,
            include_target: true,
            include_file: true,
            include_line: true,
        }
    }
}

/// JSON 日志格式化器
#[derive(Debug, Clone, Default)]
pub struct JsonFormatter {
    config: JsonFormatterConfig,
}

impl JsonFormatter {
    /// 创建新的 JSON 格式化器
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用自定义配置创建 JSON 格式化器
    pub fn with_config(config: JsonFormatterConfig) -> Self {
        Self { config }
    }

    /// 设置是否包含时间戳
    pub fn include_timestamp(mut self, include: bool) -> Self {
        self.config.include_timestamp = include;
        self
    }

    /// 设置是否包含日志级别
    pub fn include_level(mut self, include: bool) -> Self {
        self.config.include_level = include;
        self
    }

    /// 设置是否包含消息
    pub fn include_message(mut self, include: bool) -> Self {
        self.config.include_message = include;
        self
    }

    /// 设置是否包含 trace_id
    pub fn include_trace_id(mut self, include: bool) -> Self {
        self.config.include_trace_id = include;
        self
    }

    /// 设置是否包含 span_id
    pub fn include_span_id(mut self, include: bool) -> Self {
        self.config.include_span_id = include;
        self
    }

    /// 设置是否包含目标
    pub fn include_target(mut self, include: bool) -> Self {
        self.config.include_target = include;
        self
    }

    /// 设置是否包含文件名
    pub fn include_file(mut self, include: bool) -> Self {
        self.config.include_file = include;
        self
    }

    /// 设置是否包含行号
    pub fn include_line(mut self, include: bool) -> Self {
        self.config.include_line = include;
        self
    }
}
