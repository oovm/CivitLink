use serde::{Deserialize, Serialize};

/// 文档中的位置，由行号和字符偏移表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 行号，从零开始
    pub line: u32,
    /// 字符偏移，从零开始
    pub character: u32,
}

/// 文档中的范围，由起始和结束位置表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    /// 范围的起始位置
    pub start: Position,
    /// 范围的结束位置
    pub end: Position,
}

/// 文档中的位置信息，包含 URI 和范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// 文档的 URI
    pub uri: String,
    /// 文档中的范围
    pub range: Range,
}

/// 诊断严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    /// 错误
    Error = 1,
    /// 警告
    Warning = 2,
    /// 信息
    Information = 3,
    /// 提示
    Hint = 4,
}

/// 诊断信息，表示文档中的问题或建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// 诊断的范围
    pub range: Range,
    /// 诊断的严重程度
    pub severity: Option<DiagnosticSeverity>,
    /// 诊断代码
    pub code: Option<String>,
    /// 诊断来源
    pub source: Option<String>,
    /// 诊断消息
    pub message: String,
}

/// 自动补全项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// 补全项的标签
    pub label: String,
    /// 补全项的类型
    pub kind: Option<u32>,
    /// 补全项的详细信息
    pub detail: Option<String>,
    /// 补全项的文档
    pub documentation: Option<String>,
    /// 补全项的插入文本
    pub insert_text: Option<String>,
}

/// 悬停信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    /// 悬停内容
    pub contents: String,
    /// 悬停范围
    pub range: Option<Range>,
}

/// 文本文档标识符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    /// 文档的 URI
    pub uri: String,
}

/// 带版本号的文本文档标识符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    /// 文档的 URI
    pub uri: String,
    /// 文档的版本号
    pub version: i32,
}

/// 文本文档内容变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    /// 变更的范围，None 表示全文替换
    pub range: Option<Range>,
    /// 变更范围的长度
    pub range_length: Option<u32>,
    /// 变更的文本内容
    pub text: String,
}
