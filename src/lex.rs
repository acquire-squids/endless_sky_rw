use reporting::{Reportable, Span, Spanned};

use std::{error, fmt};

pub struct Lexer {
    source: String,
    source_id: usize,
    byte_offset: usize,
    start_byte_offset: usize,
    lookahead: Option<<Self as Iterator>::Item>,
    on_new_line: bool,
    spaces: IndentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IndentKind {
    Space,
    Tab,
    Unknown,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    Symbol,
    Indent,
    Newline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    MixedIndentation,
    UnclosedString,
    NonAsciiCharacter,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MixedIndentation => write!(f, "mixed indentation detected"),
            Self::UnclosedString => write!(f, "this string was never closed"),
            Self::NonAsciiCharacter => {
                write!(
                    f,
                    "only ascii characters are allowed in Endless Sky data files"
                )
            }
        }
    }
}

impl error::Error for Error {}

impl Reportable for Error {
    fn notes(&self) -> Vec<String> {
        match self {
            Self::MixedIndentation => {
                vec!["you should only use one of tabs or spaces when indenting, not both".to_string()]
            }
            Self::UnclosedString => vec![
                "the string terminated at the newline character, but you should close it anyway".to_string(),
            ],
            Self::NonAsciiCharacter => vec![
                "if this has changed since endless_sky_rw was written, the library needs to be updated".to_string(),
            ],
        }
    }
}

impl Lexer {
    #[must_use]
    pub const fn new(source_id: usize) -> Self {
        Self {
            source: String::new(),
            source_id,
            byte_offset: 0,
            start_byte_offset: 0,
            lookahead: None,
            on_new_line: true,
            spaces: IndentKind::Unknown,
        }
    }

    pub fn push_source(&mut self, source: &str) {
        self.source.push_str(source);
    }

    #[must_use]
    pub const fn source_id(&self) -> usize {
        self.source_id
    }

    #[must_use]
    pub fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        if self.lookahead.is_none() {
            self.lookahead = self.next();
        }

        self.lookahead.as_ref()
    }

    const fn at(&self) -> usize {
        self.byte_offset
    }

    const fn start(&self) -> usize {
        self.start_byte_offset
    }

    fn ahead(&self) -> Option<char> {
        self.source
            .get((self.at())..)
            .and_then(|text| text.chars().next())
    }

    fn advance(&mut self) {
        self.byte_offset = self.source.ceil_char_boundary(self.at() + 1);
    }

    const fn single_char_token(&self, kind: Token) -> Spanned<Token> {
        Spanned::new(kind, Span::new(self.source_id(), self.start(), self.at()))
    }
}

impl Lexer {
    fn indent(&mut self, kind: IndentKind) -> <Self as Iterator>::Item {
        let token = self.single_char_token(Token::Indent);

        if self.spaces != kind && !matches!(self.spaces, IndentKind::Unknown | IndentKind::Mixed) {
            self.spaces = IndentKind::Mixed;
            self.lookahead = Some(Ok(token));

            return Err(Spanned::new(
                Error::MixedIndentation,
                Span::new(self.source_id(), self.start(), self.at()),
            ));
        } else if matches!(self.spaces, IndentKind::Unknown) {
            self.spaces = kind;
        }

        Ok(token)
    }
}

impl Iterator for Lexer {
    type Item = Result<Spanned<Token>, Spanned<Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(lookahead) = self.lookahead.take() {
            return Some(lookahead);
        }

        while let Some(ch) = self.ahead() {
            self.start_byte_offset = self.at();

            self.advance();

            match ch {
                '\n' => {
                    self.on_new_line = true;

                    return Some(Ok(self.single_char_token(Token::Newline)));
                }
                ' ' if self.on_new_line => {
                    return Some(self.indent(IndentKind::Space));
                }
                '\t' if self.on_new_line => {
                    return Some(self.indent(IndentKind::Tab));
                }
                ' ' | '\t' => {}
                '#' => {
                    while let Some(n) = self.ahead()
                        && n != '\n'
                    {
                        self.advance();
                    }
                }
                '`' | '"' => {
                    self.on_new_line = false;

                    self.start_byte_offset = self.at();

                    while let Some(n) = self.ahead()
                        && n != '\n'
                        && n != ch
                    {
                        self.advance();
                    }

                    let token = Spanned::new(
                        Token::Symbol,
                        Span::new(self.source_id(), self.start(), self.at()),
                    );

                    if let Some(n) = self.ahead()
                        && n != ch
                    {
                        self.lookahead = Some(Ok(token));

                        return Some(Err(Spanned::new(
                            Error::UnclosedString,
                            Span::new(self.source_id(), self.start() - ch.len_utf8(), self.start()),
                        )));
                    }

                    self.advance();

                    return Some(Ok(token));
                }
                _ if ch.is_ascii() => {
                    self.on_new_line = false;

                    while let Some(n) = self.ahead()
                        && !n.is_ascii_whitespace()
                        && n.is_ascii()
                    {
                        self.advance();
                    }

                    return Some(Ok(Spanned::new(
                        Token::Symbol,
                        Span::new(self.source_id(), self.start(), self.at()),
                    )));
                }
                _ => {
                    self.on_new_line = false;

                    return Some(Err(Spanned::new(
                        Error::NonAsciiCharacter,
                        Span::new(self.source_id(), self.start(), self.at()),
                    )));
                }
            }
        }

        None
    }
}
