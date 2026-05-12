#![allow(dead_code)]

use crate::lexer::expr::Expr;

#[derive(Debug)]
pub enum Statement {
    Expression(Expr),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statement::Expression(e) => write!(f, "(stmt {e})"),
        }
    }
}
