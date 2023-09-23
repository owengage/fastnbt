//! This module contains a serde deserializer.
//! It can do most of the things you would expect of
//! a typical serde deserializer, such as deserializing into:
//! - Rust structs.
//! - containers like `HashMap` and `Vec`.
//! - an arbitrary `Value`.
//!
//! This deserializer supports [`from_str`](crate::from_str) for
//! zero-copy deserialization for types like [`&str`] if possible.
//! If there are escaped characters in the string, it will have
//! to own the resulting string.
//!
//! ## Uuid
//! Because [`Deserializer`] expects a human-readable format,
//! `UUID`s are expected to be strings.

use std::{borrow::Cow, marker::PhantomData};

use byteorder::{WriteBytesExt, BE};
use serde::{de::{self, SeqAccess, MapAccess, IntoDeserializer, value::{BorrowedStrDeserializer, SeqAccessDeserializer, BytesDeserializer}, Visitor, DeserializeSeed}, forward_to_deserialize_any};

use crate::{error::Error, parser::{parse_i8, parse_i16, parse_i32, parse_i64, parse_bool, parse_f32, parse_f64, parse_str}, LONG_ARRAY_TOKEN, INT_ARRAY_TOKEN, BYTE_ARRAY_TOKEN};

pub struct Deserializer<'de> {
    pub(crate) input: &'de str,
    pub(crate) pos: usize,
}

impl<'a, 'de: 'a> Deserializer<'de> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &'de str) -> Self {
        Self { input, pos: 0, }
    }

    pub(crate) fn advance(&mut self, new_input: &'de str) {
        self.pos += self.input.len() - new_input.len();
        self.input = new_input;
    }

    pub(crate) fn starts_delimiter(&mut self, start: &str) -> bool {
        if self.input.starts_with(start) {
            let input = &self.input[start.len()..];
            self.advance(input);
            true
        } else {
            false
        }
    }

    pub(crate) fn end_delimiter(&'a mut self, end: &'de str) -> Result<&'de str, Error> {
        if !self.input.starts_with(end) {
            Err(Error::expected_collection_end())
        } else {
            Ok(&self.input[end.len()..])
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
    {
        if self.input.is_empty() {
            return Err(Error::unexpected_eof());
        }

        // It's important to keep this in the correct order -> precedence rules
        let (input, value) = if let Ok((input, v)) = parse_f32(self.input) {
            visitor.visit_f32(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_f64(self.input) {
            visitor.visit_f64(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_i8(self.input) {
            visitor.visit_i8(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_i16(self.input) {
            visitor.visit_i16(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_i64(self.input) {
            visitor.visit_i64(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_i32(self.input) {
            visitor.visit_i32(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_bool(self.input) {
            visitor.visit_bool(v).map(|v| (input, v))
        } else if let Ok((input, v)) = parse_str(self.input) {
            match v {
                Cow::Borrowed(v) => visitor.visit_borrowed_str(v),
                Cow::Owned(v) => visitor.visit_str(&v),
            }.map(|v| (input, v))
        } else if self.starts_delimiter("[B;") {
            match visitor.visit_map(ArrayWrapperAccess::bytes(self)) {
                Ok(v) => self.end_delimiter("]").map(|input| (input, v)),
                Err(e) => Err(e),
            }
        } else if self.starts_delimiter("[I;") {
            match visitor.visit_map(ArrayWrapperAccess::ints(self)) {
                Ok(v) => self.end_delimiter("]").map(|input| (input, v)),
                Err(e) => Err(e),
            }
        } else if self.starts_delimiter("[L;") {
            match visitor.visit_map(ArrayWrapperAccess::longs(self)) {
                Ok(v) => self.end_delimiter("]").map(|input| (input, v)),
                Err(e) => Err(e),
            }
        } else if self.starts_delimiter("[") {
            match visitor.visit_seq(CommaSep::new(self)) {
                Ok(v) => self.end_delimiter("]").map(|input| (input, v)),
                Err(e) => Err(e),
            }
        } else if self.starts_delimiter("{") {
            match visitor.visit_map(CommaSep::new(self)) {
                Ok(v) => self.end_delimiter("}").map(|input| (input, v)),
                Err(e) => Err(e),
            }
        } else {
            Err(Error::invalid_input(self.pos))
        }?;

        self.advance(input);
        Ok(value)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
    {
        let (input, value) = if let Ok((input, v)) = parse_str(self.input) {
            visitor.visit_enum(v.as_ref().into_deserializer()).map(|v| (input, v))
        } else {
            Err(Error::invalid_input(self.pos))
        }?;
        self.advance(input);
        Ok(value)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>
    {
        self.deserialize_any(visitor)
    }

}

struct CommaSep<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    first: bool,
}

impl<'a, 'de> CommaSep<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        CommaSep { de, first: true, }
    }
}

impl<'a, 'de> SeqAccess<'de> for CommaSep<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>
    {
        if self.de.input.starts_with(']') {
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.input.chars().next().ok_or(Error::unexpected_eof())? != ',' {
            return Err(Error::expected_comma());
        } else if !self.first {
            self.de.advance(&self.de.input[','.len_utf8()..])
        }
        self.first = false;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'a, 'de> MapAccess<'de> for CommaSep<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>
    {
        if self.de.input.starts_with('}') {
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.input.chars().next().ok_or(Error::unexpected_eof())? != ',' {
            return Err(Error::expected_comma());
        } else if !self.first {
            self.de.advance(&self.de.input[','.len_utf8()..]);
        }
        self.first = false;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>
    {
        if self.de.input.chars().next().ok_or(Error::unexpected_eof())? != ':' {
            return Err(Error::expected_colon());
        } else {
            self.de.advance(&self.de.input[':'.len_utf8()..]);
        }
        seed.deserialize(&mut *self.de)
    }
}

struct ArrayWrapperAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    token: &'static str,
    read: bool,
}

impl<'a, 'de> ArrayWrapperAccess<'a, 'de> {
    pub fn bytes(de: &'a mut Deserializer<'de>) -> Self {
        ArrayWrapperAccess { de, token: BYTE_ARRAY_TOKEN, read: false }
    }

    pub fn ints(de: &'a mut Deserializer<'de>) -> Self {
        ArrayWrapperAccess { de, token: INT_ARRAY_TOKEN, read: false }
    }

    pub fn longs(de: &'a mut Deserializer<'de>) -> Self {
        ArrayWrapperAccess { de, token: LONG_ARRAY_TOKEN, read: false }
    }
}

impl<'a, 'de> MapAccess<'de> for ArrayWrapperAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>
    {
        if !self.read {
            self.read = true;
            seed.deserialize(BorrowedStrDeserializer::new(self.token)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>
    {
        match self.token {
            BYTE_ARRAY_TOKEN => {
                let data = <Vec<i8> as de::Deserialize>::deserialize(SeqAccessDeserializer::new(CommaSep::new(self.de)))?;
                let data = unsafe { &*(data.as_slice() as *const [i8] as *const [u8]) };
                seed.deserialize(BytesDeserializer::new(data))
            }
            INT_ARRAY_TOKEN => {
                let data = NumStride::<i32>(PhantomData).deserialize(SeqAccessDeserializer::new(CommaSep::new(self.de)))?;
                seed.deserialize(BytesDeserializer::new(&data.bytes))
            }
            LONG_ARRAY_TOKEN => {
                let data = NumStride::<i64>(PhantomData).deserialize(SeqAccessDeserializer::new(CommaSep::new(self.de)))?;
                seed.deserialize(BytesDeserializer::new(&data.bytes))
            }
            _ => unreachable!("Cannot have a different NBT array type"),
        }
    }
}

struct NumStride<T>(PhantomData<T>);
struct NumByteArray {
    bytes: Vec<u8>,
}

impl<'de> de::DeserializeSeed<'de> for NumStride<i32> {
    type Value = NumByteArray;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        struct NumArrVisitor(Vec<u8>);

        impl<'de> Visitor<'de> for NumArrVisitor {
            type Value = NumByteArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of i32")
            }

            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
            {
                while let Some(n) = seq.next_element::<i32>()? {
                    self.0.write_i32::<BE>(n).unwrap();
                }
                Ok(NumByteArray { bytes: self.0 })
            }
        }

        deserializer.deserialize_seq(NumArrVisitor(vec![]))
    }
}

impl<'de> de::DeserializeSeed<'de> for NumStride<i64> {
    type Value = NumByteArray;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        struct NumArrVisitor(Vec<u8>);

        impl<'de> Visitor<'de> for NumArrVisitor {
            type Value = NumByteArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of i64")
            }

            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
            {
                while let Some(n) = seq.next_element::<i64>()? {
                    self.0.write_i64::<BE>(n).unwrap();
                }
                Ok(NumByteArray { bytes: self.0 })
            }
        }

        deserializer.deserialize_seq(NumArrVisitor(vec![]))
    }
}
