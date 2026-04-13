//! 编译上下文模块
//! 定义编译诊断信息、编译配置和编译上下文类型

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 编译诊断级别
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
}

impl DiagnosticLevel {
    /// 获取诊断级别的显示名称
    pub fn display(&self) -> &'static str {
        match self {
            DiagnosticLevel::Error => "ERROR",
            DiagnosticLevel::Warning => "WARNING",
            DiagnosticLevel::Info => "INFO",
        }
    }
}

/// 源码位置信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// 源文件路径
    pub file: String,
    /// 行号（从 1 开始）
    pub line: u32,
    /// 列号（从 1 开始）
    pub column: u32,
}

impl SourceLocation {
    /// 创建新的源码位置
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self { file: file.into(), line, column }
    }
}

/// 编译诊断信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// 诊断级别
    pub level: DiagnosticLevel,
    /// 来源转换器名称
    pub source: String,
    /// 诊断消息
    pub message: String,
    /// 错误代码
    pub error_code: Option<String>,
    /// 源码位置
    pub source_location: Option<SourceLocation>,
    /// 修复建议
    pub suggestion: Option<String>,
    /// 相关代码片段
    #[serde(default)]
    pub code_snippet: Option<String>,
}

impl Diagnostic {
    /// 格式化诊断信息为可读字符串
    pub fn format(&self) -> String {
        let mut result = String::new();

        result.push_str(&format!("[{}]", self.level.display()));

        if let Some(ref loc) = self.source_location {
            result.push_str(&format!(" {}:{}:{}", loc.file, loc.line, loc.column));
        }

        if let Some(ref code) = self.error_code {
            result.push_str(&format!(" [{}]", code));
        }

        result.push_str(&format!(" {}", self.message));

        if let Some(ref snippet) = self.code_snippet {
            result.push('\n');
            result.push_str(&format!("  | {}", snippet));
        }

        if let Some(ref suggestion) = self.suggestion {
            result.push('\n');
            result.push_str(&format!("  help: {}", suggestion));
        }

        result
    }
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
        self.diagnostics.push(Diagnostic {
            level,
            source: source.to_string(),
            message: message.to_string(),
            error_code: None,
            source_location: None,
            suggestion: None,
            code_snippet: None,
        });
    }

    /// 添加一条带详细信息的诊断信息
    pub fn add_diagnostic_with_details(
        &mut self,
        level: DiagnosticLevel,
        source: &str,
        message: &str,
        error_code: Option<String>,
        source_location: Option<SourceLocation>,
        suggestion: Option<String>,
    ) {
        self.diagnostics.push(Diagnostic {
            level,
            source: source.to_string(),
            message: message.to_string(),
            error_code,
            source_location,
            suggestion,
            code_snippet: None,
        });
    }

    /// 创建错误级别诊断的便捷方法
    pub fn error(source: &str, message: &str) -> Diagnostic {
        Diagnostic {
            level: DiagnosticLevel::Error,
            source: source.to_string(),
            message: message.to_string(),
            error_code: None,
            source_location: None,
            suggestion: None,
            code_snippet: None,
        }
    }

    /// 创建警告级别诊断的便捷方法
    pub fn warning(source: &str, message: &str) -> Diagnostic {
        Diagnostic {
            level: DiagnosticLevel::Warning,
            source: source.to_string(),
            message: message.to_string(),
            error_code: None,
            source_location: None,
            suggestion: None,
            code_snippet: None,
        }
    }

    /// 检查是否存在错误级别的诊断
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Error)
    }

    /// 获取所有诊断信息的切片
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// 格式化所有诊断信息，按级别排序（Error > Warning > Info）
    pub fn format_diagnostics(&self) -> String {
        let mut sorted = self.diagnostics.clone();
        sorted.sort_by(|a, b| {
            let level_order = |l: &DiagnosticLevel| match l {
                DiagnosticLevel::Error => 0,
                DiagnosticLevel::Warning => 1,
                DiagnosticLevel::Info => 2,
            };
            level_order(&a.level).cmp(&level_order(&b.level))
        });
        sorted.iter().map(|d| d.format()).collect::<Vec<_>>().join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_format_with_all_fields() {
        let diagnostic = Diagnostic {
            level: DiagnosticLevel::Error,
            source: "type_checker".to_string(),
            message: "Type mismatch".to_string(),
            error_code: Some("E001".to_string()),
            source_location: Some(SourceLocation::new("main.script", 10, 5)),
            suggestion: Some("Expected Int but found String".to_string()),
            code_snippet: Some("let x: Int = \"hello\"".to_string()),
        };
        let formatted = diagnostic.format();
        assert!(formatted.contains("[ERROR]"));
        assert!(formatted.contains("main.script:10:5"));
        assert!(formatted.contains("[E001]"));
        assert!(formatted.contains("Type mismatch"));
        assert!(formatted.contains("  | let x: Int = \"hello\""));
        assert!(formatted.contains("  help: Expected Int but found String"));
    }

    #[test]
    fn test_diagnostic_format_with_minimal_fields() {
        let diagnostic = Diagnostic {
            level: DiagnosticLevel::Warning,
            source: "parser".to_string(),
            message: "Unused variable".to_string(),
            error_code: None,
            source_location: None,
            suggestion: None,
            code_snippet: None,
        };
        let formatted = diagnostic.format();
        assert_eq!(formatted, "[WARNING] Unused variable");
    }

    #[test]
    fn test_diagnostic_level_display() {
        assert_eq!(DiagnosticLevel::Error.display(), "ERROR");
        assert_eq!(DiagnosticLevel::Warning.display(), "WARNING");
        assert_eq!(DiagnosticLevel::Info.display(), "INFO");
    }

    #[test]
    fn test_format_diagnostics_sorting() {
        let mut context = BuildContext::new(BuildConfig::new());
        context.add_diagnostic(DiagnosticLevel::Info, "source", "info message");
        context.add_diagnostic(DiagnosticLevel::Error, "source", "error message");
        context.add_diagnostic(DiagnosticLevel::Warning, "source", "warning message");

        let formatted = context.format_diagnostics();
        let error_pos = formatted.find("[ERROR]").unwrap();
        let warning_pos = formatted.find("[WARNING]").unwrap();
        let info_pos = formatted.find("[INFO]").unwrap();

        assert!(error_pos < warning_pos);
        assert!(warning_pos < info_pos);
    }
}
