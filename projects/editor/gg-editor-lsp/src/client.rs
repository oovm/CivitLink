//! LSP 客户端实现
//!
//! 封装与语言服务器的交互逻辑，包括初始化、文档同步和语言功能请求。

use gg_core::{GError, GErrorKind, GResult};

use crate::{
    diagnostic::DiagnosticCollector,
    transport::LspTransport,
    types::{
        CompletionItem, Diagnostic, Hover, Location, Position, TextDocumentContentChangeEvent, TextDocumentIdentifier,
        VersionedTextDocumentIdentifier,
    },
};

/// 诊断回调类型
///
/// 当收到语言服务器的诊断通知时调用，参数为文档 URI 和诊断列表
pub type OnDiagnosticsFn = dyn FnMut(&str, &[Diagnostic]);

/// LSP 客户端，封装与语言服务器的交互逻辑
pub struct LspClient {
    /// 传输层实现
    pub transport: Box<dyn LspTransport>,
    /// 是否已初始化
    pub initialized: bool,
    /// 诊断信息收集器
    diagnostic_collector: DiagnosticCollector,
    /// 诊断回调，当收到诊断通知时调用
    on_diagnostics: Option<Box<OnDiagnosticsFn>>,
}

impl LspClient {
    /// 创建新的 LSP 客户端
    ///
    /// # 参数
    /// - `transport`: LSP 传输层实现
    pub fn new(transport: Box<dyn LspTransport>) -> Self {
        Self { transport, initialized: false, diagnostic_collector: DiagnosticCollector::new(), on_diagnostics: None }
    }

    /// 设置诊断回调
    ///
    /// 当收到语言服务器的诊断通知时，将调用此回调
    ///
    /// # 参数
    /// - `callback`: 诊断回调函数
    pub fn set_on_diagnostics(&mut self, callback: Box<OnDiagnosticsFn>) {
        self.on_diagnostics = Some(callback);
    }

    /// 处理语言服务器发送的通知
    ///
    /// 根据通知方法名分发处理逻辑，目前支持：
    /// - `textDocument/publishDiagnostics`: 提取诊断信息并触发回调
    ///
    /// # 参数
    /// - `method`: 通知方法名
    /// - `params`: 通知参数
    pub fn handle_notification(&mut self, method: &str, params: serde_json::Value) {
        match method {
            "textDocument/publishDiagnostics" => {
                if let Some(uri) = params.get("uri").and_then(|v| v.as_str()) {
                    let diagnostics: Vec<Diagnostic> =
                        params.get("diagnostics").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
                    self.diagnostic_collector.add_diagnostics(uri, diagnostics.clone());
                    if let Some(callback) = &mut self.on_diagnostics {
                        callback(uri, &diagnostics);
                    }
                }
            }
            _ => {}
        }
    }

    /// 获取诊断信息收集器引用
    pub fn diagnostic_collector(&self) -> &DiagnosticCollector {
        &self.diagnostic_collector
    }

    /// 获取诊断信息收集器可变引用
    pub fn diagnostic_collector_mut(&mut self) -> &mut DiagnosticCollector {
        &mut self.diagnostic_collector
    }

    /// 初始化语言服务器
    ///
    /// 发送 initialize 请求，成功后设置 initialized 为 true
    ///
    /// # 参数
    /// - `root_uri`: 项目根目录 URI
    pub fn initialize(&mut self, root_uri: &str) -> GResult<()> {
        let params = serde_json::json!({
            "rootUri": root_uri,
            "capabilities": {},
        });
        self.transport.send_request("initialize", params)?;
        self.initialized = true;
        let initialized_params = serde_json::json!({});
        self.transport.send_notification("initialized", initialized_params)?;
        Ok(())
    }

    /// 关闭语言服务器
    ///
    /// 发送 shutdown 请求，成功后设置 initialized 为 false
    pub fn shutdown(&mut self) -> GResult<()> {
        self.transport.send_request("shutdown", serde_json::Value::Null)?;
        self.transport.send_notification("exit", serde_json::Value::Null)?;
        self.initialized = false;
        Ok(())
    }

    /// 通知语言服务器文档已打开
    ///
    /// # 参数
    /// - `document`: 带版本号的文本文档标识符
    /// - `language_id`: 语言标识
    /// - `text`: 文档文本内容
    pub fn did_open(&mut self, document: &VersionedTextDocumentIdentifier, language_id: &str, text: &str) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
                "languageId": language_id,
                "version": document.version,
                "text": text,
            }
        });
        self.transport.send_notification("textDocument/didOpen", params)
    }

    /// 通知语言服务器文档内容已变更
    ///
    /// # 参数
    /// - `document`: 带版本号的文本文档标识符
    /// - `changes`: 变更事件列表
    pub fn did_change(
        &mut self,
        document: &VersionedTextDocumentIdentifier,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
                "version": document.version,
            },
            "contentChanges": changes,
        });
        self.transport.send_notification("textDocument/didChange", params)
    }

    /// 通知语言服务器文档已关闭
    ///
    /// # 参数
    /// - `document`: 文本文档标识符
    pub fn did_close(&mut self, document: &TextDocumentIdentifier) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            }
        });
        self.transport.send_notification("textDocument/didClose", params)
    }

    /// 请求自动补全
    ///
    /// # 参数
    /// - `document`: 文本文档标识符
    /// - `position`: 请求补全的位置
    ///
    /// # 返回
    /// 补全项列表
    pub fn completion(&mut self, document: &TextDocumentIdentifier, position: &Position) -> GResult<Vec<CompletionItem>> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });
        let result = self.transport.send_request("textDocument/completion", params)?;
        let items: Vec<CompletionItem> = serde_json::from_value(result)
            .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析补全结果失败: {}", e) })?;
        Ok(items)
    }

    /// 请求悬停信息
    ///
    /// # 参数
    /// - `document`: 文本文档标识符
    /// - `position`: 请求悬停的位置
    ///
    /// # 返回
    /// 悬停信息，如果无可用信息则返回 None
    pub fn hover(&mut self, document: &TextDocumentIdentifier, position: &Position) -> GResult<Option<crate::types::Hover>> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });
        let result = self.transport.send_request("textDocument/hover", params)?;
        if result.is_null() {
            Ok(None)
        }
        else {
            let hover: Hover = serde_json::from_value(result)
                .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析悬停结果失败: {}", e) })?;
            Ok(Some(hover))
        }
    }

    /// 请求跳转到定义
    ///
    /// # 参数
    /// - `document`: 文本文档标识符
    /// - `position`: 请求跳转的位置
    ///
    /// # 返回
    /// 定义位置，如果无可用定义则返回 None
    pub fn goto_definition(&mut self, document: &TextDocumentIdentifier, position: &Position) -> GResult<Option<Location>> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });
        let result = self.transport.send_request("textDocument/definition", params)?;
        if result.is_null() {
            Ok(None)
        }
        else {
            let location: Location = serde_json::from_value(result)
                .map_err(|e| GError { kind: GErrorKind::Other, message: format!("解析定义位置结果失败: {}", e) })?;
            Ok(Some(location))
        }
    }
}
