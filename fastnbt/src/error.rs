//! Contains the Error and Result type used by the deserializer.
use std::fmt::Display;

/// Various errors that can occur during deserialization.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Serde error, or other unexpected error.
    Message(String),
    /// An underlying IO error.
    IO(String),
    /// Tag was not a valid NBT tag.
    InvalidTag(u8),
    /// String or name was not valid unicode.
    NonunicodeString(Vec<u8>),
    /// No root compound was found.
    NoRootCompound,
    /// A list/array was an invalid size (likely negative)
    InvalidSize(i32),
}

/// Convenience type for Result.
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
            Error::NonunicodeString(_) => f.write_str("string or name was not valid utf-8"),
            Error::NoRootCompound => {
                f.write_str("require a root compound to start deserialization")
            }
            Error::InvalidSize(size) => f.write_fmt(format_args!("list size was {}", size)),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err.to_string())
    }
}

impl std::error::Error for Error {}
