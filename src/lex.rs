pub mod error;
pub mod token;

use self::{
    error::{LexError, LexErrorKind},
    token::{Token, TokenKind},
};

use crate::reporting::Span;

pub type LexItem = Result<Token, LexError>;

pub struct Lexer {
    source: String,
    lookahead: Option<LexItem>,
    on_new_line: bool,
    byte_offset: usize,
    spaces: IndentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IndentKind {
    Space,
    Tab,
    Unknown,
    Mixed,
}

impl Lexer {
    pub const fn new(source: String) -> Self {
        Self {
            source,
            lookahead: None,
            on_new_line: true,
            byte_offset: 0,
            spaces: IndentKind::Unknown,
        }
    }

    pub const fn source(&self) -> &str {
        self.source.as_str()
    }

    const fn at(&self) -> usize {
        self.byte_offset
    }

    fn ahead(&self) -> Option<char> {
        self.source()
            .get((self.at())..)
            .and_then(|text| text.chars().next())
    }

    fn advance(&mut self) {
        if let Some(ch) = self.ahead() {
            self.byte_offset += ch.len_utf8();
        }
    }
}

impl Lexer {
    fn indent(&mut self, start: usize, kind: IndentKind) -> LexItem {
        let token = Token::new(TokenKind::Indent, Span::new(start, self.at()));

        if self.spaces != kind && !matches!(self.spaces, IndentKind::Unknown | IndentKind::Mixed) {
            self.spaces = IndentKind::Mixed;
            self.lookahead = Some(Ok(token));

            return Err(LexError::new(
                LexErrorKind::MixedIndentation,
                Span::new(start, self.at()),
            ));
        } else if matches!(self.spaces, IndentKind::Unknown) {
            self.spaces = kind;
        }

        Ok(token)
    }
}

impl Iterator for Lexer {
    type Item = LexItem;

    fn next(&mut self) -> Option<LexItem> {
        if let Some(lookahead) = self.lookahead.take() {
            return Some(lookahead);
        }

        while let Some(c) = self.ahead() {
            let start = self.at();

            self.advance();

            match c {
                '\n' => {
                    self.on_new_line = true;

                    return Some(Ok(Token::new(
                        TokenKind::Newline,
                        Span::new(start, self.byte_offset),
                    )));
                }
                ' ' if self.on_new_line => {
                    return Some(self.indent(start, IndentKind::Space));
                }
                '\t' if self.on_new_line => {
                    return Some(self.indent(start, IndentKind::Tab));
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

                    let after_quote = self.at();

                    while let Some(n) = self.ahead()
                        && n != '\n'
                        && n != c
                    {
                        self.advance();
                    }

                    let token = Token::new(TokenKind::Symbol, Span::new(after_quote, self.at()));

                    if let Some(n) = self.ahead()
                        && n != c
                    {
                        self.lookahead = Some(Ok(token));

                        return Some(Err(LexError::new(
                            LexErrorKind::UnclosedString,
                            Span::new(start, after_quote),
                        )));
                    }

                    self.advance();

                    return Some(Ok(token));
                }
                _ if c.is_ascii() => {
                    self.on_new_line = false;

                    while let Some(n) = self.ahead()
                        && !n.is_ascii_whitespace()
                        && n.is_ascii()
                    {
                        self.advance();
                    }

                    return Some(Ok(Token::new(
                        TokenKind::Symbol,
                        Span::new(start, self.at()),
                    )));
                }
                _ => {
                    self.on_new_line = false;

                    return Some(Err(LexError::new(
                        LexErrorKind::NonAsciiCharacter,
                        Span::new(start, self.at()),
                    )));
                }
            }
        }

        None
    }
}
