use crate::lex::error::LexError;
use crate::reporting::{Reportable, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    LexError(LexError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseError {
    kind: ParseErrorKind,
    span: Span,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Reportable<String, String> for ParseError {
    fn span(&self) -> Span {
        self.span
    }

    fn message(&self) -> Option<String> {
        match self.kind {
            ParseErrorKind::LexError(lex_error) => lex_error.message(),
        }
    }

    fn notes(&self) -> Vec<String> {
        match self.kind {
            ParseErrorKind::LexError(lex_error) => lex_error.notes(),
        }
    }
}
