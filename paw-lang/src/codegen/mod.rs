#![allow(dead_code)]
pub(crate) mod emitter;
pub(crate) mod statements;

use crate::lexer::{LiteralValue, expr::Expr, Token, TokenType};
use crate::codegen::{emitter::Emitter, statements::{Statement, VarKind}};

use std::collections::HashMap;
use std::io;

pub struct Codegen<'e> {
    emitter: Emitter<'e>,
    operators: HashMap<TokenType, &'static str>,
}

impl<'e> Codegen<'e> {
    pub fn new(emitter: Emitter<'e>) -> Self {
        let operators = HashMap::from([
            (TokenType::BangEqual, "~="),
            (TokenType::Bang, "not"),
            (TokenType::And, "and"),
            (TokenType::Or, "or"),
        ]);
        Self { emitter, operators }
    }

    pub fn finish(self) -> io::Result<()> {
        self.emitter.finish()
    }

    pub fn emit_program(&mut self, stmts: &[Statement]) {
        for stmt in stmts {
            self.emit_stmt(stmt);
        }
    }

    fn emit_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr) => {
                self.emitter.emit_indent();
                self.emit_expr(expr);
                self.emitter.newline();
            }

            Statement::Block(stmts) => {
                for s in stmts {
                    self.emit_stmt(s);
                }
            }

            Statement::Variable { name, init, kind } => {
                self.emitter.emit_indent();
                self.emitter.write(match kind {
                    VarKind::Let => "local ",
                    VarKind::Const => "const ",
                });
                self.emitter.write(&name.lexeme);

                if let Some(expr) = init {
                    self.emitter.write_spaced("=");
                    self.emit_expr(expr);
                }
                self.emitter.newline();
            }

            Statement::If { predicate, then, else_block } => {
                self.emitter.emit_indent();
                self.emitter.write("if ");
                self.emit_expr(predicate);

                self.emitter.write(" then");
                self.emitter.newline();

                self.emitter.indent();
                self.emit_stmt(then);
                self.emitter.dedent();

                if let Some(else_stmt) = else_block {
                    self.emitter.writeln("else");
                    self.emitter.indent();
                    self.emit_stmt(else_stmt);
                    self.emitter.dedent();
                }
                self.emitter.writeln("end");
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(v) => self.emit_literal(v),
            Expr::Binary { left, operator, right } => {
                self.emit_expr(left);
                self.emitter.write_spaced(self.op(operator));
                self.emit_expr(right);
            }

            Expr::Unary { operator, right } => {
                let op = self.op(operator);
                self.emitter.write(op);
                
                if op.ends_with(char::is_alphabetic) {
                    self.emitter.write(" ");
                }
                self.emit_expr(right);
            }

            Expr::Grouping(inner) => {
                self.emitter.write("(");
                self.emit_expr(inner);
                self.emitter.write(")");
            }

            Expr::Variable(name) => {
                self.emitter.write(&name.lexeme);
            }

            Expr::Assign { name, value } => {
                self.emitter.write(&name.lexeme);
                self.emitter.write_spaced("=");
                self.emit_expr(value);
            }
        }
    }

    fn emit_literal(&mut self, v: &LiteralValue) {
        match v {
            LiteralValue::NumberValue(n) => self.emitter.write(&n.to_string()),
            LiteralValue::StringValue(s) => self.emitter.write(&format!("\"{}\"", s)),
            LiteralValue::DecoratorValue(_) => todo!(),
            LiteralValue::True => self.emitter.write("true"),
            LiteralValue::False => self.emitter.write("false"),
            LiteralValue::Nil => self.emitter.write("nil"),
        }
    }

    fn op<'a>(&self, tok: &'a Token) -> &'a str {
        self.operators
            .get(&tok.token_type)
            .copied()
            .unwrap_or(tok.lexeme.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::scanner::Scanner;
    use crate::lexer::parser::Parser;

    fn compile(src: &str) -> String {
        let tokens = Scanner::new(src).scan_tokens().expect("scan failed");
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        
        let mut buf = Vec::new();
        let emitter = Emitter::new(&mut buf);
        let mut cg = Codegen::new(emitter);

        cg.emit_program(&stmts);
        cg.finish().expect("io error");
        String::from_utf8(buf).expect("invalid utf-8")
    }

    #[test]
    fn literal_num() {
        assert_eq!(compile("42"), "42\n");
    }

    #[test]
    fn literal_string() {
        assert_eq!(compile("\"hello\""), "\"hello\"\n");
    }

    #[test]
    fn binary_add() {
        assert_eq!(compile("1 + 2"), "1 + 2\n");
    }

    #[test]
    fn binary_neq() {
        assert_eq!(compile("1 != 2"), "1 ~= 2\n");
    }

    #[test]
    fn unary_not() {
        assert_eq!(compile("!true"), "not true\n");
    }

    #[test]
    fn unary_neg() {
        assert_eq!(compile("-5"), "-5\n");
    }

    #[test]
    fn grouping() {
        assert_eq!(compile("(1 + 2)"), "(1 + 2)\n");
    }

    #[test]
    fn nested_binary() {
        assert_eq!(compile("1 + 2 * 3"), "1 + 2 * 3\n");
    }

    #[test]
    fn let_decl() {
        assert_eq!(compile("let x = 5"), "local x = 5\n");
    }

    #[test]
    fn let_no_init_fallback() {
        assert_eq!(compile("let x"), "local x\n");
    }

    #[test]
    fn const_decl() {
        assert_eq!(compile("const x = 5"), "const x = 5\n");
    }

    #[test]
    fn variable_ref() {
        assert_eq!(compile("x"), "x\n");
    }

    #[test]
    fn assign_literal() {
        assert_eq!(compile("x = 5"), "x = 5\n");
    }

    #[test]
    fn assign_expr() {
        assert_eq!(compile("x = 1 + 2"), "x = 1 + 2\n");
    }

    #[test]
    fn let_then_assign() {
        assert_eq!(
            compile("let x = 1\nx = 2"),
            "local x = 1\nx = 2\n"
        );
    }

    #[test]
    fn logical_and_emits_word() {
        assert_eq!(compile("true && false"), "true and false\n");
    }

    #[test]
    fn logical_or_emits_word() {
        assert_eq!(compile("true || false"), "true or false\n");
    }

    #[test]
    fn logical_mixed() {
        assert_eq!(
            compile("true || false && true"),
            "true or false and true\n"
        );
    }

    #[test]
    fn if_no_else() {
        assert_eq!(
            compile("if true { 1 }"),
            "if true then\n\t1\nend\n" // it's impossible, but whatever, I'll fix it.
        );
    }

    #[test]
    fn if_with_else() {
        assert_eq!(
            compile("if true { 1 } else { 2 }"),
            "if true then\n\t1\nelse\n\t2\nend\n" // same thing here as the previous one
        );
    }

    #[test]
    fn if_block_multiple_stmts() {
        assert_eq!(
            compile("if true { let x = 1\nx = 2 }"),
            "if true then\n\tlocal x = 1\n\tx = 2\nend\n"
        );
    }
}
