//! This module contains a serde deserializer. It can do most of the things you
//! would expect of a typical serde deserializer, such as deserializing into:
//! * Rust structs.
//! * containers like `HashMap` and `Vec`.
//! * an arbitrary [`Value`][`crate::Value`].
//! * enums. For NBT typically you want either internally or untagged enums.
//!
//! This deserializer supports [`from_bytes`][`crate::from_bytes`] for zero-copy
//! deserialization for types like `&[u8]` and
//! [`borrow::LongArray`][`crate::borrow::LongArray`]. There is also
//! [`from_reader`][`crate::from_reader`] for deserializing from types
//! implementing [`Read`][`std::io::Read`].
//!
//! # Avoiding allocations
//!
//! When using [`from_bytes`][`crate::from_bytes`], we can avoid allocations for
//! things like strings and vectors, instead deserializing into a reference to
//! the input data.
//!
//! The following table summarises what types you likely want to store NBT data
//! in for owned or borrowed types:
//!
//! | NBT type | Owned type | Borrowed type |
//! | -------- | ---------- | ------------- |
//! | Byte | `u8` or `i8` | use owned |
//! | Short | `u16` or `i16` | use owned |
//! | Int | `i32` or `u32` | use owned |
//! | Long | `i64` or `u64` | use owned |
//! | Float | `f32` | use owned |
//! | Double | `f64` | use owned |
//! | String | `String` | [`Cow<'a, str>`][`std::borrow::Cow`] or `&[u8]` (see below) |
//! | List | `Vec<T>` | use owned |
//! | Byte Array | [`ByteArray`][`crate::ByteArray`] | [`borrow::ByteArray`][`crate::borrow::ByteArray`] |
//! | Int Array | [`IntArray`][`crate::IntArray`] | [`borrow::IntArray`][`crate::borrow::IntArray`] |
//! | Long Array | [`LongArray`][`crate::LongArray`] | [`borrow::LongArray`][`crate::borrow::LongArray`] |
//!
//! ## Primitives
//!
//! Borrowing for primitive types like the integers and floats is generally not
//! possible due to alignment requirements of those types. It likely wouldn't be
//! faster/smaller anyway.
//!
//! ## Strings
//!
//! For strings, we cannot know ahead of time whether the data can be borrowed
//! as `&str`. This is because Minecraft uses Java's encoding of Unicode, not
//! UTF-8. If the string contains Unicode characters outside of the Basic
//! Multilingual Plane then we need to convert it to UTF-8, requiring us to own
//! the string data.
//!
//! Using [`Cow<'a, str>`][`std::borrow::Cow`] lets us borrow when possible, but
//! produce an owned value when the representation is different.
//!
//! Strings can also be deserialized to `&[u8]` which will always succeed. These
//! bytes will be Java's CESU-8 format. You can use [`cesu8::from_java_cesu8`]
//! to decode this.
//!
//! # Representation of NBT arrays
//!
//! In order for [`Value`][`crate::Value`] to preserve all NBT information, the
//! deserializer "[maps into serde's data
//! model](https://serde.rs/data-model.html#mapping-into-the-data-model)". As a
//! consequence of this, NBT array types must be (de)serialized using the
//! types provided in this crate, eg [LongArray][`crate::LongArray`]. Sequence
//! containers like `Vec` will (de)serialize to NBT Lists, and will fail if an
//! NBT array is instead expected.
//!
//! # 128 bit integers and UUIDs
//!
//! UUIDs tend to be stored in NBT using 4-long IntArrays. When deserializing
//! `i128` or `u128`, IntArray with length 4 are accepted. This is parsed as big
//! endian i.e. the most significant bit (and int) is first.
//!
//! # Other quirks
//!
//! Some other quirks which may not be obvious:
//! * When deserializing to unsigned types such as u32, it will be an error if a
//!   value is negative to avoid unexpected behaviour with wrap-around. This
//!   does not apply to deserializing lists of integrals to `u8` slice or
//!   vectors.
//! * Any integral value from NBT can be deserialized to bool. Any non-zero
//!   value becomes `true`. Bear in mind serializing the same type will change
//!   the NBT structure, likely unintended.
//! * You can deserialize a field to the unit type `()` or unit struct. This
//!   ignores the value but ensures that it existed.
//! * You cannot deserialize into anything other than a `struct` or similar
//!   container eg `HashMap`. This is due to a misalignment between the NBT
//!   format and Rust's types. Attempting to will give an error about no root
//!   compound. This means you can never do `let s: String = from_bytes(...)`.
//!   Serialization of a struct assumes an empty-named compound.
//!
//! # Example Minecraft types
//!
//! This section demonstrates writing types for a few real Minecraft structures.
//!
//! ## Extracting entities as an enum
//!
//! This demonstrates the type that you would need to write in order to extract
//! some subset of entities. This uses a tagged enum in serde, meaning that it
//! will look for a certain field in the structure to tell it what enum variant
//! to deserialize into. We use serde's `other` attribute to not error when an
//! unknown entity type is found.
//!
//! ```rust
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(tag = "id")]
//! enum Entity {
//!    #[serde(rename = "minecraft:bat")]
//!    Bat {
//!        #[serde(rename = "BatFlags")]
//!        bat_flags: i8,
//!    },
//!
//!    #[serde(rename = "minecraft:creeper")]
//!    Creeper { ignited: i8 },
//!
//!    // Entities we haven't coded end up as just 'unknown'.
//!    #[serde(other)]
//!    Unknown,
//! }
//! ```
//!
//! ## Capture unknown entities
//!
//! If you need to capture all entity types, but do not wish to manually type
//! all of them, you can wrap the above entity type in an untagged enum.
//!
//! ```rust
//! use serde::Deserialize;
//! use fastnbt::Value;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(untagged)]
//! enum Entity {
//!     Known(KnownEntity),
//!     Unknown(Value),
//! }

//! #[derive(Deserialize, Debug)]
//! #[serde(tag = "id")]
//! enum KnownEntity {
//!     #[serde(rename = "minecraft:bat")]
//!     Bat {
//!         #[serde(rename = "BatFlags")]
//!         bat_flags: i8,
//!     },

//!     #[serde(rename = "minecraft:creeper")]
//!     Creeper { ignited: i8 },
//! }
//! ```
//!
//! ## Avoiding allocations in a Chunk
//!
//! This example shows how to avoid some allocations. The `Section` type below
//! contains the block states which stores the state of part of the Minecraft
//! world. In NBT this is bit-packed data stored as an array of
//! longs (i64). We avoid allocating a vector for this by storing it as a
//! [`borrow::LongArray`][`crate::borrow::LongArray`] instead, which stores it
//! as `&[u8]` under the hood. We can't safely store it as `&[i64]` due to memory
//! alignment constraints. The `fastanvil` crate has a `PackedBits` type that can
//! handle the unpacking of these block states.
//!
//! ```rust
//! # use serde::Deserialize;
//! use fastnbt::borrow::LongArray;
//!
//! #[derive(Deserialize)]
//! struct Chunk<'a> {
//!     #[serde(rename = "Level")]
//!     #[serde(borrow)]
//!     level: Level<'a>,
//! }
//!
//! #[derive(Deserialize)]
//! struct Level<'a> {
//!     #[serde(rename = "Sections")]
//!     #[serde(borrow)]
//!     pub sections: Option<Vec<Section<'a>>>,
//! }
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(rename_all = "PascalCase")]
//! pub struct Section<'a> {
//!     #[serde(borrow)]
//!     pub block_states: Option<LongArray<'a>>,
//! }
//! ```
//!
//! ## Unit variant enum from status of chunk
//!
//! ```no_run
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct Chunk {
//!     #[serde(rename = "Level")]
//!     level: Level,
//! }
//!
//! #[derive(Deserialize)]
//! struct Level {
//!     #[serde(rename = "Status")]
//!     status: Status,
//! }
//!
//! #[derive(Deserialize, PartialEq, Debug)]
//! #[serde(rename_all = "snake_case")]
//! enum Status {
//!     Empty,
//!     StructureStarts,
//!     StructureReferences,
//!     Biomes,
//!     Noise,
//!     Surface,
//!     Carvers,
//!     LiquidCarvers,
//!     Features,
//!     Light,
//!     Spawn,
//!     Heightmaps,
//!     Full,
//! }
//! ```
use std::io::Read;

use serde::{
    de::{
        self,
        value::{BorrowedBytesDeserializer, BorrowedStrDeserializer, BytesDeserializer},
    },
    forward_to_deserialize_any,
};

use crate::{
    error::{Error, Result},
    input, DeOpts, Tag, BYTE_ARRAY_TOKEN, INT_ARRAY_TOKEN, LONG_ARRAY_TOKEN,
};

use crate::input::{Input, Reference};

/// Deserializer for NBT data. See the [`de`] module for more information.
///
/// [`de`]: ./index.html
pub struct Deserializer<In> {
    input: In,
    scratch: Vec<u8>,
    seen_root: bool,
    opts: DeOpts,
}

impl<'de, In> Deserializer<In>
where
    In: Input<'de>,
{
    pub fn new(input: In, opts: DeOpts) -> Self {
        Self {
            input,
            scratch: Vec::new(),
            seen_root: false,
            opts,
        }
    }
}

impl<'a> Deserializer<input::Slice<'a>> {
    /// Create Deserializer for a `T` from some NBT data slice. See the [`de`] module
    /// for more information.
    ///
    /// [`de`]: ./index.html
    pub fn from_bytes(bytes: &'a [u8], opts: DeOpts) -> Self {
        Deserializer::new(input::Slice { data: bytes }, opts)
    }
}

impl<R: Read> Deserializer<input::Reader<R>> {
    /// Create Deserializer for a `T` from some NBT data. See the [`de`] module
    /// for more information.
    ///
    /// [`de`]: ./index.html
    pub fn from_reader(reader: R, opts: DeOpts) -> Self {
        Deserializer::new(input::Reader { reader }, opts)
    }
}

impl<'de, 'a, In> de::Deserializer<'de> for &'a mut Deserializer<In>
where
    In: Input<'de>,
{
    type Error = Error;

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct
        identifier ignored_any bytes enum newtype_struct byte_buf option
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if !self.seen_root {
            let peek = self.input.consume_tag()?;

            match peek {
                Tag::Compound => {
                    if self.opts.expect_coumpound_names {
                        self.input.ignore_str()?
                    }
                }
                _ => return Err(Error::no_root_compound()),
            }

            self.seen_root = true;
        }

        visitor.visit_map(MapAccess::new(self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
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

fn arr_check(key: &str) -> Result<&str> {
    if key.starts_with("__")
        && (key == BYTE_ARRAY_TOKEN || key == INT_ARRAY_TOKEN || key == LONG_ARRAY_TOKEN)
    {
        Err(Error::bespoke(
            "compound using special fastnbt array tokens".to_string(),
        ))
    } else {
        Ok(key)
    }
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
            Reference::Borrowed(s) => visitor.visit_borrowed_str(arr_check(s)?),
            Reference::Copied(s) => visitor.visit_str(arr_check(s)?),
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

    forward_to_deserialize_any!(u8 u16 u32 u64 i8 i16 i32 i64 f32
        f64 str string struct tuple map identifier char);

    fn deserialize_any<V>(mut self, v: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_hint = self.last_hint;
        self.last_hint = Hint::None;

        match self.tag {
            Tag::End => Err(Error::bespoke("expected value, found end tag".into())),
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

                // End values have no payload. An end tag on it's own is the payload
                // of an empty compound. A logical interpretation is that this could
                // be a list of zero-sized units, but this mean an easy short
                // malicious payload of a massive list taking up lots of memory (as
                // the Value type's unit variant would not be zero sized.
                //
                // Some old chunks store empty lists as as 'list of end', so if the
                // size is zero we let it slide.
                if tag == Tag::End && remaining != 0 {
                    return Err(Error::bespoke(
                        "unexpected list of type 'end', which is not supported".into(),
                    ));
                }

                if remaining > self.de.opts.max_seq_len {
                    return Err(Error::bespoke(format!(
                        "size ({}) greater than max sequence length ({})",
                        remaining, self.de.opts.max_seq_len,
                    )));
                }

                v.visit_seq(ListAccess {
                    de: self.de,
                    tag,
                    remaining,
                })
            }
            Tag::Compound => v.visit_map(MapAccess::new(self.de)),
            Tag::ByteArray => {
                if let Hint::Seq = last_hint {
                    return Err(Error::array_as_seq());
                }
                let len = self.de.input.consume_i32()? as usize;
                v.visit_map(ArrayWrapperAccess::bytes(self.de, len)?)
            }
            Tag::IntArray => {
                if let Hint::Seq = last_hint {
                    return Err(Error::array_as_seq());
                }
                let len = self.de.input.consume_i32()? as usize;
                v.visit_map(ArrayWrapperAccess::ints(self.de, len)?)
            }
            Tag::LongArray => {
                if let Hint::Seq = last_hint {
                    return Err(Error::array_as_seq());
                }
                let len = self.de.input.consume_i32()? as usize;
                v.visit_map(ArrayWrapperAccess::longs(self.de, len)?)
            }
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let consume_visit =
            |de: &mut Deserializer<In>, len: usize, el_size| match de.input.consume_bytes(
                len.checked_mul(el_size)
                    .ok_or_else(|| Error::bespoke("overflow deserializing bytes".to_string()))?,
                &mut de.scratch,
            )? {
                Reference::Borrowed(bs) => visitor.visit_borrowed_bytes(bs),
                Reference::Copied(bs) => visitor.visit_bytes(bs),
            };

        match self.tag {
            Tag::String => {
                let len = self.de.input.consume_i16()? as usize;
                consume_visit(self.de, len, 1)
            }
            Tag::List => {
                let tag = self.de.input.consume_tag()?;
                let remaining = self.de.input.consume_i32()? as usize;

                match tag {
                    Tag::Byte => consume_visit(self.de, remaining, std::mem::size_of::<i8>()),
                    Tag::Short => consume_visit(self.de, remaining, std::mem::size_of::<i16>()),
                    Tag::Int => consume_visit(self.de, remaining, std::mem::size_of::<i32>()),
                    Tag::Long => consume_visit(self.de, remaining, std::mem::size_of::<i64>()),
                    _ => Err(Error::bespoke(format!(
                        "cannot convert list of {} to bytes",
                        tag
                    ))),
                }
            }
            Tag::ByteArray => {
                let remaining = self.de.input.consume_i32()? as usize;
                consume_visit(self.de, remaining, std::mem::size_of::<i8>())
            }
            Tag::LongArray => {
                let remaining = self.de.input.consume_i32()? as usize;
                consume_visit(self.de, remaining, std::mem::size_of::<i64>())
            }
            _ => Err(Error::bespoke(format!(
                "cannot convert {} to bytes",
                self.tag
            ))),
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

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
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
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(UnitVariantAccess {
            de: AnonymousValue {
                tag: self.tag,
                de: self.de,
                last_hint: Hint::None,
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
        V: de::Visitor<'de>,
    {
        self.last_hint = Hint::Seq;
        self.deserialize_any(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // We specifically allow any intergral type to be deserialized into a
        // bool.
        match self.tag {
            Tag::Byte => visitor.visit_bool(self.de.input.consume_byte()? != 0),
            Tag::Short => visitor.visit_bool(self.de.input.consume_i16()? != 0),
            Tag::Int => visitor.visit_bool(self.de.input.consume_i32()? != 0),
            Tag::Long => visitor.visit_bool(self.de.input.consume_i64()? != 0),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i128(get_i128_value(&mut self)?)
    }

    #[inline]
    fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u128(get_i128_value(&mut self)? as u128)
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

enum State {
    Unread,
    Read,
}

pub(crate) struct ArrayWrapperAccess<'a, In: 'a> {
    de: &'a mut Deserializer<In>,
    token: &'static str,
    bytes_size: usize,
    state: State,
}

impl<'a, In: 'a> ArrayWrapperAccess<'a, In> {
    pub(crate) fn bytes(de: &'a mut Deserializer<In>, size: usize) -> Result<Self> {
        Ok(Self {
            de,
            bytes_size: size
                .checked_mul(1)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?,
            token: BYTE_ARRAY_TOKEN,
            state: State::Unread,
        })
    }

    pub(crate) fn ints(de: &'a mut Deserializer<In>, size: usize) -> Result<Self> {
        Ok(Self {
            de,
            bytes_size: size
                .checked_mul(4)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?,
            token: INT_ARRAY_TOKEN,
            state: State::Unread,
        })
    }

    pub(crate) fn longs(de: &'a mut Deserializer<In>, size: usize) -> Result<Self> {
        Ok(Self {
            de,
            bytes_size: size
                .checked_mul(8)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?,
            token: LONG_ARRAY_TOKEN,
            state: State::Unread,
        })
    }
}

impl<'de, 'a, In: Input<'de> + 'a> de::MapAccess<'de> for ArrayWrapperAccess<'a, In> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let State::Unread = self.state {
            self.state = State::Read;
            seed.deserialize(BorrowedStrDeserializer::new(self.token))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let data = self
            .de
            .input
            .consume_bytes(self.bytes_size, &mut self.de.scratch)?;

        match data {
            Reference::Borrowed(bs) => seed.deserialize(BorrowedBytesDeserializer::new(bs)),
            Reference::Copied(bs) => seed.deserialize(BytesDeserializer::new(bs)),
        }
    }
}

fn get_i128_value<'de, In>(de: &mut AnonymousValue<In>) -> Result<i128>
where
    In: Input<'de>,
{
    let tag = de.tag;

    match tag {
        Tag::IntArray => {
            let size = de.de.input.consume_i32()? as usize;

            let size = size
                .checked_mul(4)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?;

            let bs = de.de.input.consume_bytes(size, &mut de.de.scratch)?;
            let bs = bs.as_ref();

            match bs.try_into() {
                Ok(bs) => Ok(i128::from_be_bytes(bs)),
                Err(_) => Err(Error::bespoke(format!(
                    "deserialize i128: expected IntArray of length 4 with 16 bytes, found {} bytes",
                    bs.len()
                ))),
            }
        }
        _ => Err(Error::bespoke(
            "deserialize i128: expected IntArray value".to_string(),
        )),
    }
}
