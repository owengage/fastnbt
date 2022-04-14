use std::collections::HashMap;

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserialize, Serialize,
};

use crate::{ByteArray, IntArray, LongArray};

/// Value is a complete NBT value. It owns its data. Compounds and Lists are
/// resursively deserialized. This type takes care to preserve all the
/// information from the original NBT, with the exception of the name of the
/// root compound (which is usually the empty string).
///
/// ```no_run
/// # use fastnbt::Value;
/// # use fastnbt::error::Result;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<()> {
/// #   let mut buf = vec![];
///     let compound: HashMap<String, Value> = fastnbt::from_bytes(buf.as_slice())?;
///     match compound["DataVersion"] {
///         Value::Int(ver) => println!("Version: {}", ver),
///         _ => {},
///     }
///     println!("{:#?}", compound);
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(ByteArray),
    IntArray(IntArray),
    LongArray(LongArray),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
}

#[cfg(feature = "arbitrary1")]
fn het_list<'a, T, F>(u: &mut arbitrary::Unstructured<'a>, f: F) -> arbitrary::Result<Vec<Value>>
where
    F: FnMut(T) -> Value,
    T: arbitrary::Arbitrary<'a>,
{
    Ok(u.arbitrary_iter::<T>()?
        .collect::<arbitrary::Result<Vec<_>>>()?
        .into_iter()
        .map(f)
        .collect())
}

#[cfg(feature = "arbitrary1")]
fn arb_list(u: &mut arbitrary::Unstructured) -> arbitrary::Result<Vec<Value>> {
    use crate::Tag;
    use Value::*;

    Ok(match u.arbitrary::<Tag>()? {
        Tag::End => return Err(arbitrary::Error::IncorrectFormat),
        Tag::Byte => het_list(u, Byte)?,
        Tag::Short => het_list(u, Short)?,
        Tag::Int => het_list(u, Int)?,
        Tag::Long => het_list(u, Long)?,
        Tag::Float => het_list(u, Float)?,
        Tag::Double => het_list(u, Double)?,
        Tag::ByteArray => het_list(u, ByteArray)?,
        Tag::String => het_list(u, String)?,
        Tag::List => {
            // make a list of lists
            let len = u.arbitrary_len::<Value>()?;
            let mut v = vec![];
            for _ in 0..len {
                v.push(Value::List(arb_list(u)?));
            }
            v
        }
        Tag::Compound => het_list(u, Compound)?,
        Tag::IntArray => het_list(u, IntArray)?,
        Tag::LongArray => het_list(u, LongArray)?,
    })
}

#[cfg(feature = "arbitrary1")]
impl<'a> arbitrary::Arbitrary<'a> for Value {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        use crate::Tag;
        use Value::*;

        Ok(match u.arbitrary::<Tag>()? {
            Tag::End => return Err(arbitrary::Error::IncorrectFormat),
            Tag::Byte => Byte(u.arbitrary()?),
            Tag::Short => Short(u.arbitrary()?),
            Tag::Int => Int(u.arbitrary()?),
            Tag::Long => Long(u.arbitrary()?),
            Tag::Float => Float(u.arbitrary()?),
            Tag::Double => Double(u.arbitrary()?),
            Tag::ByteArray => ByteArray(u.arbitrary()?),
            Tag::String => String(u.arbitrary()?),
            Tag::Compound => Compound(u.arbitrary()?),
            Tag::IntArray => IntArray(u.arbitrary()?),
            Tag::LongArray => LongArray(u.arbitrary()?),

            // Lists need to all be the same type.
            Tag::List => List(arb_list(u)?),
        })
    }
}

impl Value {
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Byte(v) => Some(v as i64),
            Value::Short(v) => Some(v as i64),
            Value::Int(v) => Some(v as i64),
            Value::Long(v) => Some(v as i64),
            Value::Float(v) => Some(v as i64),
            Value::Double(v) => Some(v as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Byte(v) => Some(v as u64),
            Value::Short(v) => Some(v as u64),
            Value::Int(v) => Some(v as u64),
            Value::Long(v) => Some(v as u64),
            Value::Float(v) => Some(v as u64),
            Value::Double(v) => Some(v as u64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Byte(v) => Some(v as f64),
            Value::Short(v) => Some(v as f64),
            Value::Int(v) => Some(v as f64),
            Value::Long(v) => Some(v as f64),
            Value::Float(v) => Some(v as f64),
            Value::Double(v) => Some(v as f64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Byte(v) => serializer.serialize_i8(*v),
            Value::Short(v) => serializer.serialize_i16(*v),
            Value::Int(v) => serializer.serialize_i32(*v),
            Value::Long(v) => serializer.serialize_i64(*v),
            Value::Float(v) => serializer.serialize_f32(*v),
            Value::Double(v) => serializer.serialize_f64(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::ByteArray(v) => v.serialize(serializer),
            Value::IntArray(v) => v.serialize(serializer),
            Value::LongArray(v) => v.serialize(serializer),
            Value::List(v) => v.serialize(serializer),
            Value::Compound(v) => v.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid NBT")
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Byte(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Short(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Int(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Long(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Double(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.to_owned()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // I think I either need to do the same trick as I did for NBT
                // Arrays and have dedicated types to this, or just accept that
                // it's a Vec<Value> and that you can construct invalid NBT with
                // it (by adding values of different type).

                // Feel like the case of list of lists will break me if I try
                // the dedicated type.
                let mut v = Vec::<Value>::with_capacity(seq.size_hint().unwrap_or(0));

                while let Some(el) = seq.next_element()? {
                    v.push(el);
                }

                Ok(Value::List(v))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                match map.next_key_seed(KeyClassifier)? {
                    Some(KeyClass::Compound(first_key)) => {
                        let mut compound = HashMap::new();

                        compound.insert(first_key, map.next_value()?);
                        while let Some((key, value)) = map.next_entry()? {
                            compound.insert(key, value);
                        }

                        Ok(Value::Compound(compound))
                    }
                    Some(KeyClass::ByteArray) => {
                        let data = map.next_value::<&[u8]>()?;
                        Ok(Value::ByteArray(ByteArray::from_bytes(data)))
                    }
                    Some(KeyClass::IntArray) => {
                        let data = map.next_value::<&[u8]>()?;
                        IntArray::from_bytes(data)
                            .map(Value::IntArray)
                            .map_err(|_| serde::de::Error::custom("could not read int array"))
                    }
                    Some(KeyClass::LongArray) => {
                        let data = map.next_value::<&[u8]>()?;
                        LongArray::from_bytes(data)
                            .map(Value::LongArray)
                            .map_err(|_| serde::de::Error::custom("could not read long array"))
                    }
                    // No keys just means an empty compound.
                    None => Ok(Value::Compound(Default::default())),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

struct KeyClassifier;

enum KeyClass {
    Compound(String),
    ByteArray,
    IntArray,
    LongArray,
}

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, deserializer: D) -> Result<KeyClass, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an nbt field string")
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match s.as_str() {
            crate::BYTE_ARRAY_TOKEN => Ok(KeyClass::ByteArray),
            crate::INT_ARRAY_TOKEN => Ok(KeyClass::IntArray),
            crate::LONG_ARRAY_TOKEN => Ok(KeyClass::LongArray),
            _ => Ok(KeyClass::Compound(s)),
        }
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match s {
            crate::BYTE_ARRAY_TOKEN => Ok(KeyClass::ByteArray),
            crate::INT_ARRAY_TOKEN => Ok(KeyClass::IntArray),
            crate::LONG_ARRAY_TOKEN => Ok(KeyClass::LongArray),
            _ => Ok(KeyClass::Compound(s.to_string())),
        }
    }
}

//
// Partial Eq impls. Everything below is copied from serde_json,
// https://github.com/serde-rs/json/blob/5d2cbcdd4b146e98b5aa2200de7a8ae6231bf0ba/src/value/partial_eq.rs
//
// For which the license is MIT:
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

fn eq_i64(value: &Value, other: i64) -> bool {
    value.as_i64().map_or(false, |i| i == other)
}

fn eq_u64(value: &Value, other: u64) -> bool {
    value.as_u64().map_or(false, |i| i == other)
}

fn eq_f64(value: &Value, other: f64) -> bool {
    value.as_f64().map_or(false, |i| i == other)
}

fn eq_str(value: &Value, other: &str) -> bool {
    value.as_str().map_or(false, |i| i == other)
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        eq_str(self, other)
    }
}

impl<'a> PartialEq<&'a str> for Value {
    fn eq(&self, other: &&str) -> bool {
        eq_str(self, *other)
    }
}

impl PartialEq<Value> for str {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, self)
    }
}

impl<'a> PartialEq<Value> for &'a str {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, *self)
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        eq_str(self, other.as_str())
    }
}

impl PartialEq<Value> for String {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, self.as_str())
    }
}

macro_rules! partialeq_numeric {
    ($($eq:ident [$($ty:ty)*])*) => {
        $($(
            impl PartialEq<$ty> for Value {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(self, *other as _)
                }
            }

            impl PartialEq<Value> for $ty {
                fn eq(&self, other: &Value) -> bool {
                    $eq(other, *self as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a Value {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a mut Value {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }
        )*)*
    }
}

partialeq_numeric! {
    eq_i64[i8 i16 i32 i64 isize]
    eq_u64[u8 u16 u32 u64 usize]
    eq_f64[f32 f64]
}
