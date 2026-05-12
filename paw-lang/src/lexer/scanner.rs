use super::{LiteralValue::*, TokenType::*, *};
use std::str::FromStr;

macro_rules! scan_err {
    ($line:expr, $fmt:literal $(, $args:expr)*) => {
        format!(concat!("Line {}: ", $fmt), $line $(, $args)*)
    };
}

pub struct Scanner {
	source: Vec<char>,
	tokens: Vec<Token>,
	start: usize,
	current: usize,
	line: usize,
}

impl Scanner {
	pub fn new(source: &str) -> Self {
		Self {
			source: source.chars().collect(),
			tokens: vec![],
			start: 0,
			current: 0,
			line: 1,
		}
	}

	pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
		let mut errors = vec![];

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
			line_number: self.line,
		});

		if !errors.is_empty() {
			let mut joined = String::new();

			for error in errors {
				joined.push_str(&error);
				// joined.push('\n');
			}

			return Err(joined);
		}

		Ok(self.tokens)
	}

	fn scan_token(&mut self) -> Result<(), String> {
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
			'@' => self.add_token(At),
            
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
            
			'/' => {
				if self.char_match('/') {
					while self.peek() != '\n' && !self.is_at_end() {
						self.advance();
					}
				} else if self.char_match('=') {
					self.add_token(SlashEqual)
				} else {
					self.add_token(Slash)
				}
			}

			' ' | '\r' | '\t' => {}
			'\n' => self.line += 1,
			'"' => self.string()?,

			c if c.is_ascii_digit() => self.number()?,
			c if c.is_alphabetic() || c == '_' => self.identifier(),
			_ => {
				return Err(scan_err!(self.line, "unexpected character '{}'", c))
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
			line_number: self.line,
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

	fn number(&mut self) -> Result<(), String> {
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
			.map_err(|_| scan_err!(self.line, "could not parse number - {}", s))?;

		self.add_token_lit(Number, Some(NumberValue(value)));
		Ok(())
	}

	fn string(&mut self) -> Result<(), String> {
		let start_line = self.line;

		while self.peek() != '"' && !self.is_at_end() {
			if self.peek() == '\n' {
				self.line += 1;
			}
			self.advance();
		}

		if self.is_at_end() {
			return Err(scan_err!(start_line, "unterminated string"));
		}
		self.advance();

		let value: String = self.source[self.start + 1..self.current - 1]
			.iter()
			.collect();

		self.add_token_lit(Str, Some(StringValue(value)));
		Ok(())
	}

	fn is_at_end(&self) -> bool {
		self.current >= self.source.len()
	}

	fn is_digit(&self) -> bool {
		self.peek().is_ascii_digit()
	}

	fn current_lexeme(&self) -> String {
		self.source[self.start..self.current].iter().collect()
	}

	fn peek(&self) -> char {
		if self.is_at_end() {
			'\0'
		} else {
			self.source[self.current]
		}
	}

	fn peek_next(&self) -> char {
		if self.current + 1 >= self.source.len() {
			'\0'
		} else {
			self.source[self.current + 1]
		}
	}

	fn advance(&mut self) -> char {
		let c = self.source[self.current];
		self.current += 1;
		c
	}

	fn char_match(&mut self, expected: char) -> bool {
		if self.is_at_end() || self.source[self.current] != expected {
			return false;
		}
		self.current += 1;
		true
	}
}
