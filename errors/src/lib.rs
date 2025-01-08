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
}

pub type CQRLResult<T> = Result<T, CQRLError>;
