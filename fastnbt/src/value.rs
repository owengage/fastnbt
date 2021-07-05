use std::collections::HashMap;

use serde::Deserialize;

use crate::{ByteArray, IntArray, LongArray};

/// Value is a complete NBT value. It owns it's data. The Byte, Short, Int and
/// Long NBT types are all deserialized into `i64`. Compounds and Lists are
/// resursively deserialized.
///
/// ```no_run
/// # use fastnbt::Value;
/// # use fastnbt::error::Result;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<()> {
/// #   let mut buf = vec![];
///     let compound: HashMap<String, Value> = fastnbt::de::from_bytes(buf.as_slice())?;
///     match compound["DataVersion"] {
///         Value::Int(ver) => println!("Version: {}", ver),
///         _ => {},
///     }
///     println!("{:#?}", compound);
/// #   Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    #[serde(deserialize_with = "strict_i8")]
    Byte(i8),
    #[serde(deserialize_with = "strict_i16")]
    Short(i16),
    #[serde(deserialize_with = "strict_i32")]
    Int(i32),
    Long(i64),
    Double(f64),
    Float(f32),
    String(String),
    ByteArray(ByteArray),
    IntArray(IntArray),
    LongArray(LongArray),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
}

fn strict_i8<'de, D>(de: D) -> std::result::Result<i8, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct StrictI8Visitor;
    impl<'de> serde::de::Visitor<'de> for StrictI8Visitor {
        type Value = i8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i8")
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i8(StrictI8Visitor)
}

fn strict_i16<'de, D>(de: D) -> std::result::Result<i16, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Stricti16Visitor;
    impl<'de> serde::de::Visitor<'de> for Stricti16Visitor {
        type Value = i16;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i16")
        }

        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i16(Stricti16Visitor)
}

fn strict_i32<'de, D>(de: D) -> std::result::Result<i32, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Stricti32Visitor;
    impl<'de> serde::de::Visitor<'de> for Stricti32Visitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expecting exactly i32")
        }

        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }
    }

    de.deserialize_i32(Stricti32Visitor)
}
