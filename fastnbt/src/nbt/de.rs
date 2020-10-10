//! A conventional `serde` deserializer module.
//!
//! `from_bytes` can be used to convert NBT data into a Rust `struct`. You can not deserialize into
//! primitive types directly eg `from_bytes::<u32>(...)` due to the NBT data format itself.

use super::error::{Error, Result};
use super::Tag;
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserialize;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::str;

/// Deserializer for getting a `T` from some NBT data. Quite often you will need
/// to rename fields using serde, as most Minecraft NBT data has inconsistent
/// naming. The examples below show this with the `rename_all` attribute. See
/// `serde`s other attributes for more.
///
/// You can take advantage of the lifetime of the input data to save allocations
/// for things like strings. You can also deserialize any Array or List of
/// primitive type as `&'a [u8]` to avoid allocating this data. See example
/// below.
///
/// When deserializing integral types, the values are range checked to prevent
/// overflow from occurring. If an overflow does occur you will get a
/// [`Error::IntegralOutOfRange`] error.
///
/// [`Error::IntegralOutOfRange`]: ../error/enum.Error.html#variant.IntegralOutOfRange
///
/// # Example of deserializing player.dat
///
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug)]
/// #[serde(rename_all = "PascalCase")]
/// struct PlayerDat {
///     data_version: i32,
///     inventory: Vec<InventorySlot>,
///     ender_items: Vec<InventorySlot>,
/// }
///
/// #[derive(Deserialize, Debug)]
/// struct InventorySlot {
///     id: String,
/// }
/// ```
///
/// # Examples of avoiding allocation
///
/// We can easily avoid allocations of `String`s using `&'a str` where `'a` is
/// the lifetime of the input data.
///
/// ```
/// use serde::Deserialize;

/// #[derive(Deserialize, Debug)]
/// struct InventorySlot<'a> {
///     id: &'a str, // we avoid allocating a string here.
/// }
/// ```
///
/// Here we're avoiding allocating memory for the various heightmaps found in chunk data.
/// The [`PackedBits`] type is used as a wrapper for the way Minecraft's Anvil format packs various
/// lists of numbers.
///
/// [`PackedBits`]: ../../anvil/struct.PackedBits.html
///
/// ```
/// use fastnbt::anvil::PackedBits;
/// use serde::Deserialize;
///
///
/// #[derive(Deserialize, Debug)]
/// #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// pub struct Heightmaps<'a> {
///     #[serde(borrow)]
///     pub motion_blocking: Option<PackedBits<'a>>,
///     pub motion_blocking_no_leaves: Option<PackedBits<'a>>,
///     pub ocean_floor: Option<PackedBits<'a>>,
///     pub world_surface: Option<PackedBits<'a>>,
///
///     #[serde(skip)]
///     unpacked_motion_blocking: Option<Vec<u16>>,
/// }
/// ```
/// # Example from region file
///
/// ```no_run
/// use fastnbt::anvil::{Chunk, Region};
/// use fastnbt::de::from_bytes;
///
/// fn main() {
///     let args: Vec<_> = std::env::args().skip(1).collect();
///     let file = std::fs::File::open(args[0].clone()).unwrap();
///
///     let mut region = Region::new(file);
///     let data = region.load_chunk(0, 0).unwrap();
///
///     let chunk: Chunk = from_bytes(data.as_slice()).unwrap();
///
///     println!("{:?}", chunk);
/// }
/// ```
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

/// Deserialize into a `T` from some NBT data. See [`Deserializer`] for more information.
///
/// [`Deserializer`]: ./struct.Deserializer.html
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
        self.consume_integral_unchecked(self.current_values_tag()?)
    }

    fn consume_integral_unchecked(&mut self, tag: Tag) -> Result<i64> {
        Ok(match tag {
            Tag::Byte => self.input.read_i8()? as i64,
            Tag::Short => self.input.read_i16::<BigEndian>()? as i64,
            Tag::Int => self.input.read_i32::<BigEndian>()? as i64,
            Tag::Long => self.input.read_i64::<BigEndian>()? as i64,
            _ => return Err(Error::TypeMismatch(tag, "integral")),
        })
    }

    fn consume_bytes_unchecked(&mut self, size: i32) -> Result<&'de [u8]> {
        let size: usize = size.try_into()?;
        let bs = &self.input[..size];
        self.input = &self.input[size..];
        Ok(bs)
    }

    fn consume_list_size(&mut self) -> Result<i32> {
        Ok(self.input.read_i32::<BigEndian>()?)
    }

    fn consume_float(&mut self) -> Result<f32> {
        Ok(self.input.read_f32::<BigEndian>()?)
    }

    fn consume_double(&mut self) -> Result<f64> {
        Ok(self.input.read_f64::<BigEndian>()?)
    }

    fn ignore_value(&mut self, tag: Tag) -> Result<()> {
        match tag {
            Tag::Byte | Tag::Short | Tag::Int | Tag::Long => {
                self.consume_integral_unchecked(tag)?;
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

    fn current_values_tag(&self) -> Result<Tag> {
        let layer = self.layers.last().ok_or(Error::Message(format!(
            "expected to be in a compound or list",
        )))?;

        match layer {
            Layer::Compound(Some(tag)) => Ok(tag.clone()),
            Layer::List(tag, _) => Ok(tag.clone()),
            Layer::Compound(None) => Err(Error::Message(
                "expected to be in compound, but do not know what to deserialize".to_owned(),
            )),
        }
    }

    fn type_check(&mut self, tag: Tag, serde_type: &'static str) -> Result<()> {
        if self.current_values_tag()? != tag {
            Err(Error::TypeMismatch(self.current_values_tag()?, serde_type))
        } else {
            Ok(())
        }
    }

    fn type_check_floating_points(&mut self) -> Result<()> {
        let current = self.current_values_tag()?;

        if current != Tag::Float || current != Tag::Double {
            Err(Error::TypeMismatch(
                self.current_values_tag()?,
                "float/double",
            ))
        } else {
            Ok(())
        }
    }
}

fn u8_to_tag(tag: u8) -> Result<Tag> {
    Tag::try_from(tag).or_else(|_| Err(Error::InvalidTag(tag)))
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
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
        self.deserialize_bytes(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let layer = self.layers.last().ok_or(Error::Message(format!(
            "expected bytes, but not in a compound or list",
        )))?;

        match layer {
            Layer::List(tag, size) => Err(Error::Message(format!(
                "expected bytes, got [{:?}; {}]",
                tag, size
            ))),
            Layer::Compound(None) => Err(Error::Message(
                "expected bytes, but do not know what to deserialize".to_owned(),
            )),
            Layer::Compound(Some(Tag::List)) => {
                let el = self.consume_tag()?;
                let size = self.consume_list_size()?;

                match el {
                    Tag::Byte => {
                        let bs = self.consume_bytes_unchecked(size)?;
                        visitor.visit_borrowed_bytes(bs)
                    }
                    _ => Err(Error::Message(format!(
                        "expected bytes, got [{:?}; {}]",
                        el, size
                    ))),
                }
            }
            Layer::Compound(Some(tag)) => match tag {
                Tag::ByteArray => {
                    let size = self.consume_list_size()?;
                    let bs = self.consume_bytes_unchecked(size)?;
                    visitor.visit_borrowed_bytes(bs)
                }
                Tag::IntArray => {
                    let size = self.consume_list_size()?;
                    let bs = self.consume_bytes_unchecked(size * 4i32)?;
                    visitor.visit_borrowed_bytes(bs)
                }
                // This allows us to borrow blockstates rather than copy them.
                Tag::LongArray => {
                    let size = self.consume_list_size()?;
                    let bs = self.consume_bytes_unchecked(size * 8i32)?;
                    visitor.visit_borrowed_bytes(bs)
                }
                _ => Err(Error::Message(format!("expected bytes, found {:?}", tag))),
            },
        }
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("char not supported".to_owned()))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // If the current tag is a string, then we want a unit variant eg
        // enum E { A, B, C }
        match self.current_values_tag()? {
            Tag::String => visitor.visit_enum(UnitVariantAccess { de: self }),
            _ => todo!("non-unit enum variants"),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.type_check_floating_points()?;
        visitor.visit_f32(self.consume_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.type_check_floating_points()?;
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
        self.type_check(Tag::String, "string")?;
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
        // For NBT, an option would just be the absense of the field.
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag = self.current_values_tag()?;
        self.ignore_value(tag)?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("unit_struct not supported".to_owned()))
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag = self.current_values_tag()?;

        match tag {
            Tag::ByteArray | Tag::IntArray | Tag::LongArray => {
                let size = self.consume_list_size()?;
                let non_array_tag = match tag {
                    Tag::ByteArray => Tag::Byte,
                    Tag::IntArray => Tag::Int,
                    Tag::LongArray => Tag::Long,
                    _ => panic!(),
                };

                // Going to pretend we're in a list to reuse the ListAccess.
                self.layers.push(Layer::List(non_array_tag, size));
                let r = visitor.visit_seq(ListAccess::new(self, size));
                self.layers.pop().unwrap();
                r
            }
            Tag::List => {
                // We should be just after the point of reading the name of the list.
                // So we need to read the element type, then the size.
                let element_tag = self.consume_tag()?;
                let size = self.consume_list_size()?;

                self.layers.push(Layer::List(element_tag, size));

                let r = visitor.visit_seq(ListAccess::new(self, size));
                self.layers.pop().unwrap();
                r
            }
            _ => Err(Error::TypeMismatch(tag, "seq")),
        }
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
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

        let layer = self
            .layers
            .last()
            .ok_or(Error::Message(format!(
                "expected unwanted payload, but not in a compound or list",
            )))?
            .clone();

        match layer {
            Layer::Compound(Some(tag)) => {
                self.ignore_value(tag)?;
            }
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
        self.de.layers.pop().unwrap();
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
    hint: i32,
}

impl<'a, 'de> ListAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, hint: i32) -> Self {
        Self { de, hint }
    }
}

impl<'a, 'de> SeqAccess<'de> for ListAccess<'a, 'de> {
    type Error = Error;

    fn size_hint(&self) -> Option<usize> {
        self.hint.try_into().ok()
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
            Layer::Compound(tag) => Err(Error::Message(format!(
                "expected to be in list, but was in compound {:?}",
                tag
            ))),
        }
    }
}

struct UnitVariantAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EnumAccess<'de> for UnitVariantAccess<'a, 'de> {
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

impl<'a, 'de> VariantAccess<'de> for UnitVariantAccess<'a, 'de> {
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
        V: Visitor<'de>,
    {
        todo!("unit variant: variant")
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!("unit variant: struct variant")
    }
}

#[derive(Clone)]
enum Layer {
    List(Tag, i32),        // Tag of elements, number of elements left.
    Compound(Option<Tag>), // Tag is the type of the next expected value.
}
