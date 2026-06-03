use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexError {
    #[error("line {line}: unexpected character '{ch}'")]
    UnexpectedChar { ch: char, line: usize },

    #[error("line {line}: unexpected '&', did you mean '&&'?")]
    SingleAmpersand { line: usize },

    #[error("line {line}: unterminated string")]
    UnterminatedString { line: usize },

    #[error("line {line}: could not parse number '{text}'")]
    InvalidNumber { text: String, line: usize },

    #[error("line {line}: expected identifier after '@'")]
    ExpectedIdentAfterAt { line: usize },
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("const requires initializer")]
    ConstRequiresInitializer,

    #[error("chained assignment is not allowed")]
    ChainedAssignment,

    #[error("invalid assignment target")]
    InvalidAssignTarget,

    #[error("expected expression")]
    ExpectedExpression,

    #[error("{msg}")]
    Expected { msg: &'static str },
}
