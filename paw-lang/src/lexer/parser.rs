#![allow(dead_code)]
use crate::codegen::statements::Statement;
use super::{
	TokenType, TokenType::*, 
	Token, LiteralValue,
	expr::{Expr, Expr::*}
};

type ParseResult<T> = Result<T, String>;

pub struct Parser {
	tokens: Vec<Token>,
	current: usize,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self { tokens, current: 0 }
	}

	pub fn parse(&mut self) -> ParseResult<Vec<Statement>> {
		let mut statements = vec![];
		let mut errors = vec![];

		while !self.is_at_end() {
			let statment = self.statement();

			match statment {
				Ok(s) => statements.push(s),
				Err(msg) => errors.push(msg),
			}
		}

		if errors.is_empty() {
			Ok(statements)
		} else {
			Err(errors.join("\n"))
		}
	}

	fn statement(&mut self) -> ParseResult<Statement> {
		if self.match_token(Print) {
			return self.print_statement()
		}
		self.expression_statement()
	}

	fn print_statement(&mut self) -> ParseResult<Statement> {
		let expr = self.expression()?;
		Ok(Statement::Print(expr))
	}

	fn expression_statement(&mut self) -> ParseResult<Statement> {
		let expr = self.expression()?;
		Ok(Statement::Expression(expr))
	}

	fn expression(&mut self) -> ParseResult<Expr> {
		self.equality()
	}

	fn equality(&mut self) -> ParseResult<Expr> {
		let mut expr = self.comparison()?;

		while self.match_tokens(&[BangEqual, EqualEqual]) {
			let operator = self.previous();
			let rhs = self.comparison()?;

			expr = Binary {
				left: Box::from(expr),
				operator,
				right: Box::from(rhs),
			}
		}

		Ok(expr)
	}

	fn comparison(&mut self) -> ParseResult<Expr> {
		let mut expr = self.term()?;

		while self.match_tokens(&[
			Greater, GreaterEqual,
			Less, LessEqual
		]) {
			let operator = self.previous();
			let rhs = self.term()?;

			expr = Binary {
				left: Box::from(expr),
				operator,
				right: Box::from(rhs),
			}
		}

		Ok(expr)
	}

	fn term(&mut self) -> ParseResult<Expr> {
		let mut expr = self.factor()?;

		while self.match_tokens(&[Minus, Plus]) {
			let operator = self.previous();
			let rhs = self.factor()?;

			expr = Binary {
				left: Box::from(expr),
				operator,
				right: Box::from(rhs),
			}
		}

		Ok(expr)
	}

	fn factor(&mut self) -> ParseResult<Expr> {
		let mut expr = self.unary()?;

		while self.match_tokens(&[Slash, Star]) {
			let operator = self.previous();
			let rhs = self.unary()?;

			expr = Binary {
				left: Box::from(expr),
				operator,
				right: Box::from(rhs),
			}
		}

		Ok(expr)
	}

	fn unary(&mut self) -> ParseResult<Expr> {
		if self.match_tokens(&[Bang, Minus]) {
			let operator = self.previous();
			let rhs = self.unary()?;

			Ok(Unary {
				operator,
				right: Box::from(rhs),
			})
		} else {
			self.primary()
		}
	}

	fn primary(&mut self) -> ParseResult<Expr> {
		let token = self.peek();

		let result;
		match token.token_type {
			LeftParen => {
				self.advance();

				let expr = self.expression()?;
				self.consume(RightParen, "expected ')'")?;
				result = Grouping(Box::from(expr));
			}
			Str | Number | True | False | Nil => {
				self.advance();
				result = Literal(LiteralValue::from_token(token))
			}
			_ => return Err("expected expression".to_string()),
		}

		Ok(result)
	}

	fn consume(&mut self, token_type: TokenType, msg: &str) -> ParseResult<Token> {
		if self.peek().token_type == token_type {
			Ok(self.advance())
		} else {
			Err(msg.to_string())
		}
	}

	fn match_token(&mut self, token_type: TokenType) -> bool {
		if self.is_at_end() {
			false
		} else {
			if self.peek().token_type == token_type {
				self.advance();
				true
			} else {
				false
			}
		}
	}

	fn match_tokens(&mut self, types: &[TokenType]) -> bool {
		for token_type in types {
			if self.match_token(*token_type) {
				return true;
			}
		}
		false
	}

	fn advance(&mut self) -> Token {
		if !self.is_at_end() {
			self.current += 1;
		}
		self.previous()
	}

	fn peek(&self) -> Token {
		self.tokens[self.current].clone()
	}

	fn previous(&self) -> Token {
		self.tokens[self.current - 1].clone()
	}

	fn is_at_end(&self) -> bool {
		self.peek().token_type == Eof
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		lexer::scanner::Scanner,
		lexer::parser::{Parser, ParseResult},
		codegen::statements::Statement
	};

	fn parse_str(src: &str) -> ParseResult<Vec<Statement>> {
		let tokens = Scanner::new(src).scan_tokens().expect("scan failed");
		let result = Parser::new(tokens).parse();
		// println!("{:?}", result);
		result
	}

	#[test]
	fn test_arithmetic() {
		assert!(parse_str("1 + 2 * 3").is_ok());
	}

	#[test]
	fn number_literal_ok() {
		assert!(parse_str("42").is_ok());
	}

	#[test]
	fn grouped_expr_ok() {
		assert!(parse_str("(1 + 2)").is_ok());
	}

	#[test]
	fn unary_ok() {
		assert!(parse_str("-5").is_ok());
		assert!(parse_str("!true").is_ok());
	}

	#[test]
	fn comparison_ok() {
		assert!(parse_str("3 > 2").is_ok());
		assert!(parse_str("1 <= 1").is_ok());
	}

	#[test]
	fn print_ok() {
		assert!(parse_str("print \"Hello\"").is_ok());
		assert!(parse_str("print true").is_ok());
		assert!(parse_str("print 1 + 2 + 3").is_ok());
	}
	
	#[test]
	#[should_panic]
	fn unclosed_paren_panics() {
		parse_str("(1 + 2").unwrap();
	}

	#[test]
	#[should_panic]
	fn incomplete_binary_panics() {
		parse_str("1 +").unwrap();
	}
}
