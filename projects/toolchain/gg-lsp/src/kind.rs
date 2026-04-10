//! Syntax kinds module.

/// Syntax kind for GG Engine.
pub enum SyntaxKind {
    /// Unknown syntax kind.
    Unknown,
    /// Identifier.
    Ident,
    /// Keyword.
    Keyword,
    /// String literal.
    String,
    /// Number literal.
    Number,
    /// Comment.
    Comment,
    /// Punctuation.
    Punct,
    /// Whitespace.
    Whitespace,
}
