use super::*;

#[derive(Debug)]
pub enum Expr {
    Literal(LiteralValue),
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Literal(v) => write!(f, "{v}"),
            Expr::Binary { left, operator, right } => {
                write!(f, "({} {left} {right})", operator.lexeme)
            }
            Expr::Unary { operator, right } => {
                write!(f, "({} {right})", operator.lexeme)
            }
            Expr::Grouping(e) => write!(f, "(group {e})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::scanner::Scanner;
    use crate::lexer::parser::Parser;

    fn ast(src: &str) -> String {
        let tokens = Scanner::new(src).scan_tokens().expect("scan failed");
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        match &stmts[0] {
            crate::codegen::statements::Statement::Expression(e) => e.to_string(),
        }
    }

    #[test]
    fn literal_num() {
        assert_eq!(ast("42"), "42");
    }

    #[test]
    fn binary_add() {
        assert_eq!(ast("1 + 2"), "(+ 1 2)");
    }

    #[test]
    fn binary_neq() {
        assert_eq!(ast("1 != 2"), "(!= 1 2)");
    }

    #[test]
    fn unary_bang() {
        assert_eq!(ast("!true"), "(! true)");
    }

    #[test]
    fn grouping() {
        assert_eq!(ast("(1 + 2)"), "(group (+ 1 2))");
    }
}
