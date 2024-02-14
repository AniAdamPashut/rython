use scanner::Tokens;
use crate::ast::Ast;

pub fn parse(tokens: Tokens) -> Ast {
    Ast::new(tokens)
}