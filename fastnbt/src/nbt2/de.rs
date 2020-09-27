use super::error::{Error, Result};
use super::Tag;
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::str;

pub struct Deserializer<'de> {
    input: &'de [u8],
    layers: Vec<Layer>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Self {
            input,
            layers: vec![],
        }
    }
}

pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut des = Deserializer::from_bytes(&input);
    let t = T::deserialize(&mut des)?;
    // TODO: trailing chars?
    Ok(t)
}

impl<'de> Deserializer<'de> {
    fn consume_tag(&mut self) -> Result<Tag> {
        let tag_byte = self.input.read_u8()?;
        u8_to_tag(tag_byte)
    }

    fn consume_name(&mut self) -> Result<&'de str> {
        self.consume_size_prefixed_string()
    }

    fn consume_size_prefixed_string(&mut self) -> Result<&'de str> {
        let len = self.input.read_u16::<BigEndian>()? as usize;
        let s = str::from_utf8(&self.input[..len]).map_err(|_| Error::InvalidName);
        self.input = &self.input[len..];
        s
    }

    fn consume_integral(&mut self) -> Result<i64> {
        Ok(match self.layers.last() {
            Some(Layer::Compound(Some(Tag::Byte))) => self.input.read_i8()? as i64,
            Some(Layer::Compound(Some(Tag::Short))) => self.input.read_i16::<BigEndian>()? as i64,
            Some(Layer::Compound(Some(Tag::Int))) => self.input.read_i32::<BigEndian>()? as i64,
            Some(Layer::Compound(Some(Tag::Long))) => self.input.read_i64::<BigEndian>()? as i64,
            Some(Layer::List(Tag::Byte, _)) => self.input.read_i8()? as i64,
            Some(Layer::List(Tag::Short, _)) => self.input.read_i16::<BigEndian>()? as i64,
            Some(Layer::List(Tag::Int, _)) => self.input.read_i32::<BigEndian>()? as i64,
            Some(Layer::List(Tag::Long, _)) => self.input.read_i64::<BigEndian>()? as i64,
            _ => return Err(Error::Message("expected integral".to_owned())),
        })
    }

    fn consume_list_size(&mut self) -> Result<i32> {
        Ok(self.input.read_i32::<BigEndian>()?)
    }

    fn consume_float(&mut self) -> Result<f32> {
        Ok(match self.layers.last() {
            Some(Layer::Compound(Some(Tag::Float))) => self.input.read_f32::<BigEndian>()?,
            Some(Layer::List(Tag::Double, _)) => self.input.read_f32::<BigEndian>()?,
            _ => return Err(Error::Message("expected float".to_owned())),
        })
    }

    fn consume_double(&mut self) -> Result<f64> {
        Ok(match self.layers.last() {
            Some(Layer::Compound(Some(Tag::Double))) => self.input.read_f64::<BigEndian>()?,
            Some(Layer::List(Tag::Double, _)) => self.input.read_f64::<BigEndian>()?,
            _ => return Err(Error::Message("expected double float".to_owned())),
        })
    }
}

fn u8_to_tag(tag: u8) -> Result<Tag> {
    Tag::try_from(tag).or_else(|_| Err(Error::InvalidTag(tag)))
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("any")
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Any non-zero number treated as true.
        let num = self.consume_integral()?;
        visitor.visit_bool(!(num == 0))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let name = self.consume_name()?;
        visitor.visit_str(name)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("byte_buf")
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("bytes")
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("char")
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("enum")
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.consume_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.consume_double()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_u8(num.try_into()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_i8(num.try_into()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_i16(num.try_into()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_i32(num.try_into()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_i64(num)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_u16(num.try_into()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_u32(num.try_into()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num = self.consume_integral()?;
        visitor.visit_u64(num.try_into()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let s = self.consume_size_prefixed_string()?;
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("option")
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("unit")
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("unit_struct")
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("newtype_struct")
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // We should be just after the point of reading the name of the list.
        // So we need to read the element type, then the size.
        let element_tag = self.consume_tag()?;
        let size = self.consume_list_size()?;

        // TODO: Fix signedness.
        self.layers.push(Layer::List(element_tag, size as i32));

        visitor.visit_seq(ListAccess::new(self))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple")
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("tuple_struct")
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // For a nested struct we get here AFTER processing the compound tag and it's name.
        // We need to immediately start looking at it's keys.

        // Get the tag, which should definitely be 'compound'.
        let tag = match self.layers.last() {
            Some(Layer::Compound(Some(tag))) => tag.clone(),
            Some(Layer::Compound(None)) => {
                return Err(Error::Message(
                    "expected struct, did not know what to deserialize".to_owned(),
                ))
            }
            Some(Layer::List(tag, _)) => tag.clone(),
            None => {
                // We're at the very start of parsing, we expect the NBT to start with a compound
                // and need to parse the tag and name before calling visit_map.
                let tag = self.consume_tag()?;
                self.consume_name()?;
                tag
            }
        };

        if tag == Tag::Compound {
            self.layers.push(Layer::Compound(None));
        } else {
            return Err(Error::Message(format!("expected compound, got {:?}", tag)));
        }

        visitor.visit_map(CompoundAccess::new(self))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // The NBT contains a field that we don't want.
        // The last layer should tell us what value we're expecting.
        // We have already read the tag and name. This is the payload.

        let layer = self.layers.last().ok_or(Error::Message(format!(
            "expected unwanted payload, but not in a compound or list",
        )))?;

        match layer {
            Layer::Compound(Some(tag)) => match tag {
                Tag::Byte | Tag::Short | Tag::Int | Tag::Long => {
                    self.consume_integral()?;
                }
                Tag::String => {
                    self.consume_size_prefixed_string()?;
                }
                Tag::Float => {
                    self.consume_float()?;
                }
                Tag::Double => {
                    self.consume_double()?;
                }
                _ => todo!("ignored_any missing {:?}", tag),
            },
            Layer::Compound(None) => todo!("compound(none)"), // ???
            Layer::List(_, _) => {
                todo!();
            }
        }

        visitor.visit_unit()
    }
}

struct CompoundAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> CompoundAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'a, 'de> MapAccess<'de> for CompoundAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        // Need to read the tag of the key.
        let tag = self.de.consume_tag()?;

        if tag == Tag::End {
            self.de.layers.pop();
            return Ok(None);
        }

        // Set the current layers next expected type.
        // TODO: Can probably do this by mutating top layer rather than pop/push.
        self.de.layers.pop();
        self.de.layers.push(Layer::Compound(Some(tag)));

        // Should just be ready to read the name.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct ListAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ListAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'a, 'de> SeqAccess<'de> for ListAccess<'a, 'de> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        None
    }

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let layer = self
            .de
            .layers
            .last_mut()
            .ok_or(Error::Message("expected to be in list".to_owned()))?;

        match layer {
            Layer::List(_, size) => {
                if *size > 0 {
                    *size = *size - 1;
                    let val = seed.deserialize(&mut *self.de)?;
                    Ok(Some(val))
                } else {
                    Ok(None)
                }
            }
            Layer::Compound(_) => Err(Error::Message(
                "expected to be in list, but was in compound".to_owned(),
            )),
        }
    }
}

enum Layer {
    List(Tag, i32),        // Tag of elements, number of elements left.
    Compound(Option<Tag>), // Tag is the type of the next expected value.
}
