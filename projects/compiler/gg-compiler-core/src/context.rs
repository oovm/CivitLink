//! 编译上下文模块
//! 定义编译诊断信息、编译配置和编译上下文类型

use std::collections::HashMap;

/// 编译诊断级别
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
}

/// 编译诊断信息
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// 诊断级别
    pub level: DiagnosticLevel,
    /// 来源转换器名称
    pub source: String,
    /// 诊断消息
    pub message: String,
}

/// 编译配置键值对
pub struct BuildConfig {
    /// 配置项映射
    entries: HashMap<String, String>,
}

impl BuildConfig {
    /// 创建一个空的编译配置
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// 获取指定键的配置值
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    /// 设置指定键的配置值
    pub fn set(&mut self, key: &str, value: &str) {
        self.entries.insert(key.to_string(), value.to_string());
    }

    /// 检查是否包含指定键的配置项
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 编译上下文，在流水线执行期间传递
pub struct BuildContext {
    /// 编译配置
    pub config: BuildConfig,
    /// 收集的诊断信息
    pub diagnostics: Vec<Diagnostic>,
}

impl BuildContext {
    /// 创建一个新的编译上下文
    pub fn new(config: BuildConfig) -> Self {
        Self { config, diagnostics: Vec::new() }
    }

    /// 添加一条诊断信息
    pub fn add_diagnostic(&mut self, level: DiagnosticLevel, source: &str, message: &str) {
        self.diagnostics.push(Diagnostic { level, source: source.to_string(), message: message.to_string() });
    }

    /// 检查是否存在错误级别的诊断
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error)
    }

    /// 获取所有诊断信息的切片
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
