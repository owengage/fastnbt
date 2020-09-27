use std::fmt::Display;
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    Eof,
    TrailingCharacters,
}

pub type Result<T> = std::result::Result<T, Error>;

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Eof => f.write_str("unexpected end of input"),
            Error::TrailingCharacters => f.write_str("trailing characters"),
        }
    }
}

impl std::error::Error for Error {}