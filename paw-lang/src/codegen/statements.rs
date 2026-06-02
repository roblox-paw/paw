#![allow(dead_code)]
use crate::lexer::{Token, expr::Expr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarKind {
    Let,
    Const,
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expr),
    Variable {
        name: Token,
        init: Option<Expr>,
        kind: VarKind,
    },
}

impl std::fmt::Display for VarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VarKind::Let => write!(f, "let"),
            VarKind::Const => write!(f, "const"),
        }
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Statement::Expression(e) => write!(f, "(stmt {e})"),
            Statement::Variable { name, init, kind } => match init {
                Some(e) => write!(f, "({kind} {} {e})", name.lexeme),
                None => write!(f, "({kind} {})", name.lexeme),
            }
        }
    }
}
