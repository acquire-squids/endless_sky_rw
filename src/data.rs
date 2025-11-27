use crate::arena::{self, Arena};
use crate::lex::token::Token;

use std::{
    collections::HashSet,
    fmt::{self, Write},
    mem,
    num::ParseFloatError,
};

pub struct Data {
    nodes: Arena<Node>,
    sources: Arena<String>,
    root_nodes: Vec<(SourceIndex, NodeIndex)>,
    error_node: NodeIndex,
}

pub enum Node {
    Some {
        tokens: Vec<Token>,
    },
    Parent {
        tokens: Vec<Token>,
        children: Vec<NodeIndex>,
    },
    Error,
}

arena::arena_index! {
    pub NodeIndex
}

arena::arena_index! {
    pub SourceIndex
}

impl Default for Data {
    fn default() -> Self {
        let mut nodes = Arena::default();
        let error_node = nodes.insert(Node::Error).into();

        Self {
            nodes,
            sources: Arena::default(),
            root_nodes: vec![],
            error_node,
        }
    }
}

impl Data {
    pub fn error_node(&self) -> NodeIndex {
        self.error_node
    }

    pub fn push_root_node(&mut self, source_index: SourceIndex, node_index: NodeIndex) {
        self.root_nodes.push((source_index, node_index));
    }

    pub fn root_nodes(&self) -> &[(SourceIndex, NodeIndex)] {
        self.root_nodes.as_slice()
    }

    pub fn insert_node(&mut self, node: Node) -> NodeIndex {
        self.nodes.insert(node).into()
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&Node> {
        self.nodes.get(index.into())
    }

    pub fn get_mut_node(&mut self, index: NodeIndex) -> Option<&mut Node> {
        self.nodes.get_mut(index.into())
    }

    pub fn push_child(&mut self, node_index: NodeIndex, child_index: NodeIndex) {
        match self.get_mut_node(node_index) {
            None => {}
            Some(Node::Error) => {}
            Some(some @ Node::Some { .. }) => {
                let Node::Some { mut tokens } = mem::replace(
                    some,
                    Node::Parent {
                        tokens: vec![],
                        children: vec![child_index],
                    },
                ) else {
                    unreachable!()
                };

                let Some(Node::Parent {
                    tokens: parent_tokens,
                    ..
                }) = self.get_mut_node(node_index)
                else {
                    unreachable!()
                };

                parent_tokens.append(&mut tokens);
            }
            Some(Node::Parent { children, .. }) => {
                children.push(child_index);
            }
        }
    }

    pub fn get_children(&self, node_index: NodeIndex) -> Option<&[NodeIndex]> {
        match self.get_node(node_index) {
            None => None,
            Some(Node::Error) => None,
            Some(Node::Some { .. }) => None,
            Some(Node::Parent { children, .. }) => Some(children.as_slice()),
        }
    }

    pub fn get_mut_children(&mut self, node_index: NodeIndex) -> Option<&mut [NodeIndex]> {
        match self.get_mut_node(node_index) {
            None => None,
            Some(Node::Error) => None,
            Some(Node::Some { .. }) => None,
            Some(Node::Parent { children, .. }) => Some(children),
        }
    }

    pub fn push_token(&mut self, node_index: NodeIndex, token: Token) {
        match self.get_mut_node(node_index) {
            None => {}
            Some(Node::Error) => {}
            Some(Node::Some { tokens } | Node::Parent { tokens, .. }) => tokens.push(token),
        }
    }

    pub fn get_tokens(&self, node_index: NodeIndex) -> Option<&[Token]> {
        match self.get_node(node_index) {
            None => None,
            Some(Node::Error) => None,
            Some(Node::Some { tokens } | Node::Parent { tokens, .. }) => Some(tokens.as_slice()),
        }
    }

    pub fn get_mut_tokens(&mut self, node_index: NodeIndex) -> Option<&mut [Token]> {
        match self.get_mut_node(node_index) {
            None => None,
            Some(Node::Error) => None,
            Some(Node::Some { tokens } | Node::Parent { tokens, .. }) => Some(tokens),
        }
    }

    pub fn insert_source(&mut self, source: String) -> SourceIndex {
        self.sources.insert(source).into()
    }

    pub fn get_source(&self, index: SourceIndex) -> Option<&str> {
        self.sources.get(index.into()).map(|s| s.as_str())
    }

    pub fn push_source<T: fmt::Display>(
        &mut self,
        index: SourceIndex,
        append: T,
    ) -> Option<(usize, usize)> {
        if let Some(source) = self.sources.get_mut(index.into()) {
            let source_len = source.len();

            let _ = write!(source, "{}", append);

            Some((source_len, source.len()))
        } else {
            None
        }
    }

    pub fn get_lexeme(&self, source_index: SourceIndex, token: Token) -> Option<&str> {
        self.get_source(source_index).and_then(|s| token.lexeme(s))
    }

    pub fn try_get_number(
        &self,
        source_index: SourceIndex,
        token: Token,
    ) -> Option<Result<f64, ParseFloatError>> {
        self.get_lexeme(source_index, token)
            .map(|l| l.parse::<f64>())
    }

    pub fn filter<P>(&self, mut predicate: P) -> impl Iterator<Item = (SourceIndex, NodeIndex)>
    where
        P: FnMut(SourceIndex, &[Token]) -> bool,
    {
        self.root_nodes()
            .iter()
            .filter(move |(source_index, root)| {
                matches!(self.get_tokens(*root), Some(tokens) if predicate(*source_index, tokens))
            })
            .copied()
    }

    pub fn filter_children<P>(
        &self,
        source_index: SourceIndex,
        node_index: NodeIndex,
        mut predicate: P,
    ) -> impl Iterator<Item = NodeIndex>
    where
        P: FnMut(SourceIndex, &[Token]) -> bool,
    {
        self.get_children(node_index)
            .unwrap_or_default()
            .iter()
            .filter(move |child| {
                matches!(self.get_tokens(**child), Some(tokens) if predicate(source_index, tokens))
            })
            .copied()
    }
}

impl Data {
    pub fn write(
        &self,
        output: &mut String,
        source_index: SourceIndex,
        node_index: NodeIndex,
        indentation: usize,
    ) -> fmt::Result {
        let mut infinity_prevention = HashSet::new();

        self.write_recursive(
            output,
            source_index,
            node_index,
            indentation,
            &mut infinity_prevention,
        )
    }

    pub fn write_root_nodes(
        &self,
        output: &mut String,
        root_nodes: &[(SourceIndex, NodeIndex)],
    ) -> fmt::Result {
        let mut infinity_prevention = HashSet::new();

        for (source_index, node_index) in root_nodes {
            self.write_recursive(
                output,
                *source_index,
                *node_index,
                0,
                &mut infinity_prevention,
            )?;

            write!(output, "\n\n\n\n")?;
        }

        Ok(())
    }

    fn write_recursive(
        &self,
        output: &mut String,
        source_index: SourceIndex,
        node_index: NodeIndex,
        indentation: usize,
        infinity_prevention: &mut HashSet<(SourceIndex, NodeIndex)>,
    ) -> fmt::Result {
        // if the pair is already in the `HashSet`, it would lead to infinite recursion
        // it's okay to silently error here because the node was already written
        if infinity_prevention.contains(&(source_index, node_index)) {
            return Ok(());
        }

        infinity_prevention.insert((source_index, node_index));

        // TODO: don't silently error
        if let Some(Node::Error) = self.get_node(node_index) {
            return Ok(());
        }

        if let Some(tokens) = self.get_tokens(node_index) {
            for (i, token) in tokens.iter().enumerate() {
                if let Some(lexeme) =
                    token.lexeme(self.get_source(source_index).unwrap_or_default())
                    && !lexeme.is_empty()
                {
                    if !lexeme.contains('"') {
                        write!(output, "\"{lexeme}\"")?;
                    } else if !lexeme.contains('`') {
                        write!(output, "`{lexeme}`")?;
                    } else {
                        write!(output, "{lexeme}")?;
                    }

                    if i < tokens.len() - 1 {
                        write!(output, " ")?;
                    }
                }
            }

            if let Some(children) = self.get_children(node_index)
                && !children.is_empty()
            {
                let indentation = indentation + 1;

                for child in children {
                    write!(output, "\n{}", "\t".repeat(indentation))?;

                    self.write_recursive(
                        output,
                        source_index,
                        *child,
                        indentation,
                        infinity_prevention,
                    )?;
                }
            }
        }

        Ok(())
    }
}
