use scanner::prelude::*;

struct AstNode {
    data: Token,
    children: Vec<Box<AstNode>>,
}

impl AstNode {
    pub fn new(
        data: Token,
        children: Vec<Box<Self>>
    ) -> Self {
        AstNode {
            data,
            children
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }
}

pub struct Ast {
    tokens: Tokens,
    lines: Vec<AstNode>,
}

impl Ast {
    pub fn new(
        tokens: Tokens
    ) -> Self {
        Ast {
            tokens,
            lines: Vec::new()
        }
    }
}

