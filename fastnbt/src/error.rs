//! Contains the Error and Result type used by the deserializer.
use std::fmt::Display;
use thiserror::Error as ThisError;

/// Various errors that can occur during deserialization.
#[derive(Debug, ThisError)]
#[non_exhaustive]
pub enum Error {
    /// Serde error, or other unexpected error.
    #[error("serde error")]
    Message(String),

    /// An underlying IO error.
    #[error("io")]
    Io {
        #[from]
        source: std::io::Error,
    },

    /// Tag was not a valid NBT tag.
    #[error("invalid NBT tag: {}", .0)]
    InvalidTag(u8),

    /// String or name was not valid unicode.
    #[error("string or name was not utf-8")]
    NonunicodeString(Vec<u8>),

    /// No root compound was found.
    #[error("require root compound for deserialization")]
    NoRootCompound,

    /// A list/array was an invalid size (likely negative)
    #[error("list or array was invalid size: {}", .0)]
    InvalidSize(i32),
}

/// Convenience type for Result.
pub type Result<T> = std::result::Result<T, Error>;

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
