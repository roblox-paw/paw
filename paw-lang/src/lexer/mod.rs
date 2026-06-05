#![allow(dead_code)]
pub(crate) mod parser;
pub(crate) mod scanner;
pub(crate) mod expr;

use miette::SourceSpan;
use strum::EnumString;

// #[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TokenType {
	// single character tokens
	LeftParen, RightParen,
	LeftBrace, RightBrace,
	LeftBracket, RightBracket,
	Comma, Semicolon, At,
    
	// one or two character tokens
	Equal, EqualEqual,
	Bang, BangEqual,
	Greater, GreaterEqual,
	Less, LessEqual,
	Plus, PlusEqual,
	Minus, MinusEqual,
	Star, StarEqual,
	Slash, SlashEqual,
    Percent, PercentEqual,
    
    Pipe, Or, // |, ||
	And, // and -> &&
    Colon, ColonColon,
    Arrow, FatArrow,
    Dot, DotDot,

    // optional
    Question, QuestionDot, 
    QuestionQuestion,
    
	// literals
	Identifier, Decorator,
    Str, Number,
    True, False, Nil,

	// keywords
	If, Else, For, In, While,
	Loop, Class, Trait, Super,
    Return, Break, Continue,
	Let, Const, Fn, As, Do, Match,
    Async, Await, Throw, Try, Catch,
    Type, Enum, Pub,
	
	#[strum(serialize = "self")]
	KwSelf,
    
	Eof,
}

impl std::fmt::Display for TokenType {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
	NumberValue(f64),
	StringValue(String),
    DecoratorValue(String),
	True, False, Nil,
}

impl LiteralValue {
	pub fn from_token(token: Token) -> Self {
		match token.token_type {
			TokenType::Number => match token.literal {
				Some(LiteralValue::NumberValue(n)) => Self::NumberValue(n),
				_ => panic!("Expected number literal"),
			},
			TokenType::Str => match token.literal {
				Some(LiteralValue::StringValue(n)) => Self::StringValue(n),
				_ => panic!("Expected string literal"),
			},
            TokenType::Decorator => match token.literal {
                Some(LiteralValue::DecoratorValue(n)) => Self::DecoratorValue(n),
                _ => panic!("Expected decorator literal")
            }

			TokenType::True => LiteralValue::True,
			TokenType::False => LiteralValue::False,
			TokenType::Nil => LiteralValue::Nil,

			_ => panic!(
				"cannot create a literal value from: '{:?}'",
				token.token_type
			),
		}
	}
}

impl std::fmt::Display for LiteralValue {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			LiteralValue::NumberValue(n) => write!(f, "{n}"),
			LiteralValue::StringValue(s) => write!(f, "{s}"),
			LiteralValue::DecoratorValue(s) => write!(f, "@{s}"),
			LiteralValue::True => write!(f, "true"),
			LiteralValue::False => write!(f, "false"),
			LiteralValue::Nil => write!(f, "nil"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Token {
	pub lexeme: String,
	pub literal: Option<LiteralValue>,
	pub token_type: TokenType,
	pub offset: u32,
	pub line: u32,
}

impl Token {
	pub fn span(&self) -> SourceSpan {
		(self.offset as usize, self.lexeme.len()).into()
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{} {}", self.token_type, self.lexeme)
	}
}
