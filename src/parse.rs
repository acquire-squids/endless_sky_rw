use crate::{
    data::{Data, Node, NodeIndex, SourceIndex},
    lex::{self, Lexer, Token},
    reporting::{Reportable, Spanned},
};

use std::{error, fmt, mem};

pub struct Parser {
    lexer: Lexer,
    source_index: SourceIndex,
    errors: Vec<Spanned<Error>>,
    indentation: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    Lex(lex::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(lex_error) => write!(f, "{lex_error}"),
        }
    }
}

impl error::Error for Error {}

impl Reportable for Error {
    fn notes(&self) -> Vec<String> {
        match self {
            Self::Lex(lex_error) => lex_error.notes(),
        }
    }
}

impl Parser {
    pub fn new(source_index: SourceIndex, source: &str) -> Self {
        let mut me = Self {
            lexer: Lexer::new(source_index.index()),
            source_index,
            errors: vec![],
            indentation: 0,
        };

        me.lexer.push_source(source);

        me
    }

    const fn source_index(&self) -> SourceIndex {
        self.source_index
    }

    fn error(&mut self, error: Spanned<Error>) {
        self.errors.push(error);
    }
}

impl Parser {
    pub fn parse(&mut self, data: &mut Data) -> Result<(), Vec<Spanned<Error>>> {
        self.indentation();

        while self.peek().is_some() {
            let node = self.node(data);
            data.push_root_node(self.source_index(), node);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    fn node(&mut self, data: &mut Data) -> NodeIndex {
        let current_indentation = self.indentation;

        let mut tokens = vec![];

        while let Some(token) = self.peek()
            && token.kind() == &Token::Symbol
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
            match self.peek().map(Spanned::kind) {
                None | Some(Token::Symbol) => return,
                Some(Token::Indent) => {
                    self.advance();
                    self.indentation += 1;
                }
                Some(Token::Newline) => {
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
            self.error(lex_error.transmute(Error::Lex));
        }
    }

    fn advance(&mut self) -> Option<Spanned<Token>> {
        self.lex_error();

        self.lexer.next().and_then(Result::ok)
    }

    fn peek(&mut self) -> Option<&Spanned<Token>> {
        self.lex_error();

        if let Some(Ok(token)) = self.lexer.peek() {
            Some(token)
        } else {
            None
        }
    }
}
