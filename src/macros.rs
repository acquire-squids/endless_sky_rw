#[macro_export]
macro_rules! node_path_iter {
    (
        @nested $data:expr => ($source:expr, $node:expr); $first:pat $(=> $($tail:tt)+)?
    ) => {
        {
            let data: &$crate::Data = $data;

            let source_index: $crate::SourceIndex = $source;
            let node_index: $crate::NodeIndex = $node;

            data.filter_children(source_index, node_index, |source_index, tokens| {
                matches!(
                    tokens.first().map(|&token| data.get_lexeme(source_index, token)),
                    Some(Some($first))
                )
            })
            $(.flat_map(|&node| {
                $crate::node_path_iter!(@nested data => node; $($tail)+)
            }))?
        }
    };

    (
        $data:expr => ($source:expr, $node:expr); $first:pat $(=> $($tail:tt)+)?
    ) => {
        $crate::node_path_iter!(@nested $data => ($source, $node); $first $(=> $($tail)+)?)
    };

    (
        $data:expr; $first:pat $(=> $($tail:tt)+)?
    ) => {
        {
            let data: &$crate::Data = $data;

            data.filter(|source_index, tokens| {
                matches!(
                    tokens.first().map(|&token| data.get_lexeme(source_index, token)),
                    Some(Some($first))
                )
            })
            $(.flat_map(|&node| {
                $crate::node_path_iter!(@nested $data => node; $($tail)+)
            }))?
        }
    };
}

#[macro_export]
macro_rules! tree_from_tokens {
    (
        @parent $node:expr,
        $data:expr; $source_index:expr =>
        $(
            : $token:expr $(, $tokens:expr)* ;
            $({ $($tail:tt)+ })?
        )+
    ) => {
        {
            let mut data: &mut $crate::Data = $data;

            let source_index: $crate::SourceIndex = $source_index;

            let parent: $crate::NodeIndex = $node;

            $(
                let node = $crate::tree_from_tokens!(
                    data; source_index =>
                    : $token $(, $tokens)* ;
                    $({ $($tail)+ })?
                );

                data.push_child(parent, node);
            )+

            node
        }
    };

    (
        $data:expr; $source_index:expr  =>
        : $token:expr $(, $tokens:expr)* ;
        $({ $($tail:tt)+ })?
    ) => {
        {
            let mut data: &mut $crate::Data = $data;

            let source_index: $crate::SourceIndex = $source_index;

            if data.get_source(source_index).is_none() {
                data.error_node()
            } else {
                let node = data.insert_node(Node::Some { tokens: vec![], });

                let span = data.push_source(source_index, $token).unwrap();
                data.push_token(node, Token::new(TokenKind::Symbol, Span::new(span.0, span.1)));

                $(
                    let span = data.push_source(source_index, $tokens).unwrap();
                    data.push_token(node, Token::new(TokenKind::Symbol, Span::new(span.0, span.1)));
                )*

                $(
                    $crate::tree_from_tokens!(@parent node, data; source_index => $($tail)+);
                )?

                node
            }
        }
    };
}
