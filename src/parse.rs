pub mod error;

use self::error::{ParseError, ParseErrorKind};

use crate::data::{Data, Node, NodeIndex, SourceIndex};

use crate::lex::{
    Lexer,
    token::{Token, TokenKind},
};

use crate::reporting::Reportable;

use std::mem;

pub struct Parser {
    lexer: Lexer,
    errors: Vec<ParseError>,
    indentation: usize,
}

impl Parser {
    pub const fn new(source_index: SourceIndex) -> Self {
        Self {
            lexer: Lexer::new(source_index),
            errors: vec![],
            indentation: 0,
        }
    }

    const fn source_index(&self) -> SourceIndex {
        self.lexer.source_index()
    }

    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn take_errors(&mut self) -> Vec<ParseError> {
        mem::take(&mut self.errors)
    }
}

impl Parser {
    pub fn parse(&mut self, data: &mut Data) {
        while self.peek(data).is_some() {
            let node = self.node(data);
            data.push_root_node(self.source_index(), node);
        }
    }

    fn node(&mut self, data: &mut Data) -> NodeIndex {
        self.indentation(data);

        let current_indentation = self.indentation;

        let mut tokens = vec![];

        while let Some(token) = self.peek(data)
            && token.kind() == TokenKind::Symbol
        {
            tokens.push(
                self.advance(data)
                    .expect("Because we're currently peeking, we know we can advance"),
            );
        }

        let mut children = vec![];

        self.indentation(data);

        while self.peek(data).is_some() && self.indentation > current_indentation {
            let node = self.node(data);
            children.push(node);
            self.indentation(data);
        }

        if children.is_empty() {
            data.insert_node(Node::Some { tokens })
        } else {
            data.insert_node(Node::Parent { tokens, children })
        }
    }

    fn indentation(&mut self, data: &Data) {
        loop {
            match self.peek(data).map(super::lex::token::Token::kind) {
                None | Some(TokenKind::Symbol) => return,
                Some(TokenKind::Indent) => {
                    self.advance(data);
                    self.indentation += 1;
                }
                Some(TokenKind::Newline) => {
                    self.advance(data);
                    self.indentation = 0;
                }
            }
        }
    }

    fn lex_error(&mut self, data: &Data) {
        while let Some(Err(_)) = self.lexer.peek(data) {
            let Some(Err(lex_error)) = self.lexer.next(data) else {
                unreachable!()
            };

            self.error(ParseError::new(
                ParseErrorKind::LexError(lex_error),
                lex_error.span(),
            ));
        }
    }

    fn advance(&mut self, data: &Data) -> Option<Token> {
        self.lex_error(data);

        if let Some(Ok(token)) = self.lexer.next(data) {
            Some(token)
        } else {
            None
        }
    }

    fn peek(&mut self, data: &Data) -> Option<&Token> {
        self.lex_error(data);

        if let Some(Ok(token)) = self.lexer.peek(data) {
            Some(token)
        } else {
            None
        }
    }
}
