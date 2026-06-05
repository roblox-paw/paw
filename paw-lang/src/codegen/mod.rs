#![allow(dead_code)]
pub(crate) mod emitter;
pub(crate) mod statements;

use crate::lexer::{LiteralValue, expr::{Expr, TableField, TableKey}, Token, TokenType};
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

                self.emit_else_chain(else_block);
                self.emitter.writeln("end");
            }

            Statement::Return(value) => {
                self.emitter.emit_indent();
                self.emitter.write("return");
                if let Some(expr) = value {
                    self.emitter.write(" ");
                    self.emit_expr(expr);
                }
                self.emitter.newline();
            }

            Statement::Loop(body) => {
                self.emitter.emit_indent();
                self.emitter.writeln("task.spawn(function()");
                self.emitter.indent();

                self.emitter.writeln("while true do");
                self.emitter.indent();
                self.emit_stmt(body);
                self.emitter.dedent();

                self.emitter.writeln("end");
                self.emitter.dedent();
                self.emitter.writeln("end)");
            }

            Statement::While { condition, body } => {
                self.emitter.emit_indent();
                self.emitter.write("while ");
                self.emit_expr(condition);

                self.emitter.write(" do");
                self.emitter.newline();

                self.emitter.indent();
                self.emit_stmt(body);
                self.emitter.dedent();
                
                self.emitter.writeln("end");
            }

            Statement::For { ident, iter, step, body } => {
                self.emitter.emit_indent();

                if let Expr::Binary { operator, left, right } = iter {
                    let is_range = 
                        operator.token_type == TokenType::DotDot || 
                        operator.token_type == TokenType::DotDotEqual;

                    if is_range {
                        self.emitter.write("for ");
                        self.emitter.write(&ident[0].lexeme);
                        self.emitter.write_spaced("=");

                        self.emit_expr(left);
                        self.emitter.write(", ");
                        self.emit_expr(right);

                        // '..=' inclusive, so no adjustment for it, only for '..'
                        if operator.token_type == TokenType::DotDot {
                            self.emitter.write(" - 1");
                        }

                        if let Some(s) = step {
                            self.emitter.write(", ");
                            self.emit_expr(s);
                        }

                        self.emitter.write(" do");
                        self.emitter.newline();

                        self.emitter.indent();
                        self.emit_stmt(body);
                        self.emitter.dedent();

                        self.emitter.writeln("end");
                        return
                    }
                }

                self.emitter.write("for ");
                for (i, var) in ident.iter().enumerate() {
                    if i > 0 {
                        self.emitter.write(", ");
                    }
                    self.emitter.write(&var.lexeme);
                }
                self.emitter.write_spaced("in");
                self.emit_expr(iter);

                self.emitter.write(" do");
                self.emitter.newline();

                self.emitter.indent();
                self.emit_stmt(body);
                self.emitter.dedent();
                self.emitter.writeln("end");
            }

            Statement::Continue => self.emitter.writeln("continue"),
            Statement::Break => self.emitter.writeln("break"),
        }
    }

    fn emit_else_chain(&mut self, else_block: &Option<Box<Statement>>) {
        let Some(stmt) = else_block else { return };
        match stmt.as_ref() {
            Statement::If { predicate, then, else_block } => {
                self.emitter.emit_indent();
                self.emitter.write("elseif ");
                self.emit_expr(predicate);

                self.emitter.write(" then");
                self.emitter.newline();

                self.emitter.indent();
                self.emit_stmt(then);
                self.emitter.dedent();

                self.emit_else_chain(else_block);
            }
            
            _ => {
                self.emitter.writeln("else");
                self.emitter.indent();
                self.emit_stmt(stmt);
                self.emitter.dedent();
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(v) => self.emit_literal(v),
            Expr::Grouping(inner) => {
                self.emitter.write("(");
                self.emit_expr(inner);
                self.emitter.write(")");
            }

            Expr::Table { fields, multiline } => {
                if fields.is_empty() {
                    self.emitter.write("{}");
                    return;
                }

                if *multiline {
                    self.emitter.write("{");
                    self.emitter.newline();
                    self.emitter.indent();

                    for field in fields {
                        self.emitter.emit_indent();
                        self.emit_table_key(field);
                        self.emit_expr(&field.value);

                        self.emitter.write(",");
                        self.emitter.newline();
                    }

                    self.emitter.dedent();
                    self.emitter.emit_indent();
                    self.emitter.write("}");
                }
                else {
                    self.emitter.write("{");
                    for (i, field) in fields.iter().enumerate() {
                        if i > 0 {
                            self.emitter.write(", ");
                        }
                        self.emit_table_key(field);
                        self.emit_expr(&field.value);
                    }
                    self.emitter.write("}");
                }
            }

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

            Expr::Variable(name) => {
                self.emitter.write(&name.lexeme);
            }

            Expr::Assign { name, value } => {
                self.emitter.write(&name.lexeme);
                self.emitter.write_spaced("=");
                self.emit_expr(value);
            }

            Expr::IfExpr { predicate, then_expr, else_expr } => {
                self.emitter.write("if ");
                self.emit_expr(predicate);

                self.emitter.write_spaced("then");
                self.emit_expr(then_expr);

                self.emitter.write_spaced("else");
                self.emit_expr(else_expr);
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

    fn emit_table_key(&mut self, field: &TableField) {
        match &field.key {
            TableKey::Ident(k) => {
                self.emitter.write(&k.lexeme);
                self.emitter.write_spaced("=");
            }
            TableKey::Computed(key_expr) => {
                self.emitter.write("[");
                self.emit_expr(key_expr);
                self.emitter.write("] = ");
            }
            TableKey::None => {}
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
        assert_eq!(compile("let x = 42"), "local x = 42\n");
    }

    #[test]
    fn literal_string() {
        assert_eq!(compile("let x = \"hello\""), "local x = \"hello\"\n");
    }

    #[test]
    fn binary_add() {
        assert_eq!(compile("let x = 1 + 2"), "local x = 1 + 2\n");
    }

    #[test]
    fn binary_neq() {
        assert_eq!(compile("let x = 1 != 2"), "local x = 1 ~= 2\n");
    }

    #[test]
    fn unary_not() {
        assert_eq!(compile("let x = !true"), "local x = not true\n");
    }

    #[test]
    fn unary_neg() {
        assert_eq!(compile("let x = -5"), "local x = -5\n");
    }

    #[test]
    fn grouping() {
        assert_eq!(compile("let x = (1 + 2)"), "local x = (1 + 2)\n");
    }

    #[test]
    fn nested_binary() {
        assert_eq!(compile("let x = 1 + 2 * 3"), "local x = 1 + 2 * 3\n");
    }

    #[test]
    #[should_panic]
    fn bare_expr_panics() {
        compile("42");
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
        assert_eq!(compile("let y = x"), "local y = x\n");
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
        assert_eq!(compile("let x = true && false"), "local x = true and false\n");
    }

    #[test]
    fn logical_or_emits_word() {
        assert_eq!(compile("let x = true || false"), "local x = true or false\n");
    }

    #[test]
    fn logical_mixed() {
        assert_eq!(
            compile("let x = true || false && true"),
            "local x = true or false and true\n"
        );
    }

    #[test]
    fn if_no_else() {
        assert_eq!(
            compile("if true { x = 1 }"),
            "if true then\n\tx = 1\nend\n"
        );
    }

    #[test]
    fn if_with_else() {
        assert_eq!(
            compile("if true { x = 1 } else { x = 2 }"),
            "if true then\n\tx = 1\nelse\n\tx = 2\nend\n"
        );
    }

    #[test]
    fn if_condition_expr() {
        assert_eq!(
            compile("if x > 0 { y = 1 }"),
            "if x > 0 then\n\ty = 1\nend\n"
        );
    }

    #[test]
    fn if_nested() {
        assert_eq!(
            compile("if true { if false { x = 1 } }"),
            "if true then\n\tif false then\n\t\tx = 1\n\tend\nend\n"
        );
    }

    #[test]
    fn if_block_multiple_stmts() {
        assert_eq!(
            compile("if true { let x = 1\nx = 2 }"),
            "if true then\n\tlocal x = 1\n\tx = 2\nend\n"
        );
    }

    #[test]
    fn else_if_emits_elseif() {
        assert_eq!(
            compile("if x > 5 { y = 1 } else if x > 0 { y = 2 }"),
            "if x > 5 then\n\ty = 1\nelseif x > 0 then\n\ty = 2\nend\n"
        );
    }

    #[test]
    fn if_expr_in_let() {
        assert_eq!(
            compile("let label = if score > 50 { \"Win\" } else { \"Loss\" }"),
            "local label = if score > 50 then \"Win\" else \"Loss\"\n"
        );
    }

    #[test]
    fn if_expr_nested() {
        assert_eq!(
            compile("let x = if a { if b { 1 } else { 2 } } else { 3 }"),
            "local x = if a then if b then 1 else 2 else 3\n"
        );
    }

    #[test]
    #[should_panic]
    fn if_expr_no_else_panics() {
        compile("let x = if true { 1 }");
    }

    #[test]
    fn else_if_chain_emits_elseif() {
        assert_eq!(
            compile("if x > 5 { y = 1 } else if x > 0 { y = 2 } else { y = 3 }"),
            "if x > 5 then\n\ty = 1\nelseif x > 0 then\n\ty = 2\nelse\n\ty = 3\nend\n"
        );
    }

    #[test]
    fn while_basic() {
        assert_eq!(
            compile("while x < 10 { x = x + 1 }"),
            "while x < 10 do\n\tx = x + 1\nend\n"
        );
    }

    #[test]
    #[should_panic]
    fn while_missing_condition_panics() {
        compile("while { x = 1 }");
    }

    #[test]
    #[should_panic]
    fn while_missing_body_panics() {
        compile("while x > 0");
    }

    #[test]
    fn loop_basic() {
        assert_eq!(
            compile("loop { x = x + 1 }"),
            "task.spawn(function()\n\twhile true do\n\t\tx = x + 1\n\tend\nend)\n"
        );
    }

    #[test]
    fn table_empty() {
        assert_eq!(compile("let t = {}"), "local t = {}\n");
    }

    #[test]
    fn table_keyed() {
        assert_eq!(
            compile("let t = { x = 1, y = 2 }"),
            "local t = {x = 1, y = 2}\n"
        );
    }

    #[test]
    fn table_positional() {
        assert_eq!(
            compile("let t = { 1, 2, 3 }"),
            "local t = {1, 2, 3}\n"
        );
    }

    #[test]
    fn table_nested() {
        assert_eq!(
            compile("let t = { pos = { x = 1, y = 2 } }"),
            "local t = {pos = {x = 1, y = 2}}\n"
        );
    }

    #[test]
    fn table_in_return() {
        assert_eq!(
            compile("return { ok = true }"),
            "return {ok = true}\n"
        );
    }

    #[test]
    fn table_computed_string_key() {
        assert_eq!(
            compile("let t = { [\"name\"] = 3 }"),
            "local t = {[\"name\"] = 3}\n"
        );
    }

    #[test]
    fn table_computed_expr_key() {
        assert_eq!(
            compile("let t = { [2 == 2] = 1 }"),
            "local t = {[2 == 2] = 1}\n"
        );
    }

    #[test]
    #[should_panic]
    fn table_computed_unclosed_bracket_panics() {
        compile("let t = { [\"x\" = 1 }");
    }

    #[test]
    fn table_trailing_comma() {
        assert_eq!(
            compile("let t = { x = 1, y = 2, }"),
            "local t = {x = 1, y = 2}\n"
        );
    }

    #[test]
    #[should_panic]
    fn table_missing_comma_panics() {
        compile("let t = { x = 1 y = 2 }");
    }

    #[test]
    #[should_panic]
    fn table_unclosed_panics() {
        compile("let t = { x = 1");
    }

    #[test]
    fn if_do_single_stmt() {
        assert_eq!(
            compile("if x > 0 do return x"),
            "if x > 0 then\n\treturn x\nend\n"
        );
    }

    #[test]
    fn continue_in_while() {
        assert_eq!(
            compile("while true { continue }"),
            "while true do\n\tcontinue\nend\n"
        );
    }

    #[test]
    fn for_range() {
        assert_eq!(
            compile("for i in 0..10 { x = i }"),
            "for i = 0, 10 - 1 do\n\tx = i\nend\n"
        );
    }

    #[test]
    fn for_collection_one_var() {
        assert_eq!(
            compile("for item in inventory { x = item }"),
            "for item in inventory do\n\tx = item\nend\n"
        );
    }

    #[test]
    fn for_collection_two_vars() {
        assert_eq!(
            compile("for i, item in inventory { x = item }"),
            "for i, item in inventory do\n\tx = item\nend\n"
        );
    }
}
