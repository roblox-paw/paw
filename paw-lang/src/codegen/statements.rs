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
    Block(Vec<Statement>),
    Variable {
        name: Token,
        init: Option<Expr>,
        kind: VarKind,
    },
    If {
        predicate: Expr,
        then: Box<Statement>,
        else_block: Option<Box<Statement>>,
    },
    Loop(Box<Statement>),
    While {
        condition: Expr,
        body: Box<Statement>,
    },
    Return(Option<Expr>),
    Continue,
    Break,
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
            Statement::Block(statements) => {
                write!(f, "(block")?;
                for s in statements {
                    write!(f, " {s}")?;
                }
                write!(f, ")")
            }

            Statement::Variable { name, init, kind } => match init {
                Some(e) => write!(f, "({kind} {} {e})", name.lexeme),
                None => write!(f, "({kind} {})", name.lexeme),
            }
            
            Statement::If { predicate, then, else_block } => match else_block {
                Some(e) => write!(f, "(if {predicate} {then} {e})"),
                None => write!(f, "(if {predicate} {then})"),
            }
            
            Statement::Loop(b) => write!(f, "(loop {b})"),
            Statement::While { condition, body } => {
                write!(f, "(while {condition} {body})")
            }

            Statement::Return(v) => match v {
                Some(e) => write!(f, "(return {e})"),
                None => write!(f, "(return)"),
            }

            Statement::Continue => write!(f, "(continue)"),
            Statement::Break => write!(f, "(break)"),
        }
    }
}
