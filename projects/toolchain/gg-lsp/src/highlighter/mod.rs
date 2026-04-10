//! Syntax highlighter module.

use oak_core::errors::ParseResult;
use oak_highlight::{
    HighlightResult,
    highlighter::{Highlighter, OakHighlighter},
    themes::Theme,
};

/// A syntax highlighter for GG Engine source code.
pub struct GgHighlighter {
    /// Whether to use parser-based highlighting for enhanced accuracy
    pub use_parser: bool,
}

impl Default for GgHighlighter {
    fn default() -> Self {
        Self { use_parser: true }
    }
}

impl GgHighlighter {
    /// Creates a new GG Engine highlighter.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Highlighter for GgHighlighter {
    fn highlight<'a>(&self, source: &'a str, language: &str, theme: Theme) -> ParseResult<HighlightResult<'a>> {
        let highlighter = OakHighlighter::new().theme(theme);

        if self.use_parser {
            // TODO: Implement parser-based highlighting for GG Engine languages
            highlighter.highlight(source, language, theme)
        }
        else {
            highlighter.highlight(source, language, theme)
        }
    }
}
