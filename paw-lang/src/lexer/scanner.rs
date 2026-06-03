use super::{LiteralValue::*, TokenType::*, *};
use crate::core::{CompileErrors, LexError};

use miette::SourceSpan;
use std::str::FromStr;

pub struct Scanner<'src> {
	src: &'src str,
	chars: Vec<char>,
	tokens: Vec<Token>,
	start: usize,
	current: usize,
	line: usize,
}

impl<'src> Scanner<'src> {
	pub fn new(src: &'src str) -> Self {
		Self {
			chars: src.chars().collect(),
			src,
			tokens: vec![],
			start: 0,
			current: 0,
			line: 1,
		}
	}

	pub fn scan_tokens(mut self) -> Result<Vec<Token>, CompileErrors<LexError>> {
		let mut errors: Vec<LexError> = vec![];

		while !self.is_at_end() {
			self.start = self.current;

			if let Err(e) = self.scan_token() {
				errors.push(e);
			}
		}

		self.tokens.push(Token {
			token_type: Eof,
			lexeme: String::new(),
			literal: None,
			offset: self.byte_offset(self.current) as u32,
		});

		if !errors.is_empty() {
			return Err(CompileErrors::new(errors));
		}

		Ok(self.tokens)
	}

	fn scan_token(&mut self) -> Result<(), LexError> {
		let c = self.advance();

		match c {
			'(' => self.add_token(LeftParen),
			')' => self.add_token(RightParen),
			'{' => self.add_token(LeftBrace),
			'}' => self.add_token(RightBrace),
			'[' => self.add_token(LeftBracket),
			']' => self.add_token(RightBracket),
			',' => self.add_token(Comma),
			';' => self.add_token(Semicolon),
            
			'+' => self.match_token('=', PlusEqual, Plus),
			'*' => self.match_token('=', StarEqual, Star),
			'%' => self.match_token('=', PercentEqual, Percent),
            
			'!' => self.match_token('=', BangEqual, Bang),
			'<' => self.match_token('=', LessEqual, Less),
			'>' => self.match_token('=', GreaterEqual, Greater),
            
			'.' => self.match_token('.', DotDot, Dot),
			':' => self.match_token(':', ColonColon, Colon),
			'|' => self.match_token('|', Or, Pipe),
            
			'-' => self.match_token_t('=', MinusEqual, '>', Arrow, Minus),
			'=' => self.match_token_t('=', EqualEqual, '>', FatArrow, Equal),
			'?' => self.match_token_t('?', QuestionQuestion, '.', QuestionDot, Question),

			'&' => {
				if self.char_match('&') {
					self.add_token(And);
				}
				else {
					return Err(LexError::SingleAmpersand { span: self.current_span() })
				}
			}
            
			'/' => {
				if self.char_match('/') {
					while self.peek() != '\n' && !self.is_at_end() {
						self.advance();
					}
				}
				else if self.char_match('=') {
					self.add_token(SlashEqual)
				}
				else {
					self.add_token(Slash)
				}
			}
			
			' ' | '\r' | '\t' => {}
			'\n' => self.line += 1,
			'"' => self.string()?,
			
			c if c.is_ascii_digit() => self.number()?,
			c if c.is_alphabetic() || c == '_' => self.identifier(),
			'@' => self.decorator()?,

			_ => {
				return Err(LexError::UnexpectedChar {
					ch: c, span: self.current_span()
				})
			}
		}

		Ok(())
	}

	fn add_token(&mut self, token_type: TokenType) {
		self.add_token_lit(token_type, None)
	}

	fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
		let lexeme = self.current_lexeme();
		self.tokens.push(Token {
			token_type,
			lexeme,
			literal,
			offset: self.byte_offset(self.start) as u32,
		})
	}

	fn match_token(&mut self, ch: char, yes: TokenType, no: TokenType) {
		let t = if self.char_match(ch) { yes } else { no };
		self.add_token(t)
	}

	fn match_token_t(&mut self, a: char, ya: TokenType, b: char, yb: TokenType, no: TokenType) {
		let t = if self.char_match(a) { ya }
		        else if self.char_match(b) { yb }
		        else { no };
				
		self.add_token(t)
	}

	fn identifier(&mut self) {
		while self.peek().is_alphanumeric() || self.peek() == '_' {
			self.advance();
		}

		let lexeme = self.current_lexeme();
		let token_type = TokenType::from_str(&lexeme).unwrap_or(Identifier);
		self.add_token(token_type)
	}

	fn number(&mut self) -> Result<(), LexError> {
		while self.is_digit() {
			self.advance();
		}

		if self.peek() == '.' && self.peek_next().is_ascii_digit() {
			self.advance();
			while self.is_digit() {
				self.advance();
			}
		}

		let s = self.current_lexeme();
		let value = s
			.parse::<f64>()
			.map_err(|_| LexError::InvalidNumber {
				text: s.clone(),
				span: self.current_span()
			})?;

		self.add_token_lit(Number, Some(NumberValue(value)));
		Ok(())
	}

	fn string(&mut self) -> Result<(), LexError> {
		while self.peek() != '"' && !self.is_at_end() {
			if self.peek() == '\n' {
				self.line += 1;
			}
			self.advance();
		}

		if self.is_at_end() {
			return Err(LexError::UnterminatedString {
				span: self.current_span()
			});
		}
		self.advance();

		let value: String = self.chars[self.start + 1..self.current - 1]
			.iter()
			.collect();

		self.add_token_lit(Str, Some(StringValue(value)));
		Ok(())
	}

	fn decorator(&mut self) -> Result<(), LexError> {
		if !self.peek().is_alphabetic() && self.peek() != '_' {
			return Err(LexError::ExpectedIdentAfterAt {
				span: self.current_span()
			});
		}
		
		while self.peek().is_alphanumeric() || self.peek() == '_' {
			self.advance();
		}
		let value: String = self.chars[self.start + 1..self.current]
			.iter()
			.collect();

		self.add_token_lit(Decorator, Some(DecoratorValue(value)));
		Ok(())
	}

	fn byte_offset(&self, char_idx: usize) -> usize {
		self.chars[..char_idx]
			.iter()
			.map(|c| c.len_utf8())
			.sum()
	}

	fn current_span(&self) -> SourceSpan {
		let start = self.byte_offset(self.start);
		let len = self.byte_offset(self.current) - start;
		(start, len).into()
	}

	fn is_at_end(&self) -> bool {
		self.current >= self.chars.len()
	}

	fn is_digit(&self) -> bool {
		self.peek().is_ascii_digit()
	}

	fn current_lexeme(&self) -> String {
		self.chars[self.start..self.current].iter().collect()
	}

	fn peek(&self) -> char {
		if self.is_at_end() {
			'\0'
		} else {
			self.chars[self.current]
		}
	}

	fn peek_next(&self) -> char {
		if self.current + 1 >= self.chars.len() {
			'\0'
		} else {
			self.chars[self.current + 1]
		}
	}

	fn advance(&mut self) -> char {
		let c = self.chars[self.current];
		self.current += 1;
		c
	}

	fn char_match(&mut self, expected: char) -> bool {
		if self.is_at_end() || self.chars[self.current] != expected {
			return false;
		}
		self.current += 1;
		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn scan(src: &str) -> Vec<Token> {
		Scanner::new(src).scan_tokens().expect("scan failed")
	}

	#[test]
	fn decorator_simple() {
		let tokens = scan("@override");
		assert_eq!(tokens[0].token_type, Decorator);
		assert_eq!(tokens[0].lexeme, "@override");
		assert!(matches!(
			&tokens[0].literal,
			Some(DecoratorValue(n)) if n == "override"
		));
	}

	#[test]
	fn decorator_underscore_prefix() {
		let tokens = scan("@_internal");
		assert_eq!(tokens[0].token_type, Decorator);
		assert!(matches!(
			&tokens[0].literal,
			Some(DecoratorValue(n)) if n == "_internal"
		));
	}

	#[test]
	fn decorator_with_digits() {
		let tokens = scan("@deprecated2");
		assert_eq!(tokens[0].token_type, Decorator);
		assert!(matches!(
			&tokens[0].literal,
			Some(DecoratorValue(n)) if n == "deprecated2"
		));
	}

	#[test]
	fn decorator_stops_at_non_ident() {
		let tokens = scan("@route ");
		assert_eq!(tokens[0].token_type, Decorator);
		assert_eq!(tokens[0].lexeme, "@route");
		assert_eq!(tokens[1].token_type, Eof);
	}

	#[test]
	#[should_panic]
	fn decorator_bare_at_panics() {
		scan("@");
	}

	#[test]
	#[should_panic]
	fn decorator_digit_after_at_panics() {
		scan("@1bad");
	}

	#[test]
	#[should_panic]
	fn decorator_space_after_at_panics() {
		scan("@ name");
	}
}