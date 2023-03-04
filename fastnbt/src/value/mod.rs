mod array_serializer;
mod de;
mod ser;

use std::collections::HashMap;

use serde::{serde_if_integer128, Deserialize, Serialize};

use crate::{error::Error, ByteArray, IntArray, LongArray};

pub use self::ser::Serializer;

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
            Value::Long(v) => Some(v),
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
            Value::Double(v) => Some(v),
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

// ------------- From<T> impls -------------

macro_rules! from {
    ($type:ty, $variant:ident $(, $($part:tt)+)?) => {
        impl From<$type> for Value {
            fn from(val: $type) -> Self {
                Self::$variant(val$($($part)+)?)
            }
        }
        impl From<&$type> for Value {
            fn from(val: &$type) -> Self {
                Self::$variant(val.to_owned()$($($part)+)?)
            }
        }
    };
}
from!(i8, Byte);
from!(u8, Byte, as i8);
from!(i16, Short);
from!(u16, Short, as i16);
from!(i32, Int);
from!(u32, Int, as i32);
from!(i64, Long);
from!(u64, Long, as i64);
from!(f32, Float);
from!(f64, Double);
from!(String, String);
from!(&str, String, .to_owned());
from!(ByteArray, ByteArray);
from!(IntArray, IntArray);
from!(LongArray, LongArray);

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Self::Byte(i8::from(val))
    }
}
impl From<&bool> for Value {
    fn from(val: &bool) -> Self {
        Self::Byte(i8::from(*val))
    }
}

//
// Everything below is copied from serde_json,
// Partial Eq impls: https://github.com/serde-rs/json/blob/5d2cbcdd4b146e98b5aa2200de7a8ae6231bf0ba/src/value/partial_eq.rs
// to/from_value(): https://github.com/serde-rs/json/blob/52a9c050f5dcc0dc3de4825b131b8ff05219cc82/src/value/mod.rs#L886-L989
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
        eq_str(self, other)
    }
}

impl PartialEq<Value> for str {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, self)
    }
}

impl<'a> PartialEq<Value> for &'a str {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, self)
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

macro_rules! from_128bit {
    ($($type:ty),+) => {
        $(
            impl From<$type> for Value {
                fn from(val: $type) -> Self {
                    Self::IntArray(IntArray::new(vec![
                        (val >> 96) as i32,
                        (val >> 64) as i32,
                        (val >> 32) as i32,
                        val as i32,
                    ]))
                }
            }

            impl From<&$type> for Value {
                fn from(val: &$type) -> Self {
                    Value::from(*val)
                }
            }
        )+
    };
}
serde_if_integer128! {
    from_128bit!(i128, u128);
}

/// Convert a `T` into `fastnbt::Value` which is an enum that can represent
/// any valid NBT data.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use fastnbt::nbt;
///
/// use std::error::Error;
///
/// #[derive(Serialize)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// fn compare_nbt_values() -> Result<(), Box<dyn Error>> {
///     let u = User {
///         fingerprint: "0xF9BA143B95FF6D82".to_owned(),
///         location: "Menlo Park, CA".to_owned(),
///     };
///
///     // The type of `expected` is `fastnbt::Value`
///     let expected = nbt!({
///         "fingerprint": "0xF9BA143B95FF6D82",
///         "location": "Menlo Park, CA",
///     });
///
///     let v = fastnbt::to_value(u).unwrap();
///     assert_eq!(v, expected);
///
///     Ok(())
/// }
/// #
/// # compare_nbt_values().unwrap();
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
///
/// ```
/// use std::collections::BTreeMap;
///
/// // The keys in this map are vectors, not strings.
/// let mut map = BTreeMap::new();
/// map.insert(vec![32, 64], "x86");
///
/// println!("{}", fastnbt::to_value(map).unwrap_err());
/// ```
pub fn to_value<T>(value: T) -> Result<Value, Error>
where
    T: Serialize,
{
    value.serialize(&mut Serializer)
}

/// Interpret a `fastnbt::Value` as an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use fastnbt::nbt;
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// // The type of `j` is `fastnbt::Value`
/// let j = nbt!({
///     "fingerprint": "0xF9BA143B95FF6D82",
///     "location": "Menlo Park, CA"
/// });
///
/// let u: User = fastnbt::from_value(&j).unwrap();
/// println!("{:#?}", u);
/// ```
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct type but the Value
/// contains something other than an NBT compound. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong with the data, for example required struct fields are missing from
/// the NBT compound or some number is too big to fit in the expected primitive
/// type.
pub fn from_value<'de, T>(value: &'de Value) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    T::deserialize(value)
}
