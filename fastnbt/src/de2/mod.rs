use std::{io::Read, marker::PhantomData};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{de, forward_to_deserialize_any, Deserialize};

use crate::{
    error::{Error, Result},
    Tag,
};

use self::{
    de_arrays::ArrayWrapperAccess,
    input::{Input, Reference},
};

mod de_arrays;
mod input;

#[cfg(test)]
mod test;

pub struct Deserializer<In> {
    input: In,
    scratch: Vec<u8>,
    seen_root: bool,
}

impl<'de, In> Deserializer<In>
where
    In: Input<'de>,
{
    pub fn new(input: In) -> Self {
        Self {
            input,
            scratch: Vec::new(),
            seen_root: false,
        }
    }
}

pub fn from_bytes<'de, T>(bytes: &'de [u8]) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    de::Deserialize::deserialize(&mut deserializer)
}

impl<'a> Deserializer<input::Slice<'a>> {
    /// Creates a JSON deserializer from a `&[u8]`.
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Deserializer::new(input::Slice { data: bytes })
    }
}

pub fn from_reader<'de, R, T>(bytes: R) -> Result<T>
where
    T: de::Deserialize<'de>,
    R: Read,
{
    let mut deserializer = Deserializer::from_reader(bytes);
    de::Deserialize::deserialize(&mut deserializer)
}

impl<R: Read> Deserializer<input::Reader<R>> {
    /// Creates a JSON deserializer from a `&[u8]`.
    pub fn from_reader(reader: R) -> Self {
        Deserializer::new(input::Reader { reader })
    }
}

impl<'de, 'a, In> de::Deserializer<'de> for &'a mut Deserializer<In>
where
    In: Input<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if !self.seen_root {
            let peek = self.input.consume_tag()?;

            match peek {
                Tag::Compound => self.input.ignore_str()?,
                _ => return Err(Error::no_root_compound()),
            }

            self.seen_root = true;
        }

        visitor.visit_map(MapAccess::new(self))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }
}

struct MapAccess<'a, In: 'a> {
    de: &'a mut Deserializer<In>,
    tag: Tag, // current tag
}

impl<'a, In: 'a> MapAccess<'a, In> {
    pub fn new(de: &'a mut Deserializer<In>) -> Self {
        Self { de, tag: Tag::End }
    }
}

impl<'de, 'a, In: Input<'de> + 'a> de::MapAccess<'de> for MapAccess<'a, In> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.tag = self.de.input.consume_tag()?;
        if self.tag == Tag::End {
            return Ok(None);
        }

        seed.deserialize(MapKey { de: &mut *self.de }).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(AnonymousValue {
            tag: self.tag,
            de: &mut *self.de,
            last_hint: Hint::None,
        })
    }
}

struct MapKey<'a, In> {
    de: &'a mut Deserializer<In>,
}

impl<'de, 'a, R> de::Deserializer<'de> for MapKey<'a, R>
where
    R: Input<'de>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.de.input.consume_str(&mut self.de.scratch)? {
            Reference::Borrowed(s) => visitor.visit_borrowed_str(s),
            Reference::Copied(s) => visitor.visit_str(s),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any bytes enum newtype_struct byte_buf option
    }
}

enum Hint {
    None,
    Seq,
}

/// Deserializer for an anonymous value, ie one with no tag or name before it.
/// This occurs in lists, but is also used to deserialize the value part of compounds.
///
/// This is the 'core' of the deserializer if there can be said to be one.
struct AnonymousValue<'a, In> {
    tag: Tag,
    last_hint: Hint,
    de: &'a mut Deserializer<In>,
}

impl<'de, 'a, In> de::Deserializer<'de> for AnonymousValue<'a, In>
where
    In: Input<'de>,
{
    type Error = Error;

    forward_to_deserialize_any!(bool u8 u16 u32 u64 i8 i16 i32 i64 f32 
        f64 str string struct tuple map identifier char);

    fn deserialize_any<V>(mut self, v: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_hint = self.last_hint;
        self.last_hint = Hint::None;


        match self.tag {
            Tag::End => todo!(),
            Tag::Byte => v.visit_i8(self.de.input.consume_byte()? as i8),
            Tag::Short => v.visit_i16(self.de.input.consume_i16()?),
            Tag::Int => v.visit_i32(self.de.input.consume_i32()?),
            Tag::Long => v.visit_i64(self.de.input.consume_i64()?),
            Tag::Float => v.visit_f32(self.de.input.consume_f32()?),
            Tag::Double => v.visit_f64(self.de.input.consume_f64()?),
            Tag::String => match self.de.input.consume_str(&mut self.de.scratch)? {
                Reference::Borrowed(s) => v.visit_borrowed_str(s),
                Reference::Copied(s) => v.visit_str(s),
            },
            Tag::List => {
                let tag = self.de.input.consume_tag()?;
                let remaining = self.de.input.consume_i32()? as usize;
                v.visit_seq(ListAccess {
                    de: self.de,
                    tag,
                    remaining,
                })
            }
            Tag::Compound => v.visit_map(MapAccess::new(self.de)),
            Tag::ByteArray => {
                if let Hint::Seq = last_hint {
                    return Err(Error::bespoke(
                        "expected NBT Array, found seq: use ByteArray, IntArray or LongArray types"
                            .into(),
                    ));
                }
                self.deserialize_newtype_struct(crate::BYTE_ARRAY_TOKEN, v)
            },
            Tag::IntArray => {
                if let Hint::Seq = last_hint {
                    return Err(Error::bespoke(
                        "expected NBT Array, found seq: use ByteArray, IntArray or LongArray types"
                            .into(),
                    ));
                }
                self.deserialize_newtype_struct(crate::INT_ARRAY_TOKEN, v)
            },
            Tag::LongArray =>{
                if let Hint::Seq = last_hint {
                    return Err(Error::bespoke(
                        "expected NBT Array, found seq: use ByteArray, IntArray or LongArray types"
                            .into(),
                    ));
                }
                self.deserialize_newtype_struct(crate::LONG_ARRAY_TOKEN, v)
            },
        }
    }


    fn deserialize_bytes<V>( self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {

        let consume_visit = |de: &mut Deserializer<In>, len, el_size| match de
            .input
            .consume_bytes(len * el_size, &mut de.scratch)?
        {
            Reference::Borrowed(bs) => visitor.visit_borrowed_bytes(bs),
            Reference::Copied(bs) => visitor.visit_bytes(bs),
        };

        match self.tag {
            Tag::End => todo!(),
            Tag::Byte => todo!(),
            Tag::Short => todo!(),
            Tag::Int => todo!(),
            Tag::Long => todo!(),
            Tag::Float => todo!(),
            Tag::Double => todo!(),
            Tag::String => {
                let len = self.de.input.consume_i16()? as usize;
                consume_visit(self.de, len, 1)
            }
            Tag::List => {
                let tag = self.de.input.consume_tag()?;
                let remaining = self.de.input.consume_i32()? as usize;

                match tag {
                    Tag::End => todo!(),
                    Tag::Byte => consume_visit(self.de, remaining, std::mem::size_of::<i8>()),
                    Tag::Short => consume_visit(self.de, remaining, std::mem::size_of::<i16>()),
                    Tag::Int => consume_visit(self.de, remaining, std::mem::size_of::<i32>()),
                    Tag::Long => consume_visit(self.de, remaining, std::mem::size_of::<i64>()),
                    Tag::Float => todo!(),
                    Tag::Double => todo!(),
                    Tag::ByteArray => todo!(),
                    Tag::String => todo!(),
                    Tag::List => todo!(),
                    Tag::Compound => todo!(),
                    Tag::IntArray => todo!(),
                    Tag::LongArray => todo!(),
                }
            }
            Tag::Compound => todo!(),
            Tag::ByteArray => {
                let remaining = self.de.input.consume_i32()? as usize;
                consume_visit(self.de, remaining, std::mem::size_of::<i8>())
            }
            Tag::IntArray => todo!(),
            Tag::LongArray => {
                let remaining = self.de.input.consume_i32()? as usize;
                consume_visit(self.de, remaining, std::mem::size_of::<i64>())
            }
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // fastnbt quirk: if your type contains a unit, we allow any valid NBT
        // value to 'fill' that hole in your type. This means a unit type can be
        // used to ensure the presense of a value in the NBT without actually
        // caring or deserializing its contents.
        self.de.input.ignore_value(self.tag)?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let target_tag = match name {
            crate::BYTE_ARRAY_TOKEN => Tag::ByteArray,
            crate::INT_ARRAY_TOKEN => Tag::IntArray,
            crate::LONG_ARRAY_TOKEN => Tag::LongArray,
            _ => return visitor.visit_newtype_struct(self),
        };

        let data_tag = self.tag;

        if target_tag == data_tag {
            let len = self.de.input.consume_i32()? as usize;
            match target_tag {
                Tag::ByteArray => visitor.visit_map(ArrayWrapperAccess::bytes(self.de, len)?),
                Tag::IntArray => visitor.visit_map(ArrayWrapperAccess::ints(self.de, len)?),
                Tag::LongArray => visitor.visit_map(ArrayWrapperAccess::longs(self.de, len)?),
                // We know that `target_tag` is one of the above because we just
                // created it.
                _ => unreachable!(),
            }
        } else {
            Err(Error::bespoke(format!(
                "NBT contained {data_tag}, expected {target_tag}"
            )))
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(UnitVariantAccess {
            de: AnonymousValue {
                tag: self.tag,
                de: self.de,
                last_hint: Hint::None
            },
        })
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.input.ignore_value(self.tag)?;
        visitor.visit_unit()
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de> {

        self.last_hint = Hint::Seq;
        self.deserialize_any(visitor)
    }
}

struct ListAccess<'a, In: 'a> {
    de: &'a mut Deserializer<In>,
    tag: Tag, // current tag
    remaining: usize,
}

impl<'de, 'a, In: Input<'de> + 'a> de::SeqAccess<'de> for ListAccess<'a, In> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.remaining > 0 {
            self.remaining -= 1;
            seed.deserialize(AnonymousValue {
                de: &mut *self.de,
                last_hint: Hint::None,
                tag: self.tag,
            })
            .map(Some)
        } else {
            Ok(None)
        }
    }
}

fn try_size(size: i32, multiplier: usize) -> Result<usize> {
    let size: usize = size
        .try_into()
        .map_err(|_| Error::bespoke("size was negative".to_string()))?;

    size.checked_mul(multiplier)
        .ok_or_else(|| Error::bespoke("size too large".to_string()))
}

struct UnitVariantAccess<'a, In: 'a> {
    de: AnonymousValue<'a, In>,
}

impl<'de, 'a, In: Input<'de> + 'a> de::EnumAccess<'de> for UnitVariantAccess<'a, In> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(AnonymousValue {
            de: &mut *self.de.de,
                last_hint: Hint::None,
                tag: self.de.tag,
        })?;
        Ok((variant, self))
    }
}

impl<'de, 'a, In: Input<'de> + 'a> de::VariantAccess<'de> for UnitVariantAccess<'a, In> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::TupleVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            de::Unexpected::StructVariant,
            &"struct variant",
        ))
    }
}
