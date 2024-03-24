use scanner::prelude::*;
use crate::ast::Ast;

pub enum PyValue {
    Number(f64),
    String(String),
    None,W
}

pub fn parse(tokens: Tokens) -> Ast {
    Ast::new(tokens)
}