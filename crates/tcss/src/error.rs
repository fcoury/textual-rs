use thiserror::Error;

#[derive(Error, Debug)]
pub enum TcssError {
    #[error("CSS syntax error: {0}")]
    InvalidSyntax(String),
    #[error("Unknown variable: {0}")]
    UnknownVariable(String),
    #[error("I/O error reading stylesheet")]
    Io(#[from] std::io::Error),
}
