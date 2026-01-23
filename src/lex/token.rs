use crate::reporting::{Span, Spannable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    #[must_use] 
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    #[must_use] 
    pub fn lexeme<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.slice((self.span().start_as_usize())..(self.span().end_as_usize()))
    }

    #[must_use] 
    pub const fn kind(&self) -> TokenKind {
        self.kind
    }

    #[must_use] 
    pub const fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Symbol,
    Indent,
    Newline,
}
