use super::Tag;
use std::fmt::Display;
#[derive(Debug, PartialEq)]
pub enum Error {
    Message(String),
    IO(String),
    InvalidTag(u8),
    InvalidName,
    IntegralOutOfRange,
    TypeMismatch(Tag, &'static str), // expected type by tag, expected serde type.
    Eof,
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
            Error::IO(e) => f.write_fmt(format_args!("{}", e)),
            Error::InvalidTag(tag) => f.write_fmt(format_args!("Invalid tag number: {}", tag)),
            Error::InvalidName => f.write_str("invalid name"),
            Error::TypeMismatch(t, s) => f.write_fmt(format_args!(
                "expecting {}, found type to have tag {:?}",
                s, t
            )),
            Error::IntegralOutOfRange => f.write_str("integral value did not fit in receiver type"),
            Error::Eof => f.write_str("unexpected end of input"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err.to_string())
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(_e: std::num::TryFromIntError) -> Self {
        Error::IntegralOutOfRange
    }
}

impl std::error::Error for Error {}
