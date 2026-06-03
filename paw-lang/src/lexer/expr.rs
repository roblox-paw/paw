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
    Variable(Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    IfExpr {
        predicate: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
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
            Expr::Variable(n) => write!(f, "(var {})", n.lexeme),
            Expr::Assign { name, value } => {
                write!(f, "(= {} {value})", name.lexeme)
            }
            Expr::IfExpr { predicate, then_expr, else_expr } => {
                write!(f, "(if-expr {predicate} {then_expr} {else_expr})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::scanner::Scanner;
    use crate::lexer::parser::Parser;

    fn ast(src: &str) -> String {
        let tokens = Scanner::new(src).scan_tokens().expect("scan failed");
        Parser::new(tokens).expression().expect("parse failed").to_string()
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

    #[test]
    fn variable_ref() {
        assert_eq!(ast("x"), "(var x)");
    }

    #[test]
    fn variable_underscore() {
        assert_eq!(ast("_internal"), "(var _internal)");
    }

    #[test]
    fn assign_literal() {
        assert_eq!(ast("x = 5"), "(= x 5)");
    }

    #[test]
    fn assign_expr() {
        assert_eq!(ast("x = 1 + 2"), "(= x (+ 1 2))");
    }

    #[test]
    fn variable_in_binary() {
        assert_eq!(ast("x + 1"), "(+ (var x) 1)");
    }

    #[test]
    fn logical_and() {
        assert_eq!(ast("true && false"), "(&& true false)");
    }

    #[test]
    fn logical_or() {
        assert_eq!(ast("true || false"), "(|| true false)");
    }

    #[test]
    fn or_binds_lower_than_and() {
        assert_eq!(
            ast("true || false && true"),
            "(|| true (&& false true))"
        );
    }

    #[test]
    fn and_binds_lower_than_equality() {
        assert_eq!(
            ast("1 == 1 && 2 == 2"),
            "(&& (== 1 1) (== 2 2))"
        );
    }

    #[test]
    #[should_panic]
    fn assign_chained() {
        ast("x = y = 3 = 4");
    }
}
