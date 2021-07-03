use std::collections::HashMap;

use serde::{de::Visitor, Deserialize};

/// Value is a complete NBT value. It owns it's data. This type does not allow
/// you to disinguish between Lists of types and the 'Array' version of the
/// type. So a List of Int is indistiguishable from IntArray.
///
/// ```
/// use fastnbt::Value;
/// # use fastnbt::error::Result;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<()> {
/// #   let buf = [10, 0, 0, 3, 0, 11, 68, 97, 116, 97, 86, 101, 114, 115, 105, 111, 110, 0, 0, 0, 0, 0];
///     let compound: HashMap<String, Value> = fastnbt::de::from_bytes(&buf)?;
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
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Double(f64),
    Float(f32),
    String(String),
    ByteArray(Vec<u8>),
    // The reason these don't work is because my deserializer is happy to parse
    // int and long arrays are bytes, entirely because we wanted to enable
    // getting these arrays without allocations. We don't actually use this
    // functionality anymore though. I could remove the ability?
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value2 {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    List(Vec<Value2>),
    Compound(HashMap<String, Value2>),
    ByteArray(Vec<u8>),
}
impl<'de> Deserialize<'de> for Value2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVis;
        impl<'de> Visitor<'de> for ValueVis {
            type Value = Value2;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "an NBT compatible value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i8(v as i8)
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::Byte(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::Short(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::Int(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::Long(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::Float(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::Double(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.into())
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.into())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value2::String(v))
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_bytes(v)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut values = vec![];

                while let Some(el) = seq.next_element::<Value2>()? {
                    values.push(el);
                }

                Ok(Value2::List(values))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut obj = HashMap::new();

                while let Some((k, v)) = map.next_entry::<String, Value2>()? {
                    obj.insert(k, v);
                }

                Ok(Value2::Compound(obj))
            }
        }

        deserializer.deserialize_any(ValueVis)
    }
}

// Plan:
// Make a pedantic value that we decode into. Taggless. Might need to order
// these tags from largest int type to smallest to make it work.
