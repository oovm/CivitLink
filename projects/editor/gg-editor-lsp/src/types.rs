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

/// 文档同步方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDocumentSyncKind {
    /// 不同步
    None = 0,
    /// 全量同步
    Full = 1,
    /// 增量同步
    Incremental = 2,
}

/// 补全提供者选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionOptions {
    /// 补全提供者支持的触发字符
    pub trigger_characters: Option<Vec<String>>,
    /// 是否在连续输入时提供补全
    pub resolve_provider: Option<bool>,
}

/// 语言服务器能力声明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// 文档同步方式
    pub text_document_sync: Option<TextDocumentSyncKind>,
    /// 补全提供者
    pub completion_provider: Option<CompletionOptions>,
    /// 悬停提供者
    pub hover_provider: Option<bool>,
    /// 定义跳转提供者
    pub definition_provider: Option<bool>,
}

/// LSP initialize 请求的响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// 语言服务器能力
    pub capabilities: ServerCapabilities,
}

/// LSP 连接配置
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// 语言服务器可执行文件路径
    pub server_command: String,
    /// 传递给语言服务器的命令行参数
    pub server_args: Vec<String>,
    /// 项目根目录 URI
    pub root_uri: String,
}
