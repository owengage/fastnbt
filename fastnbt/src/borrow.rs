//! This module contains types enabling 'zero-copy' capture of the array NBT
//! types. These types retain a reference to the input data when deserializing,
//! meaning the input has to live as long as the deserialized object. This can
//! be hard to manage, but offers potential performance improvements. Measure!
//! Usually the dominating factor in deserialization is decompressing the NBT
//! data.
//!
//! The [`ByteArray`], [`IntArray`], and [`LongArray`] types are the types to
//! use in your own data structures. They all implement an `iter()` method to
//! allow you to iterate over the data they contain.
//!
//! For versions that own their data, see
//! `fasnbt::{`[`ByteArray`][`crate::ByteArray`],
//! [`IntArray`][`crate::IntArray`], [`LongArray`][`crate::LongArray`]`}`.
//!
//! The `iter()` methods return an iterator to the values read on demand from an
//! internal reference to the input data.
//!
//! # Example
//!
//! ```no_run
//! use fastnbt::borrow::LongArray;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! #[serde(rename_all = "PascalCase")]
//! pub struct Section<'a> {
//!     #[serde(borrow)]
//!     pub block_states: Option<LongArray<'a>>,
//!     pub y: i8,
//! }
//!
//!# fn main(){
//!     let buf: &[u8] = unimplemented!("get a buffer from somewhere");
//!     let section: Section = fastnbt::from_bytes(buf).unwrap();
//!     let states = section.block_states.unwrap();
//!
//!     for long in states.iter() {
//!         // do something
//!     }
//!# }

use std::{borrow::Cow, fmt, marker::PhantomData};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_bytes::Bytes;

use crate::{BYTE_ARRAY_TOKEN, INT_ARRAY_TOKEN, LONG_ARRAY_TOKEN};

/// ByteArray can be used to deserialize the NBT data of the same name. This
/// borrows from the original input data when deserializing. The carving masks
/// in a chunk use this type, for example.
#[derive(Debug, Clone, Copy)]
pub struct ByteArray<'a> {
    data: &'a [u8],
}

impl<'a> ByteArray<'a> {
    /// Create an iterator over the bytes.
    pub fn iter(&self) -> ByteIter<'a> {
        ByteIter(*self)
    }

    pub fn new(data: &'a [i8]) -> Self {
        let (_, data, _) = unsafe { data.align_to::<u8>() };
        Self { data }
    }

    pub(crate) fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for ByteArray<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor<'a>(PhantomData<&'a ()>);
        impl<'a, 'de: 'a> Visitor<'de> for InnerVisitor<'a> {
            type Value = ByteArray<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("byte array")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let token = map.next_key::<&str>()?.ok_or_else(|| {
                    serde::de::Error::custom("expected NBT byte array token, but got empty map")
                })?;
                let data = map.next_value::<&[u8]>()?;

                if token == BYTE_ARRAY_TOKEN {
                    Ok(ByteArray::from_bytes(data))
                } else {
                    Err(serde::de::Error::custom("expected NBT byte array token"))
                }
            }
        }
        deserializer.deserialize_newtype_struct(BYTE_ARRAY_TOKEN, InnerVisitor(PhantomData))
    }
}

pub struct ByteIter<'a>(ByteArray<'a>);

impl<'a> Iterator for ByteIter<'a> {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i8().ok()
    }
}

impl<'a> Serialize for ByteArray<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // We can't know anything about NBT here, since we might be serializing
        // to a different format. But we can create a hidden inner type to
        // signal the serializer.
        #[derive(Serialize)]
        #[allow(non_camel_case_types)]
        enum Inner<'a> {
            __fastnbt_byte_array(&'a Bytes),
        }

        // Alignment of i64 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.align_to::<u8>() };

        let array = Inner::__fastnbt_byte_array(Bytes::new(data));

        array.serialize(serializer)
    }
}

/// IntArray can be used to deserialize the NBT data of the same name. This
/// borrows from the original input data when deserializing. Biomes in the chunk
/// format are an example of this data type.
#[derive(Debug, Clone, Copy)]
pub struct IntArray<'a> {
    data: &'a [u8],
}

impl<'a> IntArray<'a> {
    /// Create an iterator over the i32s
    pub fn iter(&self) -> IntIter<'a> {
        IntIter(*self)
    }

    pub fn new(data: &'a [i32]) -> Self {
        let (_, data, _) = unsafe { data.align_to::<u8>() };
        Self { data }
    }

    pub(crate) fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for IntArray<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor<'a>(PhantomData<&'a ()>);
        impl<'a, 'de: 'a> Visitor<'de> for InnerVisitor<'a> {
            type Value = IntArray<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("int array")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let token = map.next_key::<&str>()?.ok_or_else(|| {
                    serde::de::Error::custom("expected NBT int array token, but got empty map")
                })?;
                let data = map.next_value::<&[u8]>()?;

                if token == INT_ARRAY_TOKEN {
                    Ok(IntArray::from_bytes(data))
                } else {
                    Err(serde::de::Error::custom("expected NBT int array token"))
                }
            }
        }
        deserializer.deserialize_newtype_struct(INT_ARRAY_TOKEN, InnerVisitor(PhantomData))
    }
}

pub struct IntIter<'a>(IntArray<'a>);

impl<'a> Iterator for IntIter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i32::<BigEndian>().ok()
    }
}

impl<'a> Serialize for IntArray<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // We can't know anything about NBT here, since we might be serializing
        // to a different format. But we can create a hidden inner type to
        // signal the serializer.
        #[derive(Serialize)]
        #[allow(non_camel_case_types)]
        enum Inner<'a> {
            __fastnbt_int_array(&'a Bytes),
        }

        // Alignment of i64 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.align_to::<u8>() };

        let array = Inner::__fastnbt_int_array(Bytes::new(data));

        array.serialize(serializer)
    }
}

/// LongArray can be used to deserialize the NBT data of the same name. This
/// borrows from the original input data when deserializing. Block states
/// (storage of all the blocks in a chunk) are an exmple of when this is used.
#[derive(Debug, Clone, Copy)]
pub struct LongArray<'a> {
    data: &'a [u8],
}

impl<'a> LongArray<'a> {
    /// Create an iterator over the i64s
    pub fn iter(&self) -> LongIter<'a> {
        LongIter(*self)
    }

    pub fn new(data: &'a [i64]) -> Self {
        let (_, data, _) = unsafe { data.align_to::<u8>() };
        Self { data }
    }

    pub(crate) fn from_bytes(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for LongArray<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor<'a>(PhantomData<&'a ()>);
        impl<'a, 'de: 'a> Visitor<'de> for InnerVisitor<'a> {
            type Value = LongArray<'a>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("long array")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let token = map.next_key::<&str>()?.ok_or_else(|| {
                    serde::de::Error::custom("expected NBT long array token, but got empty map")
                })?;
                let data = map.next_value::<&[u8]>()?;

                if token == LONG_ARRAY_TOKEN {
                    Ok(LongArray::from_bytes(data))
                } else {
                    Err(serde::de::Error::custom("expected NBT long array token"))
                }
            }
        }
        deserializer.deserialize_newtype_struct(LONG_ARRAY_TOKEN, InnerVisitor(PhantomData))
    }
}

pub struct LongIter<'a>(LongArray<'a>);

impl<'a> Iterator for LongIter<'a> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i64::<BigEndian>().ok()
    }
}

impl<'a> Serialize for LongArray<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // We can't know anything about NBT here, since we might be serializing
        // to a different format. But we can create a hidden inner type to
        // signal the serializer.
        #[derive(Serialize)]
        #[allow(non_camel_case_types)]
        enum Inner<'a> {
            __fastnbt_long_array(&'a Bytes),
        }

        // Alignment of i64 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.align_to::<u8>() };

        let array = Inner::__fastnbt_long_array(Bytes::new(data));

        array.serialize(serializer)
    }
}

struct CowStr<'a>(Cow<'a, str>);

impl<'de> serde::Deserialize<'de> for CowStr<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CowStrVisitor)
    }
}

struct CowStrVisitor;

impl<'de> serde::de::Visitor<'de> for CowStrVisitor {
    type Value = CowStr<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    // Borrowed directly from the input string, which has lifetime 'de
    // The input must outlive the resulting Cow.
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowStr(Cow::Borrowed(value)))
    }

    // A string that currently only lives in a temporary buffer -- we need a copy
    // (Example: serde is reading from a BufRead)
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowStr(Cow::Owned(value.to_owned())))
    }

    // An optimisation of visit_str for situations where the deserializer has
    // already taken ownership. For example, the string contains escaped characters.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CowStr(Cow::Owned(value)))
    }
}

pub fn deserialize_cow_str<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wrapper = CowStr::deserialize(deserializer)?;
    Ok(wrapper.0)
}
