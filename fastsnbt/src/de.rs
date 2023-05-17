use std::borrow::Cow;

use serde::{de::{self, SeqAccess, MapAccess}, forward_to_deserialize_any};

use crate::{error::Error, parser::{parse_i8, parse_i16, parse_i32, parse_i64, parse_bool, parse_f32, parse_f64, parse_str}};

pub struct Deserializer<'de> {
    pub(crate) input: &'de str,
    pub(crate) pos: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Self { input, pos: 0, }
    }

    pub(crate) fn advance(&mut self, new_input: &'de str) {
        self.pos += self.input.len() - new_input.len();
        self.input = new_input;
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
        } else if self.input.starts_with('[') {
            let input = &self.input['['.len_utf8()..];
            self.advance(input);
            match visitor.visit_seq(CommaSep::new(self)) {
                Ok(v) => {
                    if self.input.starts_with(']') {
                        let input = &self.input[']'.len_utf8()..];
                        Ok((input, v))
                    } else {
                        Err(Error::expected_array_end())
                    }
                }
                Err(e) => Err(e),
            }
        } else if self.input.starts_with('{') {
            let input = &self.input['{'.len_utf8()..];
            self.advance(input);
            match visitor.visit_map(CommaSep::new(self)) {
                Ok(v) => {
                    if self.input.starts_with('}') {
                        let input = &self.input['}'.len_utf8()..];
                        Ok((input, v))
                    } else {
                        Err(Error::expected_map_end())
                    }
                }
                Err(e) => Err(e)
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

    // TODO: make enums work
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de> {
        todo!()
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
