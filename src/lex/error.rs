use crate::reporting::{Reportable, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexErrorKind {
    MixedIndentation,
    UnclosedString,
    NonAsciiCharacter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LexError {
    kind: LexErrorKind,
    span: Span,
}

impl LexError {
    pub fn new(kind: LexErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl Reportable<String, String> for LexError {
    fn span(&self) -> Span {
        self.span
    }

    fn message(&self) -> Option<String> {
        Some(
            match self.kind {
                LexErrorKind::MixedIndentation => "Mixed indentation detected",
                LexErrorKind::UnclosedString => "This string was never closed",
                LexErrorKind::NonAsciiCharacter => {
                    "Only ASCII characters are allowed in Endless Sky data files"
                }
            }
            .to_owned(),
        )
    }

    fn notes(&self) -> Vec<String> {
        match self.kind {
            LexErrorKind::MixedIndentation => vec!["You should only use one of tabs or spaces when indenting, not both".to_owned()],
            LexErrorKind::UnclosedString => vec!["The string terminated at the newline character, but you should close it anyway".to_owned()],
            LexErrorKind::NonAsciiCharacter => vec!["If this has changed since Endless Sky RW was written, the library needs to be updated".to_owned()],
        }
    }
}
