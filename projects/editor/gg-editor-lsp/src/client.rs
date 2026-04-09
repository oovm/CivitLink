use gg_core::{GError, GErrorKind, GResult};

use crate::transport::LspTransport;
use crate::types::{
    CompletionItem, Hover, Location, Position, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, VersionedTextDocumentIdentifier,
};

/// LSP 客户端，封装与语言服务器的交互逻辑
pub struct LspClient {
    /// 传输层实现
    pub transport: Box<dyn LspTransport>,
    /// 是否已初始化
    pub initialized: bool,
}

impl LspClient {
    /// 创建新的 LSP 客户端
    ///
    /// # 参数
    /// - `transport`: LSP 传输层实现
    pub fn new(transport: Box<dyn LspTransport>) -> Self {
        Self {
            transport,
            initialized: false,
        }
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
        self.transport
            .send_notification("initialized", initialized_params)?;
        Ok(())
    }

    /// 关闭语言服务器
    ///
    /// 发送 shutdown 请求，成功后设置 initialized 为 false
    pub fn shutdown(&mut self) -> GResult<()> {
        self.transport
            .send_request("shutdown", serde_json::Value::Null)?;
        self.transport
            .send_notification("exit", serde_json::Value::Null)?;
        self.initialized = false;
        Ok(())
    }

    /// 通知语言服务器文档已打开
    ///
    /// # 参数
    /// - `document`: 带版本号的文本文档标识符
    /// - `language_id`: 语言标识
    /// - `text`: 文档文本内容
    pub fn did_open(
        &mut self,
        document: &VersionedTextDocumentIdentifier,
        language_id: &str,
        text: &str,
    ) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
                "languageId": language_id,
                "version": document.version,
                "text": text,
            }
        });
        self.transport
            .send_notification("textDocument/didOpen", params)
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
        self.transport
            .send_notification("textDocument/didChange", params)
    }

    /// 通知语言服务器文档已关闭
    ///
    /// # 参数
    /// - `document`: 文本文档标识符
    pub fn did_close(
        &mut self,
        document: &TextDocumentIdentifier,
    ) -> GResult<()> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            }
        });
        self.transport
            .send_notification("textDocument/didClose", params)
    }

    /// 请求自动补全
    ///
    /// # 参数
    /// - `document`: 文本文档标识符
    /// - `position`: 请求补全的位置
    ///
    /// # 返回
    /// 补全项列表
    pub fn completion(
        &mut self,
        document: &TextDocumentIdentifier,
        position: &Position,
    ) -> GResult<Vec<CompletionItem>> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });
        let result = self
            .transport
            .send_request("textDocument/completion", params)?;
        let items: Vec<CompletionItem> = serde_json::from_value(result).map_err(|e| {
            GError {
                kind: GErrorKind::Other,
                message: format!("解析补全结果失败: {}", e),
            }
        })?;
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
    pub fn hover(
        &mut self,
        document: &TextDocumentIdentifier,
        position: &Position,
    ) -> GResult<Option<crate::types::Hover>> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });
        let result = self
            .transport
            .send_request("textDocument/hover", params)?;
        if result.is_null() {
            Ok(None)
        } else {
            let hover: Hover = serde_json::from_value(result).map_err(|e| {
                GError {
                    kind: GErrorKind::Other,
                    message: format!("解析悬停结果失败: {}", e),
                }
            })?;
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
    pub fn goto_definition(
        &mut self,
        document: &TextDocumentIdentifier,
        position: &Position,
    ) -> GResult<Option<Location>> {
        let params = serde_json::json!({
            "textDocument": {
                "uri": document.uri,
            },
            "position": {
                "line": position.line,
                "character": position.character,
            }
        });
        let result = self
            .transport
            .send_request("textDocument/definition", params)?;
        if result.is_null() {
            Ok(None)
        } else {
            let location: Location = serde_json::from_value(result).map_err(|e| {
                GError {
                    kind: GErrorKind::Other,
                    message: format!("解析定义位置结果失败: {}", e),
                }
            })?;
            Ok(Some(location))
        }
    }
}
