use super::Tag;
use std::fmt::Display;
#[derive(Debug)]
pub enum Error {
    Message(String),
    IO(std::io::Error),
    InvalidTag(u8),
    InvalidName,
    UnexpectedNegativeInt,
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
            Error::UnexpectedNegativeInt => f.write_str("unexpected negative integer"),
            Error::Eof => f.write_str("unexpected end of input"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(e: std::num::TryFromIntError) -> Self {
        Error::Message(format!("{}", e))
    }
}

impl std::error::Error for Error {}
