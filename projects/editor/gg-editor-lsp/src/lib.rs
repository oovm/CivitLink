#![warn(missing_docs)]

//! GG 编辑器 LSP 客户端模块
//! 提供语言服务器协议客户端接口和诊断信息收集功能

pub mod client;
pub mod diagnostic;
pub mod transport;
pub mod types;

pub use client::LspClient;
pub use diagnostic::DiagnosticCollector;
pub use transport::LspTransport;
pub use types::{
    CompletionItem, Diagnostic, DiagnosticSeverity, Hover, Location, Position, Range, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, VersionedTextDocumentIdentifier,
};
