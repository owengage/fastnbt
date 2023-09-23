//! `fastsnbt` aims for fast deserializing and serializing of
//! sNBT data from *Minecraft: Java Edition*.
//! This is the human-readable equivalent of NBT data. (see
//! [`fastnbt`](https://crates.io/crates/fastnbt)).
//!
//! - For documentation of serde (de)serialization, see [`ser`] and [`de`].
//! - See [`fastnbt`](https://crates.io/crates/fastnbt) for most
//!   NBT related things.
//!
//! # Example
//! ```
//! # use serde::{Serialize, Deserialize};
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct SimpleStruct {
//!     num: i64,
//!     s: String,
//! }
//!
//! let data = SimpleStruct {
//!     num: 31300,
//!     s: "Hello world!".into(),
//! };
//! let ser = fastsnbt::to_string(&data).unwrap();
//! assert_eq!("{\"num\":31300l,\"s\":\"Hello world!\"}", ser);
//!
//! let input = "{num:31300L,s:\"Hello world!\"}";
//! let de: SimpleStruct = fastsnbt::from_str(input).unwrap();
//! assert_eq!(data, de);
//! ```

use de::Deserializer;
use error::Result;
use ser::Serializer;
use serde::Serialize;

pub mod ser;
pub mod de;
pub mod error;
pub(crate) mod parser;

pub(crate) const BYTE_ARRAY_TOKEN_STR: &str = "\"__fastnbt_byte_array\"";
pub(crate) const INT_ARRAY_TOKEN_STR: &str = "\"__fastnbt_int_array\"";
pub(crate) const LONG_ARRAY_TOKEN_STR: &str = "\"__fastnbt_long_array\"";
pub(crate) const BYTE_ARRAY_TOKEN: &str = "__fastnbt_byte_array";
pub(crate) const INT_ARRAY_TOKEN: &str = "__fastnbt_int_array";
pub(crate) const LONG_ARRAY_TOKEN: &str = "__fastnbt_long_array";

#[cfg(test)]
mod tests;

/// Deserialize into a `T` from some sNBT data. See the
/// [`de`] module for more information.
pub fn from_str<'a, T>(input: &'a str) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    let mut des = Deserializer::from_str(input);
    let t = T::deserialize(&mut des)?;
    if !des.input.is_empty() {
        return Err(error::Error::input_not_consumed());
    }
    Ok(t)
}

/// Serialize some `T` into some sNBT string. This produces
/// valid utf-8. See the [`ser`] module for more information.
pub fn to_vec<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut serializer = Serializer { writer: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.writer)
}

/// Serialize some `T` into a sNBT string. See the [`ser`]
/// module for more information.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}
