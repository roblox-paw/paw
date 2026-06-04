#![allow(dead_code)]
use crate::codegen::statements::{Statement, VarKind};
use crate::core::{CompileErrors, ParseError};

use super::{
	TokenType, TokenType::*,
	Token, LiteralValue,
	expr::{Expr, Expr::*, TableField, TableKey}
};

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
	tokens: Vec<Token>,
	current: usize,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self { tokens, current: 0 }
	}

	pub fn parse(&mut self) -> Result<Vec<Statement>, CompileErrors<ParseError>> {
		let mut statements = vec![];
		let mut errors: Vec<ParseError> = vec![];

		while !self.is_at_end() {
			let statement = self.declaration();

			match statement {
				Ok(s) => statements.push(s),
				Err(e) => {
					errors.push(e);
					self.synchronize();
				}
			}
		}

		if errors.is_empty() {
			Ok(statements)
		} else {
			Err(CompileErrors::new(errors))
		}
	}

	pub fn expression(&mut self) -> ParseResult<Expr> {
		self.assignment()
	}

	fn declaration(&mut self) -> ParseResult<Statement> {
		if self.match_token(Let) {
			self.var_declaration(VarKind::Let)
		} 
		else if self.match_token(Const) {
			self.var_declaration(VarKind::Const)
		} 
		else {
			self.statement()
		}
	}

	fn var_declaration(&mut self, kind: VarKind) -> ParseResult<Statement> {
		let name = self.consume(Identifier)?;

		// todo DO NOT FORGET!!!!!
		// when type annotations land, require init OR type annotation for `let`
		let init = if self.match_token(Equal) {
			Some(self.expression()?)
		} 
		else if kind == VarKind::Const {
			return Err(ParseError::ConstRequiresInitializer {
				span: self.previous().span(),
			})
		} 
		else {
			None
		};

		Ok(Statement::Variable { name, init, kind })
	}

	fn statement(&mut self) -> ParseResult<Statement> {
		if self.match_token(If) {
			self.if_statement()
		}
		else if self.match_token(While) {
			self.while_statement()
		}
		else if self.match_token(Loop) {
			self.loop_statement()
		}
		else if self.match_token(Return) {
			self.return_statement()
		}
		else {
			self.expression_statement()
		}
	}

	fn block(&mut self) -> ParseResult<Statement> {
		self.consume_with(LeftBrace, "add an opening '{' to start the block")?;
		let mut statements = vec![];

		while !self.is_at_end() && self.peek().token_type != RightBrace {
			statements.push(self.declaration()?);
		}

		self.consume_with(RightBrace, "add a closing '}' to end the block")?;
		Ok(Statement::Block(statements))
	}
	
	fn if_statement(&mut self) -> ParseResult<Statement> {
		if self.peek().token_type == LeftBrace || self.is_at_end() {
			return Err(ParseError::IfMissingCondition {
				span: self.peek().span()
			});
		}

		let predicate = self.expression()?;
		let then = Box::from(self.block()?);

		let else_block = if self.match_token(Else) {
			let stmt = if self.match_token(If) {
				self.if_statement()?
			}
			else {
				self.block()?
			};
			Some(Box::from(stmt))
		}
		else {
			None
		};

		Ok(Statement::If { predicate, then, else_block })
	}

	fn while_statement(&mut self) -> ParseResult<Statement> {
		if self.peek().token_type == LeftBrace || self.is_at_end() {
			return Err(ParseError::WhileMissingCondition {
				span: self.peek().span()
			});
		}

		let condition = self.expression()?;
		let body = Box::from(self.block()?);
		Ok(Statement::While { condition, body })
	}

	// basically translates loop { ... } -> 'while true do ... end' that is wrapped in task.spawn()
	fn loop_statement(&mut self) -> ParseResult<Statement> {
		let body = Box::from(self.block()?);
		Ok(Statement::Loop(body))
	}

	fn return_statement(&mut self) -> ParseResult<Statement> {
		let value = if self.is_at_end() || self.peek().token_type == RightBrace {
			None
		} else {
			Some(self.expression()?)
		};
		Ok(Statement::Return(value))
	}
	
	fn expression_statement(&mut self) -> ParseResult<Statement> {
		let expr = self.expression()?;
		match &expr {
			Assign { .. } => Ok(Statement::Expression(expr)),
			_ => Err(ParseError::ExpressionHasNoEffect {
				span: self.previous().span()
			}),
		}
	}

	fn assignment(&mut self) -> ParseResult<Expr> {
		let expr = self.or()?;

		if self.peek().token_type == Equal {
			let eq_tok = self.advance();
			let value = self.or()?;

			if self.peek().token_type == Equal {
				return Err(ParseError::ChainedAssignment {
					span: self.peek().span()
				});
			}

			match expr {
				Variable(name) => Ok(Assign {name, value: Box::from(value) }),
				_ => Err(ParseError::InvalidAssignTarget { span: eq_tok.span() }),
			}
		} else {
			Ok(expr)
		}
	}

	fn or(&mut self) -> ParseResult<Expr> {
		let mut expr = self.and()?;

		while self.match_token(Or) {
			let operator = self.previous();
			let right = self.and()?;

			expr = Binary {
				left: Box::from(expr),
				operator,
				right: Box::from(right),
			}
		}

		Ok(expr)
	}

	fn and(&mut self) -> ParseResult<Expr> {
		let mut expr = self.equality()?;

		while self.match_token(And) {
			let operator = self.previous();
			let right = self.equality()?;

			expr = Binary {
				left: Box::from(expr),
				operator,
				right: Box::from(right),
			}
		}

		Ok(expr)
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
			If => {
				self.advance();
				let predicate = Box::from(self.expression()?);

				self.consume_with(LeftBrace, "add '{' after the condition")?;
				let then_expr = Box::from(self.expression()?);
				self.consume(RightBrace)?;

				if !self.match_token(Else) {
					return Err(ParseError::IfExprRequiresElse {
						span: self.previous().span()
					});
				}

				self.consume_with(LeftBrace, "add '{' after the else keyword")?;
				let else_expr = Box::from(self.expression()?);
				self.consume(RightBrace)?;

				result = IfExpr { predicate, then_expr, else_expr }
			}

			LeftBrace => {
				self.advance();
				let mut fields = vec![];

				while !self.is_at_end() && self.peek().token_type != RightBrace {
					fields.push(self.table_field()?);

					if self.match_token(Comma) {
						continue
					}

					if self.peek().token_type != RightBrace {
						return Err(ParseError::TableExpectedComma {
							span: self.previous().span()
						});
					}
				}

				self.consume_with(RightBrace, "add a closing '}' to end the table")?;
				result = Table(fields)
			}

			LeftParen => {
				self.advance();

				let expr = self.expression()?;
				self.consume(RightParen)?;
				result = Grouping(Box::from(expr))
			}

			Str | Number | True | False | Nil => {
				self.advance();
				result = Literal(LiteralValue::from_token(token))
			}

			Identifier => {
				self.advance();
				result = Variable(self.previous())
			}

			_ => return Err(ParseError::ExpectedExpression {
				span: self.peek().span()
			}),
		}

		Ok(result)
	}

	fn table_field(&mut self) -> ParseResult<TableField> {
		// !!! todo: table must have a great typechecker
		// to verify that all keys share the same type.
		// wrote this for myself, so I won't forget

		// name = value
		if self.peek().token_type == Identifier 
			&& self.peek_next().token_type == Equal
		{
			let name = self.advance();
			self.advance(); // consume '='

			return Ok(TableField {
				key: TableKey::Ident(name),
				value: self.expression()?,
			});
		}

		// [expr] = value
		if self.peek().token_type == LeftBracket {
			self.advance();
			let key_expr = self.expression()?;

			self.consume_with(RightBracket, "close the computed key with ']'")?;
			self.consume_with(Equal, "a computed key needs '= value'")?;

			return Ok(TableField {
				key: TableKey::Computed(key_expr),
				value: self.expression()?,
			});
		}

		// value or [index] = value
		let value = self.expression()?;
		Ok(TableField { key: TableKey::None, value })
	}

	fn synchronize(&mut self) {
		while !self.is_at_end() {
			self.advance();
			
			match self.peek().token_type {
				Let | Const | If | While | Loop | Return => return,
				_ => ()
			}
		}
	}

	fn consume(&mut self, token_type: TokenType) -> ParseResult<Token> {
		self.consume_with(token_type, "")
	}

	fn consume_with(&mut self, token_type: TokenType, advice: impl Into<String>) -> ParseResult<Token> {
		if self.peek().token_type == token_type {
			Ok(self.advance())
		} 
		else {
			Err(ParseError::ExpectedToken {
				expected: token_type,
				found: self.peek().token_type,
				span: self.peek().span(),
				
				advice: {
					let s = advice.into();
					if s.is_empty() { None } else { Some(s) }
				},
			})
		}
	}

	fn match_token(&mut self, token_type: TokenType) -> bool {
		if self.is_at_end() {
			false
		} 
		else {
			if self.peek().token_type == token_type {
				self.advance();
				true
			} 
			else {
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

	fn peek_next(&self) -> Token {
		self.tokens[(self.current + 1).min(self.tokens.len() - 1)].clone()
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
		lexer::parser::Parser,
		core::{CompileErrors, ParseError},
		codegen::statements::Statement
	};

	fn parse_str(src: &str) -> Result<Vec<Statement>, CompileErrors<ParseError>> {
		let tokens = Scanner::new(src).scan_tokens().expect("scan failed");
		let result = Parser::new(tokens).parse();
		// println!("{:?}", result);
		result
	}

	#[test]
	fn test_arithmetic() {
		assert!(parse_str("let x = 1 + 2 * 3").is_ok());
	}

	#[test]
	fn number_literal_ok() {
		assert!(parse_str("let x = 42").is_ok());
	}

	#[test]
	fn grouped_expr_ok() {
		assert!(parse_str("let x = (1 + 2)").is_ok());
	}

	#[test]
	fn unary_ok() {
		assert!(parse_str("let x = -5").is_ok());
		assert!(parse_str("let x = !true").is_ok());
	}

	#[test]
	fn comparison_ok() {
		assert!(parse_str("let x = 3 > 2").is_ok());
		assert!(parse_str("let x = 1 <= 1").is_ok());
	}

	#[test]
	#[should_panic]
	fn bare_expr_panics() {
		parse_str("1 + 2").unwrap();
	}

	#[test]
	fn declare_var_ok() {
		assert!(parse_str("let x = 1").is_ok());
		assert!(parse_str("const y = 2").is_ok());
	}

	#[test]
	fn declare_var_fallback_ok() {
		assert!(parse_str("let x").is_ok());
	}

	#[test]
	#[should_panic]
	fn declare_var_fallback_const_panics() {
		assert!(parse_str("const y").is_ok());
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
	
	#[test]
	fn and_ok() {
		assert!(parse_str("let x = true && false").is_ok());
	}

	#[test]
	fn or_ok() {
		assert!(parse_str("let x = true || false").is_ok());
	}

	#[test]
	fn and_chained_ok() {
		assert!(parse_str("let x = true && false && true").is_ok());
	}

	#[test]
	fn or_chained_ok() {
		assert!(parse_str("let x = true || false || true").is_ok());
	}

	#[test]
	fn or_and_precedence_ok() {
		assert!(parse_str("let x = true || false && true").is_ok());
	}

	#[test]
	fn logical_with_comparison_ok() {
		assert!(parse_str("let x = 1 < 2 && 3 > 2").is_ok());
	}

	#[test]
	#[should_panic]
	fn incomplete_and_panics() {
		parse_str("true &&").unwrap();
	}

	#[test]
	#[should_panic]
	fn incomplete_or_panics() {
		parse_str("true ||").unwrap();
	}

	#[test]
	fn if_ok() {
		assert!(parse_str("if true { x = 1 }").is_ok());
	}

	#[test]
	fn if_else_ok() {
		assert!(parse_str("if true { x = 1 } else { x = 2 }").is_ok());
	}

	#[test]
	fn if_nested_ok() {
		assert!(parse_str("if true { if false { x = 1 } }").is_ok());
	}

	#[test]
	#[should_panic]
	fn bare_block_panics() {
		parse_str("{ 1 }").unwrap();
	}

	#[test]
	#[should_panic]
	fn if_missing_brace_panics() {
		parse_str("if true 1").unwrap();
	}
}
