// want to make sure that we can
// * do hashmaps
// * do arbitrary decoding with a Value type.
// * do normal structs
// * do enums, internally tagged would be useful for entities in the chunk format.
//
// In order to do this I think I need to
//
// * leverage deserialize_any heavily.
// * keep track of the exact state of deserialization, ie the level and
//   structure of nesting in lists and compounds, as well as the state of the
//   current tag/name/value.

// Some notes about this implementation
// * When deserializing into singular unsigned types, it is an error to try and
//   parse a negative value. This will not apply to &[u8].
// * The unit type acts as a way to verify a field exists, but ignores its value.
// * An NBT List of Byte can be deserialized to &[u8] or other sequence.
// * An NBT ByteArray, IntArray or LongArray can all be deserialized into &[u8].

use std::convert::{TryFrom, TryInto};

use crate::error::{Error, Result};
use crate::Tag;
use byteorder::{BigEndian, ReadBytesExt};
use serde::{de, forward_to_deserialize_any};

enum Stage {
    Tag,
    Name,
    Value,
}

enum Layer {
    List {
        remaining_elements: i32, // would make more sense as usize, but format is i32.
        element_tag: Tag,
    },
    Compound {
        current_tag: Option<Tag>,
        stage: Stage,
    },
}

// Without this we would not be able to implement helper functions for the
// input. If we wrote the helper functions as part of the Deserializer impl, it
// would force borrowing the entire deserializer mutably. This helper allows us
// to borrow just the input, making us free to also borrow/mutate the layers.
struct InputHelper<'de>(&'de [u8]);

pub struct Deserializer<'de> {
    input: InputHelper<'de>,
    layers: Vec<Layer>,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Self {
            input: InputHelper(input),
            layers: vec![],
        }
    }
}

fn consume_value<'de, V>(de: &mut Deserializer<'de>, visitor: V, tag: Tag) -> Result<V::Value>
where
    V: de::Visitor<'de>,
{
    match tag {
        Tag::Byte => visitor.visit_i8(de.input.0.read_i8()?),
        Tag::Short => visitor.visit_i16(de.input.0.read_i16::<BigEndian>()?),
        Tag::Int => visitor.visit_i32(de.input.0.read_i32::<BigEndian>()?),
        Tag::Long => visitor.visit_i64(de.input.0.read_i64::<BigEndian>()?),
        Tag::String => visitor.visit_borrowed_str(de.input.consume_size_prefixed_string()?),
        Tag::Float => visitor.visit_f32(de.input.consume_float()?),
        Tag::Double => visitor.visit_f64(de.input.consume_double()?),
        Tag::Compound => {
            de.layers.push(Layer::Compound {
                current_tag: None,
                stage: Stage::Tag,
            });

            visitor.visit_map(CompoundAccess::new(de))
        }
        Tag::List => {
            let element_tag = de.input.consume_tag()?;
            let size = de.input.consume_list_size()?;

            de.layers.push(Layer::List {
                remaining_elements: size,
                element_tag,
            });

            visitor.visit_seq(ListAccess::new(de, size))
        }
        Tag::ByteArray | Tag::IntArray | Tag::LongArray => {
            let size = de.input.consume_list_size()?;
            let non_array_tag = match tag {
                Tag::ByteArray => Tag::Byte,
                Tag::IntArray => Tag::Int,
                Tag::LongArray => Tag::Long,
                _ => panic!(),
            };

            de.layers.push(Layer::List {
                remaining_elements: size,
                element_tag: non_array_tag,
            });

            // Going to pretend we're in a list to reuse the ListAccess.
            visitor.visit_seq(ListAccess::new(de, size))
        }
        _ => todo!("any value"),
    }
}

impl<'de> InputHelper<'de> {
    fn consume_tag(&mut self) -> Result<Tag> {
        let tag_byte = self.0.read_u8()?;
        Tag::try_from(tag_byte).or_else(|_| Err(Error::InvalidTag(tag_byte)))
    }

    fn consume_name(&mut self) -> Result<&'de str> {
        self.consume_size_prefixed_string()
    }

    fn consume_size_prefixed_string(&mut self) -> Result<&'de str> {
        let len = self.0.read_u16::<BigEndian>()? as usize;
        let s = std::str::from_utf8(&self.0[..len]).map_err(|_| Error::InvalidName);
        self.0 = &self.0[len..];
        s
    }

    fn consume_bytes_unchecked(&mut self, size: i32) -> Result<&'de [u8]> {
        let size: usize = size.try_into()?;
        let bs = &self.0[..size];
        self.0 = &self.0[size..];
        Ok(bs)
    }

    fn consume_list_size(&mut self) -> Result<i32> {
        Ok(self.0.read_i32::<BigEndian>()?)
    }

    fn consume_float(&mut self) -> Result<f32> {
        Ok(self.0.read_f32::<BigEndian>()?)
    }

    fn consume_double(&mut self) -> Result<f64> {
        Ok(self.0.read_f64::<BigEndian>()?)
    }

    fn ignore_value(&mut self, tag: Tag) -> Result<()> {
        match tag {
            Tag::Byte => {
                self.0.read_i8()?;
            }
            Tag::Short => {
                self.0.read_i16::<BigEndian>()?;
            }
            Tag::Int => {
                self.0.read_i32::<BigEndian>()?;
            }
            Tag::Long => {
                self.0.read_i64::<BigEndian>()?;
            }
            Tag::Float => {
                self.consume_float()?;
            }
            Tag::Double => {
                self.consume_double()?;
            }
            Tag::String => {
                self.consume_size_prefixed_string()?;
            }
            Tag::ByteArray => {
                let size = self.consume_list_size()?;
                self.consume_bytes_unchecked(size)?;
            }
            Tag::IntArray => {
                let size = self.consume_list_size()?;
                self.consume_bytes_unchecked(size * 4)?;
            }
            Tag::LongArray => {
                let size = self.consume_list_size()?;
                self.consume_bytes_unchecked(size * 8)?;
            }
            Tag::Compound => {
                // Need to loop and ignore each value until we reach an end tag.

                // we need to enter the compound, then ignore it's value.
                loop {
                    let tag = self.consume_tag()?;
                    if tag == Tag::End {
                        break;
                    }

                    self.consume_name()?;
                    self.ignore_value(tag)?;
                }
            }
            Tag::List => {
                let element_tag = self.consume_tag()?;
                let size = self.consume_list_size()?;
                for _ in 0..size {
                    self.ignore_value(element_tag)?;
                }
            }
            _ => return Err(Error::Message(format!("ignore value: {:?}", tag))),
        }

        Ok(())
    }
}

/// Deserialize into a `T` from some NBT data. See [`Deserializer`] for more information.
///
/// [`Deserializer`]: ./struct.Deserializer.html
pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    let mut des = Deserializer::from_bytes(&input);
    let t = T::deserialize(&mut des)?;
    // TODO: trailing chars?
    Ok(t)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any!(struct map identifier i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 str string seq tuple);

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let tag = match self.layers.last_mut().as_mut() {
            None => {
                // No existing layers. This means we should be at the start of
                // parsing, and we should be parsing a Compound. We need to get
                // the tag and the following name and discard it.
                let tag = self.input.consume_tag()?;
                if tag != Tag::Compound {
                    return Err(Error::NoRootCompound);
                }

                self.input.consume_name()?;

                self.layers.push(Layer::Compound {
                    current_tag: None,
                    stage: Stage::Tag,
                });

                return visitor.visit_map(CompoundAccess::new(self));
            }
            Some(layer) => {
                // Pick what we do based on the stage of parsing.
                match layer {
                    Layer::Compound {
                        ref mut current_tag,
                        ref mut stage,
                    } => match stage {
                        Stage::Tag => {
                            *current_tag = Some(self.input.consume_tag()?);
                            *stage = Stage::Value;
                            return visitor.visit_borrowed_str(self.input.consume_name()?);
                        }
                        Stage::Name => {
                            *stage = Stage::Value;
                            return visitor.visit_borrowed_str(self.input.consume_name()?);
                        }
                        Stage::Value => {
                            *stage = Stage::Tag;

                            // TODO: Remove unwrap
                            current_tag.unwrap()
                        }
                    },
                    Layer::List {
                        remaining_elements: _,
                        element_tag,
                    } => *element_tag,
                    _ => todo!("layer type"),
                }
            }
        };

        consume_value(self, visitor, tag)
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let tag = match self.layers.last() {
            Some(Layer::Compound { current_tag, .. }) => current_tag.as_ref().ok_or_else(|| {
                Error::Message("deserialize bool: did not know value's tag".to_string())
            }),
            Some(Layer::List { element_tag, .. }) => Ok(element_tag),
            None => Err(Error::Message(
                "deserialize bool: not in compound or list".to_string(),
            )),
        }?;

        match tag {
            Tag::Byte => visitor.visit_bool(self.input.0.read_i8()? != 0),
            Tag::Short => visitor.visit_bool(self.input.0.read_i16::<BigEndian>()? != 0),
            Tag::Int => visitor.visit_bool(self.input.0.read_i32::<BigEndian>()? != 0),
            Tag::Long => visitor.visit_bool(self.input.0.read_i64::<BigEndian>()? != 0),
            _ => Err(Error::Message(
                "deserialize bool: expected integral value".to_string(),
            )),
        }
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("char")
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let layer = self.layers.last().ok_or(Error::Message(format!(
            "expected bytes, but not in a compound or list",
        )))?;

        match layer {
            Layer::List {
                remaining_elements,
                element_tag,
                ..
            } => Err(Error::Message(format!(
                "expected bytes, got [{:?}; {}]",
                element_tag, remaining_elements
            ))),
            Layer::Compound {
                current_tag: None, ..
            } => Err(Error::Message(
                "expected bytes, but do not know what to deserialize".to_owned(),
            )),
            Layer::Compound {
                current_tag: Some(Tag::List),
                ..
            } => {
                let el = self.input.consume_tag()?;
                let size = self.input.consume_list_size()?;

                match el {
                    Tag::Byte => {
                        let bs = self.input.consume_bytes_unchecked(size)?;
                        visitor.visit_borrowed_bytes(bs)
                    }
                    Tag::Short => {
                        let bs = self.input.consume_bytes_unchecked(size * 2)?;
                        visitor.visit_borrowed_bytes(bs)
                    }
                    Tag::Int => {
                        let bs = self.input.consume_bytes_unchecked(size * 4)?;
                        visitor.visit_borrowed_bytes(bs)
                    }
                    Tag::Long => {
                        let bs = self.input.consume_bytes_unchecked(size * 8)?;
                        visitor.visit_borrowed_bytes(bs)
                    }
                    _ => Err(Error::Message(format!(
                        "expected bytes, got [{:?}; {}]",
                        el, size
                    ))),
                }
            }
            Layer::Compound {
                current_tag: Some(tag),
                ..
            } => match tag {
                Tag::ByteArray => {
                    let size = self.input.consume_list_size()?;
                    let bs = self.input.consume_bytes_unchecked(size)?;
                    visitor.visit_borrowed_bytes(bs)
                }
                Tag::IntArray => {
                    let size = self.input.consume_list_size()?;
                    let bs = self.input.consume_bytes_unchecked(size * 4i32)?;
                    visitor.visit_borrowed_bytes(bs)
                }
                // This allows us to borrow blockstates rather than copy them.
                Tag::LongArray => {
                    let size = self.input.consume_list_size()?;
                    let bs = self.input.consume_bytes_unchecked(size * 8i32)?;
                    visitor.visit_borrowed_bytes(bs)
                }
                _ => Err(Error::Message(format!("expected bytes, found {:?}", tag))),
            },
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // How do we even get here? Vec and slices don't call this.
        unimplemented!("byte_buf")
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let tag = match self.layers.last() {
            Some(Layer::Compound { current_tag, .. }) => current_tag.as_ref().ok_or_else(|| {
                Error::Message("deserialize unit: did not know value's tag".to_string())
            }),
            Some(Layer::List { element_tag, .. }) => Ok(element_tag),
            None => Err(Error::Message(
                "deserialize_unit: not in compound or list".to_string(),
            )),
        }?;

        self.input.ignore_value(*tag)?;
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("unit_struct")
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("tuple_struct")
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(UnitVariantAccess { de: self })
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // The NBT contains a field that we don't want.
        // The last layer should tell us what value we're expecting.
        // We have already read the tag and name. This is the payload.

        let layer = self
            .layers
            .last()
            .ok_or(Error::Message(format!(
                "expected unwanted payload, but not in a compound or list",
            )))?
            .clone();

        match layer {
            Layer::Compound {
                current_tag: Some(tag),
                stage: Stage::Value,
            } => {
                self.input.ignore_value(*tag)?;
            }
            Layer::Compound {
                current_tag: _,
                stage: _,
            } => todo!("compound(none)"), // ???
            Layer::List {
                remaining_elements: _,
                element_tag: _,
            } => {
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

impl<'a, 'de> de::MapAccess<'de> for CompoundAccess<'a, 'de> {
    type Error = Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        // Need to read the tag of the key.
        let tag = self.de.input.consume_tag()?;

        if tag == Tag::End {
            self.de.layers.pop();
            return Ok(None);
        }

        // Set the current layers next expected type.
        // TODO: Can probably do this by mutating top layer rather than pop/push.
        self.de.layers.pop().unwrap();
        self.de.layers.push(Layer::Compound {
            current_tag: Some(tag),
            stage: Stage::Name,
        });

        // Should just be ready to read the name.
        seed.deserialize(&mut *self.de).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct ListAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    hint: i32,
}

impl<'a, 'de> ListAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, hint: i32) -> Self {
        Self { de, hint }
    }
}

impl<'a, 'de> de::SeqAccess<'de> for ListAccess<'a, 'de> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        self.hint.try_into().ok()
    }

    #[inline]
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
            Layer::List {
                remaining_elements,
                element_tag: _,
            } => {
                if *remaining_elements > 0 {
                    *remaining_elements = *remaining_elements - 1;
                    let val = seed.deserialize(&mut *self.de)?;
                    Ok(Some(val))
                } else {
                    self.de.layers.pop();
                    Ok(None)
                }
            }
            Layer::Compound {
                current_tag,
                stage: _,
            } => Err(Error::Message(format!(
                "expected to be in list, but was in compound {:?}",
                current_tag
            ))),
        }
    }
}

struct UnitVariantAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::EnumAccess<'de> for UnitVariantAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'a, 'de> de::VariantAccess<'de> for UnitVariantAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        todo!("unit variant: newtype variant")
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("unit variant: variant")
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        todo!("unit variant: struct variant")
    }
}
