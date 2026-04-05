pub mod error;

use self::error::{ParseError, ParseErrorKind};

use crate::data::{Data, Node, NodeIndex, SourceIndex};

use crate::lex::{
    Lexer,
    token::{Token, TokenKind},
};

use crate::reporting::Reportable;

use std::{iter::Peekable, mem};

pub struct Parser {
    lexer: Peekable<Lexer>,
    source_index: SourceIndex,
    errors: Vec<ParseError>,
    indentation: usize,
}

impl Parser {
    pub fn new(source_index: SourceIndex, source: String) -> Self {
        Self {
            lexer: Lexer::new(source).peekable(),
            source_index,
            errors: vec![],
            indentation: 0,
        }
    }

    const fn source_index(&self) -> SourceIndex {
        self.source_index
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
        self.indentation();

        while self.peek().is_some() {
            let node = self.node(data);
            data.push_root_node(self.source_index(), node);
        }
    }

    fn node(&mut self, data: &mut Data) -> NodeIndex {
        let current_indentation = self.indentation;

        let mut tokens = vec![];

        while let Some(token) = self.peek()
            && token.kind() == TokenKind::Symbol
        {
            tokens.push(
                self.advance()
                    .expect("Because we're currently peeking, we know we can advance"),
            );
        }

        self.indentation();

        if self.indentation > current_indentation && self.peek().is_some() {
            let mut children = vec![];

            while self.peek().is_some() && self.indentation > current_indentation {
                let node = self.node(data);
                children.push(node);
                self.indentation();
            }

            data.insert_node(Node::Parent { tokens, children })
        } else {
            data.insert_node(Node::Some { tokens })
        }
    }

    fn indentation(&mut self) {
        loop {
            match self.peek().map(super::lex::token::Token::kind) {
                None | Some(TokenKind::Symbol) => return,
                Some(TokenKind::Indent) => {
                    self.advance();
                    self.indentation += 1;
                }
                Some(TokenKind::Newline) => {
                    self.advance();
                    self.indentation = 0;
                }
            }
        }
    }

    fn lex_error(&mut self) {
        while let Some(Err(_)) = self.lexer.peek()
            && let Some(Err(lex_error)) = self.lexer.next()
        {
            self.error(ParseError::new(
                ParseErrorKind::LexError(lex_error),
                lex_error.span(),
            ));
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.lex_error();

        self.lexer.next().and_then(Result::ok)
    }

    fn peek(&mut self) -> Option<&Token> {
        self.lex_error();

        if let Some(Ok(token)) = self.lexer.peek() {
            Some(token)
        } else {
            None
        }
    }
}
