use core::panic;
use std::convert::TryInto;
use std::io::Read;
use std::num::TryFromIntError;

use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{self, IntoDeserializer};
use serde::forward_to_deserialize_any;

use crate::error::{Error, Result};
use crate::{de::Deserializer, Tag};

enum ArrWrapStage {
    Tag,
    Data,
    Done,
}

pub(crate) struct ArrayWrapperAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    stage: ArrWrapStage,
    tag: Tag,
    size: i32,
}

impl<'a, 'de> ArrayWrapperAccess<'a, 'de> {
    pub(crate) fn new(de: &'a mut Deserializer<'de>, size: i32, tag: Tag) -> Self {
        Self {
            de,
            tag,
            size,
            stage: ArrWrapStage::Tag,
        }
    }
}

impl<'a, 'de> de::MapAccess<'de> for ArrayWrapperAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.stage {
            ArrWrapStage::Tag => seed.deserialize("tag".into_deserializer()).map(Some),
            ArrWrapStage::Data => seed.deserialize("data".into_deserializer()).map(Some),
            ArrWrapStage::Done => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.stage {
            ArrWrapStage::Tag => {
                self.stage = ArrWrapStage::Data;
                let t: u8 = self.tag.into();
                seed.deserialize(t.into_deserializer())
            }
            ArrWrapStage::Data => {
                self.stage = ArrWrapStage::Done;
                seed.deserialize(ArrayDeserializer {
                    de: &mut *self.de,
                    size: self.size,
                    tag: self.tag,
                })
            }
            ArrWrapStage::Done => panic!("extra key"),
        }
    }
}

struct ArrayAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    hint: i32,
    remaining: i32,
    tag: Tag,
}

impl<'a, 'de> ArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, tag: Tag, size: i32) -> Self {
        Self {
            de,
            hint: size,
            remaining: size,
            tag,
        }
    }
}

impl<'a, 'de> de::SeqAccess<'de> for ArrayAccess<'a, 'de> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        self.hint.try_into().ok()
    }

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining > 0 {
            self.remaining = self.remaining - 1;
            let val = seed.deserialize(ArrayElementDeserializer {
                de: self.de,
                tag: self.tag,
            })?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }
}

pub(crate) struct ArrayElementDeserializer<'a, 'de> {
    pub(crate) de: &'a mut Deserializer<'de>,
    pub(crate) tag: Tag,
}

impl<'a, 'de> serde::Deserializer<'de> for ArrayElementDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i16 i128 u16  u128 f32 f64 char str string seq
        bytes byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // This happens when we know we're in an array type value, but the type
        // we're deserializing into does not.
        match self.tag {
            Tag::ByteArray => self.deserialize_i8(visitor),
            Tag::IntArray => self.deserialize_i32(visitor),
            Tag::LongArray => self.deserialize_i64(visitor),
            t => panic!("invalid tag for array deserializer: {:?}", t),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.de.input.0.read_i8()?;
        visitor.visit_i8(val)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.de.input.0.read_u8()?;
        visitor.visit_u8(val)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.de.input.0.read_i32::<BigEndian>()?;
        visitor.visit_i32(val)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.de.input.0.read_u32::<BigEndian>()?;
        visitor.visit_u32(val)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.de.input.0.read_i64::<BigEndian>()?;
        visitor.visit_i64(val)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let val = self.de.input.0.read_u64::<BigEndian>()?;
        visitor.visit_u64(val)
    }
}

pub(crate) struct ArrayDeserializer<'a, 'de> {
    pub(crate) de: &'a mut Deserializer<'de>,
    pub(crate) size: i32,
    pub(crate) tag: Tag,
}

// Job is to start deserializing a Seq which is a *Array type, and to actually
// deserialize the elements.
impl<'a, 'de> serde::Deserializer<'de> for ArrayDeserializer<'a, 'de> {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string seq
     byte_buf option unit unit_struct newtype_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(ArrayAccess::new(self.de, self.tag, self.size)) // TOOD: size
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let len: usize = self
            .size
            .try_into()
            .map_err(|e: TryFromIntError| Error::bespoke(e.to_string()))?;

        let total_bytes = len * element_size(self.tag);

        let res = &self.de.input.0[0..total_bytes];
        self.de.input.0 = &self.de.input.0[total_bytes..];
        visitor.visit_borrowed_bytes(res)
    }
}

fn element_size(tag: Tag) -> usize {
    match tag {
        Tag::ByteArray => std::mem::size_of::<i8>(),
        Tag::IntArray => std::mem::size_of::<i32>(),
        Tag::LongArray => std::mem::size_of::<i64>(),
        _ => panic!("element size of non-array type"),
    }
}
