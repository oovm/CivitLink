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
    DiagnosticSeverity, HoverInfo, SemanticAnalyzer, SemanticDiagnostic, SemanticResult, Symbol, SymbolKind, SymbolTable,
};

#[cfg(feature = "lsp")]
mod lang {
    use oak_core::language::{ElementType, Language, LanguageCategory, TokenType, UniversalElementRole, UniversalTokenRole};
    use std::hash::Hash;

    /// Valkyrie 语言的 token 类型。
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub enum ValkyrieToken {
        /// 输入流结束
        Eof,
        /// 关键字
        Keyword,
        /// 标识符
        Ident,
        /// 字符串字面量
        String,
        /// 数字字面量
        Number,
        /// 注释
        Comment,
        /// 标点符号
        Punct,
        /// 空白字符
        Whitespace,
        /// 未知
        Unknown,
    }

    impl TokenType for ValkyrieToken {
        type Role = UniversalTokenRole;

        const END_OF_STREAM: Self = Self::Eof;

        fn role(&self) -> Self::Role {
            match self {
                Self::Eof => UniversalTokenRole::Eof,
                Self::Keyword => UniversalTokenRole::Keyword,
                Self::Ident => UniversalTokenRole::Name,
                Self::String | Self::Number => UniversalTokenRole::Literal,
                Self::Comment => UniversalTokenRole::Comment,
                Self::Punct => UniversalTokenRole::Punctuation,
                Self::Whitespace => UniversalTokenRole::Whitespace,
                Self::Unknown => UniversalTokenRole::Error,
            }
        }
    }

    /// Valkyrie 语言的元素类型。
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub enum ValkyrieElement {
        /// 根节点
        Root,
        /// 命名空间
        Namespace,
        /// 函数定义
        Function,
        /// 变量绑定
        Binding,
        /// 引用
        Reference,
        /// 语句
        Statement,
        /// 表达式
        Expression,
        /// 调用
        Call,
        /// 类型标注
        Typing,
        /// 参数
        Parameter,
        /// 错误节点
        Error,
    }

    impl ElementType for ValkyrieElement {
        type Role = UniversalElementRole;

        fn role(&self) -> Self::Role {
            match self {
                Self::Root => UniversalElementRole::Root,
                Self::Namespace => UniversalElementRole::Container,
                Self::Function => UniversalElementRole::Definition,
                Self::Binding => UniversalElementRole::Binding,
                Self::Reference => UniversalElementRole::Reference,
                Self::Statement => UniversalElementRole::Statement,
                Self::Expression => UniversalElementRole::Expression,
                Self::Call => UniversalElementRole::Call,
                Self::Typing => UniversalElementRole::Typing,
                Self::Parameter => UniversalElementRole::Detail,
                Self::Error => UniversalElementRole::Error,
            }
        }
    }

    /// Valkyrie 语言定义。
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct ValkyrieLang;

    impl Language for ValkyrieLang {
        const NAME: &'static str = "valkyrie";
        const CATEGORY: LanguageCategory = LanguageCategory::Programming;
        type TokenType = ValkyrieToken;
        type ElementType = ValkyrieElement;
        type TypedRoot = ();
    }
}

#[cfg(feature = "lsp")]
use {
    core::range::Range, futures::Future, oak_core::source::Source, oak_core::tree::RedNode, oak_lsp::service::LanguageService,
    oak_lsp::types::Hover as LspHover, oak_vfs::Vfs, std::sync::Mutex,
};

#[cfg(feature = "lsp")]
pub use lang::ValkyrieLang;

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
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::new(), analyzer: Mutex::new(SemanticAnalyzer::new()) }
    }

    /// 提供自动补全
    pub fn completion(&self, _uri: &str, prefix: &str) -> Vec<CompletionEntry> {
        let provider = CompletionProvider::new();
        let source = String::new();
        provider.complete(prefix, &source)
    }

    /// 分析指定 URI 的源码，返回语义分析结果。
    pub fn analyze_source(&self, uri: &str) -> Option<SemanticResult> {
        let src = self.vfs.get_source(uri)?;
        let text = src.get_text_in(Range { start: 0, end: src.length() }).into_owned();
        let mut analyzer = self.analyzer.lock().ok()?;
        Some(analyzer.analyze(&text))
    }
}

#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for GgLanguageService<V> {
    type Lang = ValkyrieLang;
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
        let src = self.vfs.get_source(uri).map(|s| s.get_text_in(Range { start: 0, end: s.length() }).into_owned());
        let hover_result = self.analyzer.lock().ok().and_then(|analyzer| {
            let source = src.as_ref()?;
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
            analyzer.get_hover_info(line, col, source)
        });
        async move {
            let hover_info = hover_result?;
            Some(LspHover { contents: hover_info.contents, range: hover_info.range.map(|(s, e)| Range { start: s, end: e }) })
        }
    }
}
