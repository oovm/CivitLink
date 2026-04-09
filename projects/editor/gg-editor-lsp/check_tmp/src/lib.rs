mod gg_core;
mod types;

use gg_core::{GError, GErrorKind, GResult};
use std::collections::HashMap;
use types::*;

pub trait LspTransport {
    fn send_request(&mut self, method: &str, params: serde_json::Value) -> GResult<serde_json::Value>;
    fn send_notification(&mut self, method: &str, params: serde_json::Value) -> GResult<()>;
}

pub struct LspClient {
    pub transport: Box<dyn LspTransport>,
    pub initialized: bool,
}

impl LspClient {
    pub fn new(transport: Box<dyn LspTransport>) -> Self {
        Self { transport, initialized: false }
    }

    pub fn initialize(&mut self, root_uri: &str) -> GResult<()> {
        let params = serde_json::json!({ "rootUri": root_uri, "capabilities": {} });
        self.transport.send_request("initialize", params)?;
        self.initialized = true;
        self.transport.send_notification("initialized", serde_json::json!({}))?;
        Ok(())
    }

    pub fn shutdown(&mut self) -> GResult<()> {
        self.transport.send_request("shutdown", serde_json::Value::Null)?;
        self.transport.send_notification("exit", serde_json::Value::Null)?;
        self.initialized = false;
        Ok(())
    }

    pub fn did_open(&mut self, document: &VersionedTextDocumentIdentifier, language_id: &str, text: &str) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": { "uri": document.uri, "languageId": language_id, "version": document.version, "text": text }
        });
        self.transport.send_notification("textDocument/didOpen", params)
    }

    pub fn did_change(&mut self, document: &VersionedTextDocumentIdentifier, changes: Vec<TextDocumentContentChangeEvent>) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": { "uri": document.uri, "version": document.version },
            "contentChanges": changes,
        });
        self.transport.send_notification("textDocument/didChange", params)
    }

    pub fn did_close(&mut self, document: &TextDocumentIdentifier) -> GResult<()> {
        let params = serde_json::json!({ "textDocument": { "uri": document.uri } });
        self.transport.send_notification("textDocument/didClose", params)
    }

    pub fn completion(&mut self, document: &TextDocumentIdentifier, position: &Position) -> GResult<Vec<CompletionItem>> {
        let params = serde_json::json!({
            "textDocument": { "uri": document.uri },
            "position": { "line": position.line, "character": position.character }
        });
        let result = self.transport.send_request("textDocument/completion", params)?;
        let items: Vec<CompletionItem> = serde_json::from_value(result).map_err(|e| GError {
            kind: GErrorKind::Other,
            message: format!("解析补全结果失败: {}", e),
        })?;
        Ok(items)
    }

    pub fn hover(&mut self, document: &TextDocumentIdentifier, position: &Position) -> GResult<Option<Hover>> {
        let params = serde_json::json!({
            "textDocument": { "uri": document.uri },
            "position": { "line": position.line, "character": position.character }
        });
        let result = self.transport.send_request("textDocument/hover", params)?;
        if result.is_null() { Ok(None) } else {
            let h: Hover = serde_json::from_value(result).map_err(|e| GError {
                kind: GErrorKind::Other,
                message: format!("解析悬停结果失败: {}", e),
            })?;
            Ok(Some(h))
        }
    }

    pub fn goto_definition(&mut self, document: &TextDocumentIdentifier, position: &Position) -> GResult<Option<Location>> {
        let params = serde_json::json!({
            "textDocument": { "uri": document.uri },
            "position": { "line": position.line, "character": position.character }
        });
        let result = self.transport.send_request("textDocument/definition", params)?;
        if result.is_null() { Ok(None) } else {
            let loc: Location = serde_json::from_value(result).map_err(|e| GError {
                kind: GErrorKind::Other,
                message: format!("解析定义位置结果失败: {}", e),
            })?;
            Ok(Some(loc))
        }
    }
}

pub struct DiagnosticCollector {
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
}

impl DiagnosticCollector {
    pub fn new() -> Self { Self { diagnostics: HashMap::new() } }
    pub fn add_diagnostics(&mut self, uri: &str, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.insert(uri.to_string(), diagnostics);
    }
    pub fn clear_diagnostics(&mut self, uri: &str) { self.diagnostics.remove(uri); }
    pub fn get_diagnostics(&self, uri: &str) -> &[Diagnostic] {
        self.diagnostics.get(uri).map(|v| v.as_slice()).unwrap_or(&[])
    }
    pub fn get_all_diagnostics(&self) -> &HashMap<String, Vec<Diagnostic>> { &self.diagnostics }
    pub fn error_count(&self) -> usize {
        self.diagnostics.values().flat_map(|v| v.iter())
            .filter(|d| d.severity == Some(DiagnosticSeverity::Error)).count()
    }
    pub fn warning_count(&self) -> usize {
        self.diagnostics.values().flat_map(|v| v.iter())
            .filter(|d| d.severity == Some(DiagnosticSeverity::Warning)).count()
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self { Self::new() }
}
