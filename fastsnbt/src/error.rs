//! Contains the Error and Result type used by the deserializer.
use std::fmt::Display;

/// Various errors that can occur during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(String);

/// Convenience type for Result.
pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

// TODO: Separate error types for ser and de?
impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error(msg.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error(format!("io error: {}", e))
    }
}

impl Error {
    pub(crate) fn invalid_input(pos: usize) -> Error {
        Error(format!("invalid input at {}", pos))
    }

    pub(crate) fn input_not_consumed() -> Error {
        Error("Input wasn't fully consumed".into())
    }

    pub(crate) fn expected_comma() -> Error {
        Error("expected comma".into())
    }

    pub(crate) fn expected_colon() -> Error {
        Error("expected colon".into())
    }

    pub(crate) fn expected_array_end() -> Error {
        Error("expected array end".into())
    }

    pub(crate) fn expected_map_end() -> Error {
        Error("expected compound tag end".into())
    }

    pub(crate) fn unexpected_eof() -> Error {
        Error("eof: unexpectedly ran out of input".to_owned())
    }

    pub(crate) fn array_as_other() -> Error {
        Error("expected NBT Array: use ByteArray, IntArray or LongArray types".into())
    }

    pub(crate) fn bespoke(msg: String) -> Error {
        Error(msg)
    }
}
