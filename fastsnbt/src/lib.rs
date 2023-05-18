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

pub fn to_vec<T: ?Sized + Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut serializer = Serializer { writer: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.writer)
}

pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}
