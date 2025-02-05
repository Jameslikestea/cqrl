use std::error::Error;

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum CQRLError {
    #[snafu(display("Ooops... An error occurred, but we don't have anymore information..."))]
    Generic,

    #[snafu(display("Cannot convert document into tokens: {error}"))]
    LexError { error: Box<dyn Error> },
    #[snafu(display("Contents of file invalid, cannot parse"))]
    ParseError,
    #[snafu(display("Invalid type used in model property"))]
    ModelTypes,
    #[snafu(display("Invalid type used in query"))]
    QueryTypes,
    #[snafu(display("Invalid type used in command"))]
    CommandTypes,
    #[snafu(display("Cannot store object in store"))]
    StoreError,
}

pub type CQRLResult<T> = Result<T, CQRLError>;
