//! Widget 编译器错误类型定义

use gg_core::GError;

/// Widget 编译器错误类型
#[derive(Debug)]
pub enum WidgetError {
    /// 解析错误
    ParseError(String),
    /// 语义分析错误
    SemanticError(String),
    /// 代码生成错误
    CodegenError(String),
    /// 序列化错误
    SerializeError(String),
}

/// Widget 编译器结果类型
pub type WidgetResult<T> = Result<T, WidgetError>;

impl From<WidgetError> for GError {
    fn from(err: WidgetError) -> Self {
        let message = match &err {
            WidgetError::ParseError(msg) => format!("Widget parse error: {}", msg),
            WidgetError::SemanticError(msg) => format!("Widget semantic error: {}", msg),
            WidgetError::CodegenError(msg) => format!("Widget codegen error: {}", msg),
            WidgetError::SerializeError(msg) => format!("Widget serialize error: {}", msg),
        };
        GError { kind: gg_core::GErrorKind::Other, message }
    }
}
