use crate::lexer::expr::Expr;
// use crate::ir::compiler::Compiler;

#[derive(Debug)]
pub enum Statement {
	Expression(Expr),
	Print(Expr),
}

impl Statement {
	pub fn resolve(statements: &Vec<Statement>, compiler: &mut Compiler, is_repl: bool) {
		for stmt in statements {
			match stmt {
				Statement::Expression(expr) => {
					if is_repl {
						compiler.emit_print(expr)
					}
					else {
						compiler.emit_expression(expr)
					}
				},
				Statement::Print(expr) => compiler.emit_print(expr),
			}
		}
	}
}

impl std::fmt::Display for Statement {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Statement::Expression(e) => write!(f, "{e}"),
			Statement::Print(e) => write!(f, "{e}"),
		}
	}
}