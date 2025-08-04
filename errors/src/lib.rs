use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum CQRLError {
    #[snafu(display("Ooops... An error occurred, but we don't have anymore information..."))]
    Generic,
    #[snafu(display("Cannot convert document into tokens: {error}"))]
    LexError { error: String },
    #[snafu(display("Contents of file invalid, cannot parse"))]
    ParseError,
    #[snafu(display("Invalid type used in model property"))]
    ModelTypes,
    #[snafu(display("Invalid type used in query"))]
    QueryTypes,
    #[snafu(display("Invalid type used in command"))]
    CommandTypes,
    #[snafu(display("Cannot store object in store: {error}"))]
    StoreError { error: String },
    #[snafu(display("Invalid event type"))]
    InvalidEventType,
    #[snafu(display("No event data"))]
    NoEventData,
    #[snafu(display("Required field not set: {name}"))]
    RequiredFieldNotSet { name: String },
    #[snafu(display("Incorrect type for field: {name}, expected {ty}"))]
    IncorrectTypeForField { name: String, ty: String },
    #[snafu(display("Permission denied"))]
    PermissionDenied,
}

pub type CQRLResult<T> = Result<T, CQRLError>;
