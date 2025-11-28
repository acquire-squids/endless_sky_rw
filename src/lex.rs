pub mod error;
pub mod token;

use self::{
    error::{LexError, LexErrorKind},
    token::{Token, TokenKind},
};

use crate::data::{Data, SourceIndex};
use crate::reporting::Span;

type LexItem = Result<Token, LexError>;

pub struct Lexer {
    source_index: SourceIndex,
    lookahead: Option<LexItem>,
    on_new_line: bool,
    byte_offset: usize,
    spaces: IndentKind,
}

enum IndentKind {
    Space,
    Tab,
    Unknown,
    Mixed,
}

impl Lexer {
    pub fn new(source_index: SourceIndex) -> Self {
        Self {
            source_index,
            lookahead: None,
            on_new_line: true,
            byte_offset: 0,
            spaces: IndentKind::Unknown,
        }
    }

    pub fn source_index(&self) -> SourceIndex {
        self.source_index
    }
}

impl Lexer {
    pub fn peek(&mut self, data: &Data) -> Option<&LexItem> {
        if self.lookahead.is_none() {
            self.lookahead = self.advance(data);
        }

        self.lookahead.as_ref()
    }

    pub fn next(&mut self, data: &Data) -> Option<LexItem> {
        self.advance(data)
    }

    fn advance(&mut self, data: &Data) -> Option<LexItem> {
        if let Some(lookahead) = self.lookahead.take() {
            return Some(lookahead);
        }

        while let Some(c) = data.get_source(self.source_index()).unwrap()[(self.byte_offset)..]
            .chars()
            .next()
        {
            let start = self.byte_offset;

            self.byte_offset += c.len_utf8();

            match c {
                '\n' => {
                    self.on_new_line = true;

                    return Some(Ok(Token::new(
                        TokenKind::Newline,
                        Span::new(start, self.byte_offset),
                    )));
                }
                ' ' if self.on_new_line => {
                    let token = Token::new(TokenKind::Indent, Span::new(start, self.byte_offset));

                    if let IndentKind::Tab = self.spaces {
                        self.spaces = IndentKind::Mixed;
                        self.lookahead = Some(Ok(token));

                        return Some(Err(LexError::new(
                            LexErrorKind::MixedIndentation,
                            Span::new(start, self.byte_offset),
                        )));
                    } else if let IndentKind::Unknown = self.spaces {
                        self.spaces = IndentKind::Space;
                    }

                    return Some(Ok(token));
                }
                '\t' if self.on_new_line => {
                    let token = Token::new(TokenKind::Indent, Span::new(start, self.byte_offset));

                    if let IndentKind::Space = self.spaces {
                        self.spaces = IndentKind::Mixed;
                        self.lookahead = Some(Ok(token));

                        return Some(Err(LexError::new(
                            LexErrorKind::MixedIndentation,
                            Span::new(start, self.byte_offset),
                        )));
                    } else if let IndentKind::Unknown = self.spaces {
                        self.spaces = IndentKind::Tab;
                    }

                    return Some(Ok(token));
                }
                ' ' | '\t' => {}
                '#' => {
                    while let Some(n) = data.get_source(self.source_index()).unwrap()
                        [(self.byte_offset)..]
                        .chars()
                        .next()
                        && n != '\n'
                    {
                        self.byte_offset += n.len_utf8();
                    }
                }
                '`' | '"' => {
                    self.on_new_line = false;

                    let after_quote = self.byte_offset;

                    while let Some(n) = data.get_source(self.source_index()).unwrap()
                        [(self.byte_offset)..]
                        .chars()
                        .next()
                        && n != '\n'
                        && n != c
                    {
                        self.byte_offset += n.len_utf8();
                    }

                    let token =
                        Token::new(TokenKind::Symbol, Span::new(after_quote, self.byte_offset));

                    if !matches!(data.get_source(self.source_index()).unwrap()[(self.byte_offset)..].chars().next(), Some(n) if n == c)
                    {
                        self.lookahead = Some(Ok(token));

                        return Some(Err(LexError::new(
                            LexErrorKind::UnclosedString,
                            Span::new(start, after_quote),
                        )));
                    } else {
                        self.byte_offset += c.len_utf8();

                        return Some(Ok(token));
                    }
                }
                _ if c.is_ascii() => {
                    self.on_new_line = false;

                    while let Some(n) = data.get_source(self.source_index()).unwrap()
                        [(self.byte_offset)..]
                        .chars()
                        .next()
                        && !n.is_ascii_whitespace()
                        && n.is_ascii()
                    {
                        self.byte_offset += n.len_utf8();
                    }

                    return Some(Ok(Token::new(
                        TokenKind::Symbol,
                        Span::new(start, self.byte_offset),
                    )));
                }
                _ => {
                    self.on_new_line = false;

                    return Some(Err(LexError::new(
                        LexErrorKind::NonAsciiCharacter,
                        Span::new(start, self.byte_offset),
                    )));
                }
            }
        }

        None
    }
}
