#![warn(missing_docs)]

//! GG 引擎 LSP 模块
//! 提供语言服务器相关功能

mod kind;

#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "oak-pretty-print")]
pub mod formatter;

use core::range::Range;
use oak_core::tree::RedNode;

#[cfg(feature = "lsp")]
use {futures::Future, oak_lsp::service::LanguageService, oak_lsp::types::Hover as LspHover, oak_vfs::Vfs};

#[cfg(feature = "lsp")]
/// Language service implementation for GG Engine.
pub struct GgLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> GgLanguageService<V> {
    /// Creates a new `GgLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::new() }
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
    fn hover(&self, _uri: &str, _range: Range<usize>) -> impl Future<Output = Option<LspHover>> + Send + '_ {
        async move { None }
    }
};
