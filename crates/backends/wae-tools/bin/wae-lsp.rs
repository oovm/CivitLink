//! WAE LSP Server - 使用 oak-lsp 实现的标准 Language Server Protocol 服务器

use oak_lsp::{LanguageService, LspServer};
use oak_vfs::{MemoryVfs, Vfs};
use std::sync::Arc;

/// WAE 语言服务实现
struct WaeLanguageService {
    vfs: Arc<MemoryVfs>,
}

impl LanguageService for WaeLanguageService {
    type Vfs = MemoryVfs;

    fn vfs(&self) -> Arc<Self::Vfs> {
        self.vfs.clone()
    }

    fn hover(&self, uri: &str, range: oak_core::Range) -> oak_lsp::service::HoverFuture {
        let response = oak_lsp::types::Hover {
            contents: "# WAE Language Server\n\nThis is a standard LSP server implementation for WAE files.".to_string(),
            range: None,
        };
        Box::pin(std::future::ready(Some(response)))
    }

    fn definition(&self, _uri: &str, _range: oak_core::Range) -> oak_lsp::service::LocationListFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn references(&self, _uri: &str, _range: oak_core::Range) -> oak_lsp::service::LocationListFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn diagnostics(&self, _uri: &str) -> oak_lsp::service::DiagnosticsFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn completion(&self, _uri: &str, _offset: usize) -> oak_lsp::service::CompletionFuture {
        let items = vec![
            oak_lsp::types::CompletionItem {
                label: "model".to_string(),
                kind: Some(oak_lsp::types::CompletionItemKind::CLASS),
                detail: Some("Define a data model".to_string()),
                documentation: None,
                sort_text: None,
                filter_text: None,
                insert_text: Some("model".to_string()),
                insert_text_format: None,
                text_edit: None,
                additional_text_edits: None,
                commit_characters: None,
                command: None,
                data: None,
            },
            oak_lsp::types::CompletionItem {
                label: "field".to_string(),
                kind: Some(oak_lsp::types::CompletionItemKind::FIELD),
                detail: Some("Define a field in a model".to_string()),
                documentation: None,
                sort_text: None,
                filter_text: None,
                insert_text: Some("field".to_string()),
                insert_text_format: None,
                text_edit: None,
                additional_text_edits: None,
                commit_characters: None,
                command: None,
                data: None,
            },
        ];
        Box::pin(std::future::ready(items))
    }

    fn document_symbols(&self, _uri: &str) -> oak_lsp::service::DocumentSymbolsFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn workspace_symbols(&self, _query: &str) -> oak_lsp::service::WorkspaceSymbolsFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn formatting(&self, _uri: &str, _options: oak_lsp::types::FormattingOptions) -> oak_lsp::service::TextEditsFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn range_formatting(
        &self,
        _uri: &str,
        _range: oak_core::Range,
        _options: oak_lsp::types::FormattingOptions,
    ) -> oak_lsp::service::TextEditsFuture {
        Box::pin(std::future::ready(Vec::new()))
    }

    fn on_change(&self, _uri: &str, _content: &str) {
        // 处理文件变化
    }

    fn on_open(&self, _uri: &str, _content: &str) {
        // 处理文件打开
    }

    fn on_close(&self, _uri: &str) {
        // 处理文件关闭
    }
}

#[tokio::main]
async fn main() {
    let vfs = Arc::new(MemoryVfs::default());
    let service = WaeLanguageService { vfs };
    let server = LspServer::new(service);
    server.run().await;
}
