#![warn(missing_docs)]
#![feature(new_range_api)]

//! GG 引擎 LSP 模块
//! 提供语言服务器相关功能

mod kind;

pub mod completion;
pub mod semantic;

#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "oak-pretty-print")]
pub mod formatter;

pub use completion::{CompletionEntry, CompletionItemKind, CompletionProvider};
pub use semantic::{
    DiagnosticSeverity, HoverInfo, SemanticAnalyzer, SemanticDiagnostic, SemanticResult, Symbol,
    SymbolKind, SymbolTable,
};

#[cfg(feature = "lsp")]
use {
    core::range::Range,
    futures::Future,
    oak_core::tree::RedNode,
    oak_lsp::service::LanguageService,
    oak_lsp::types::Hover as LspHover,
    oak_vfs::Vfs,
    std::sync::Mutex,
};

#[cfg(feature = "lsp")]
/// Language service implementation for GG Engine.
pub struct GgLanguageService<V: Vfs> {
    /// 虚拟文件系统
    vfs: V,
    /// 工作区管理器
    workspace: oak_lsp::workspace::WorkspaceManager,
    /// 语义分析器
    analyzer: Mutex<SemanticAnalyzer>,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> GgLanguageService<V> {
    /// Creates a new `GgLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self {
            vfs,
            workspace: oak_lsp::workspace::WorkspaceManager::new(),
            analyzer: Mutex::new(SemanticAnalyzer::new()),
        }
    }

    /// 提供自动补全
    pub fn completion(&self, _uri: &str, prefix: &str) -> Vec<CompletionEntry> {
        let provider = CompletionProvider::new();
        let source = String::new();
        provider.complete(prefix, &source)
    }

    /// 分析指定 URI 的源码，返回语义分析结果。
    pub fn analyze_source(&self, uri: &str) -> Option<SemanticResult> {
        let source = self.vfs.get_source(uri)?.read().to_string();
        let mut analyzer = self.analyzer.lock().ok()?;
        Some(analyzer.analyze(&source))
    }
}

#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for GgLanguageService<V> {
    type Lang = ();
    type Vfs = V;
    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }
    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }
    fn get_root(&self, _uri: &str) -> impl Future<Output = Option<RedNode<'_, Self::Lang>>> + Send + '_ {
        async move { None }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Option<LspHover>> + Send + '_ {
        let source = self.vfs.get_source(uri).map(|s| s.read().to_string());
        let analyzer = self.analyzer.lock().ok();
        async move {
            let source = source?;
            let analyzer = analyzer?;
            let start = range.start;
            let mut line = 0;
            let mut col = 0;
            for (i, ch) in source.char_indices() {
                if i >= start {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                }
                else {
                    col += 1;
                }
            }
            let hover_info = analyzer.get_hover_info(line, col, &source)?;
            Some(LspHover {
                contents: hover_info.contents,
                range: hover_info.range.map(|(s, e)| core::range::Range::new(s, e)),
            })
        }
    }
}
