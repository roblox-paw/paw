use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::lexer::TokenType;

#[derive(Debug, Error, Diagnostic)]
#[error("compilation failed with {} error(s)", errors.len())]
pub struct CompileErrors<E>
where
    E: Diagnostic + Send + Sync + 'static,
{
    #[related]
    pub errors: Vec<E>,
}

impl<E: Diagnostic + Send + Sync + 'static> CompileErrors<E> {
    pub fn new(errors: Vec<E>) -> Self {
        Self { errors }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(severity(Error))]
pub enum LexError {
    #[error("unexpected character '{ch}'")]
    #[diagnostic(code(paw::lexer::unexpected_char))]
    UnexpectedChar {
        ch: char,
        #[label("not valid here")]
        span: SourceSpan,
    },

    #[error("unexpected '&', did you mean '&&'?")]
    #[diagnostic(
        code(paw::lexer::single_ampersand),
        help("paw uses '&&' for logical and; there is no single '&' operator")
    )]
    SingleAmpersand {
        #[label("expected another '&' here")]
        span: SourceSpan,
    },

    #[error("unterminated string")]
    #[diagnostic(
        code(paw::lexer::unterminated_string),
        help("add a closing '\"' to end the string")
    )]
    UnterminatedString {
        #[label("string starts here")]
        span: SourceSpan,
    },

    #[error("could not parse number '{text}'")]
    #[diagnostic(code(paw::lexer::invalid_number))]
    InvalidNumber {
        text: String,
        #[label("not a valid number")]
        span: SourceSpan,
    },

    #[error("expected an identifier after '@'")]
    #[diagnostic(code(paw::lexer::expected_ident_after_at))]
    ExpectedIdentAfterAt {
        #[label("expected a decorator name here")]
        span: SourceSpan,
    },
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(severity(Error))]
pub enum ParseError {
    #[error("expected ',' or '}}' in table")]
    #[diagnostic(
        code(paw::parser::table_expected_comma),
        help("separate table fields with ',' and close the table with '}}'")
    )]
    TableExpectedComma {
        #[label("expected ',' or '}}' after this field")]
        span: SourceSpan,
    },

    #[error("constant variable requires an initializer")]
    #[diagnostic(code(paw::parser::const_requires_initializer))]
    ConstRequiresInitializer {
        #[label("this 'const' has no '= value'")]
        span: SourceSpan,
    },

    #[error("chained assignment is not allowed")]
    #[diagnostic(
        code(paw::parser::chained_assignment),
        help("assign one value at a time")
    )]
    ChainedAssignment {
        #[label("unexpected second assignment")]
        span: SourceSpan,
    },

    // todo: suggest closest defined name on assignment to unknown variable
    #[error("invalid assignment target")]
    #[diagnostic(
        code(paw::parser::invalid_assign_target),
        help("the left side of '=' must be an expression or variable")
    )]
    InvalidAssignTarget {
        #[label("cannot assign to this")]
        span: SourceSpan,
    },

    #[error("expected an expression")]
    #[diagnostic(code(paw::parser::expected_expression))]
    ExpectedExpression {
        #[label("expected an expression here")]
        span: SourceSpan,
    },

    #[error("expected {expected}, found {found}")]
    #[diagnostic(code(paw::parser::expected_token))]
    ExpectedToken {
        expected: TokenType,
        found: TokenType,
        #[label("unexpected token")]
        span: SourceSpan,
        #[help]
        advice: Option<String>,
    },

    #[error("expected {what}")]
    #[diagnostic(code(paw::parser::expected_construct))]
    #[allow(unused)] // remove that later
    ExpectedConstruct {
        what: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[help]
        advice: Option<String>,
    },

    #[error("'if' requires a condition before the body")]
    #[diagnostic(
        code(paw::parser::if_missing_condition),
        help("write a condition before '{{'")
    )]
    IfMissingCondition {
        #[label("condition expected here, found '{{'")]
        span: SourceSpan,
    },

    #[error("if expression requires an 'else' branch")]
    #[diagnostic(
        code(paw::parser::if_expr_requires_else),
        help("'if' used as a value must always produce one - add '}} else {{ ... }}'")
    )]
    IfExprRequiresElse {
        #[label("this 'if' has no 'else' statement")]
        span: SourceSpan,
    },

    #[error("'while' requires a condition before the body")]
    #[diagnostic(
        code(paw::parser::while_missing_condition),
        help("write a condition before '{{'")
    )]
    WhileMissingCondition {
        #[label("condition expected here, found '{{'")]
        span: SourceSpan,
    },

    #[error("for loop requires at least one binding")]
    #[diagnostic(
        code(paw::parser::for_invalid_syntax),
        help("expected 'for <var(s)> in <expr> ...'")
    )]
    ForMissingVars {
        #[label("expected an identifier here")]
        span: SourceSpan,
    },

    #[error("expression has no effect")]
    #[diagnostic(
        code(paw::parser::expression_has_no_effect),
        help("only assignments and function calls can be used as statements")
    )]
    ExpressionHasNoEffect {
        #[label("this expression does nothing")]
        span: SourceSpan,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_error_message_includes_char() {
        let err = LexError::UnexpectedChar {
            ch: '$',
            span: (8, 1).into(),
        };
        assert_eq!(err.to_string(), "unexpected character '$'");
    }

    #[test]
    fn lex_error_unterminated_string_message() {
        let err = LexError::UnterminatedString {
            span: (10, 6).into(),
        };
        assert_eq!(err.to_string(), "unterminated string");
    }

    #[test]
    fn lex_error_carries_help_text() {
        let err = LexError::SingleAmpersand {
            span: (3, 1).into(),
        };
        let help = err.help().map(|h| h.to_string());
        assert_eq!(
            help.as_deref(),
            Some("paw uses '&&' for logical and; there is no single '&' operator")
        );
    }

    #[test]
    fn lex_error_has_diagnostic_code() {
        let err = LexError::UnexpectedChar {
            ch: '#',
            span: (0, 1).into(),
        };
        let code = err.code().map(|c| c.to_string());
        assert_eq!(code.as_deref(), Some("paw::lexer::unexpected_char"));
    }

    #[test]
    fn parse_error_expected_token_message() {
        let err = ParseError::ExpectedToken {
            expected: TokenType::RightParen,
            found: TokenType::Eof,
            span: (12, 1).into(),
            advice: None,
        };
        assert_eq!(err.to_string(), "expected RightParen, found Eof");
    }

    #[test]
    fn parse_error_expected_token_with_advice() {
        let err = ParseError::ExpectedToken {
            expected: TokenType::RightParen,
            found: TokenType::Eof,
            span: (12, 1).into(),
            advice: Some("did you forget to close the parenthesis?".to_string()),
        };
        assert_eq!(
            err.help().map(|h| h.to_string()).as_deref(),
            Some("did you forget to close the parenthesis?")
        );
    }

    #[test]
    fn parse_error_const_no_help() {
        let err = ParseError::ConstRequiresInitializer {
            span: (0, 5).into(),
        };
        assert!(err.help().is_none());
    }

    #[test]
    fn compile_errors_message() {
        let errs: CompileErrors<LexError> = CompileErrors::new(vec![
            LexError::UnterminatedString { span: (0, 1).into() },
            LexError::SingleAmpersand { span: (5, 1).into() },
        ]);
        assert_eq!(errs.to_string(), "compilation failed with 2 error(s)");
    }

    #[test]
    #[should_panic]
    fn unwrap_missing_help_panics() {
        let err = ParseError::ExpectedExpression {
            span: (4, 1).into(),
        };
        err.help().expect("no help text on this variant");
    }
}
