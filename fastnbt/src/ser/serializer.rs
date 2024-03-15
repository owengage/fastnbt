use std::{io::Write, mem};

use byteorder::{BigEndian, WriteBytesExt};
use serde::{
    ser::{self, Impossible, SerializeTuple},
    Serialize,
};

use crate::{
    error::{Error, Result},
    Tag, BYTE_ARRAY_TOKEN, INT_ARRAY_TOKEN, LONG_ARRAY_TOKEN,
};

use super::{
    array_serializer::ArraySerializer, name_serializer::NameSerializer, write_nbt::WriteNbt,
};

enum DelayedHeader {
    List { len: usize }, // header for a list, so element tag and list size.
    MapEntry { outer_name: Vec<u8> }, // header for a compound, so tag, name of compound.
    Root { root_name: String }, // root compound, special because it isn't allowed to be an array type. Must be compound.
}

pub struct Serializer<W: Write> {
    pub(crate) writer: W,

    // Desired name of the root compound, typically an empty string.
    // NOTE: This is `mem:take`en, so is only valid at the start of serialization!
    pub(crate) root_name: String,
}

macro_rules! no_root {
    ($v:ident($($t:ty),* $(,)?) -> $r:ty) => {
        fn $v(self, $(_: $t),*) -> Result<$r> {
            Err(Error::no_root_compound())
        }
    };

    ($v:ident<T>($($t:ty),* $(,)?)) => {
        fn $v<T: ?Sized>(self, $(_: $t),*) -> Result<Self::Ok> {
            Err(Error::no_root_compound())
        }
    };

    ($v:ident($($t:ty),* $(,)?)) => {
        no_root!{$v($($t,)*) -> ()}
    };
}

impl<'a, W: 'a + Write> serde::ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializerTuple<'a, W>;
    type SerializeTuple = SerializerTuple<'a, W>;
    type SerializeTupleStruct = SerializerTuple<'a, W>;
    type SerializeTupleVariant = SerializerTuple<'a, W>;
    type SerializeMap = SerializerMap<'a, W>;
    type SerializeStruct = SerializerMap<'a, W>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        // Take the root name to avoid a clone. Need to be careful not to use
        // self.root_name elsewhere.
        let root_name = mem::take(&mut self.root_name);
        Ok(SerializerMap {
            ser: self,
            key: None,
            header: Some(DelayedHeader::Root { root_name }),
            trailer: Some(Tag::End),
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    no_root!(serialize_bool(bool));
    no_root!(serialize_i8(i8));
    no_root!(serialize_i16(i16));
    no_root!(serialize_i32(i32));
    no_root!(serialize_i64(i64));
    no_root!(serialize_i128(i128));
    no_root!(serialize_u8(u8));
    no_root!(serialize_u16(u16));
    no_root!(serialize_u32(u32));
    no_root!(serialize_u64(u64));
    no_root!(serialize_u128(u128));
    no_root!(serialize_f32(f32));
    no_root!(serialize_f64(f64));
    no_root!(serialize_char(char));
    no_root!(serialize_str(&str));
    no_root!(serialize_bytes(&[u8]));
    no_root!(serialize_none());
    no_root!(serialize_some<T>(&T));
    no_root!(serialize_unit());
    no_root!(serialize_unit_struct(&'static str));
    no_root!(serialize_unit_variant(&'static str, u32, &'static str));
    no_root!(serialize_newtype_variant<T>(&'static str, u32, &'static str, &T));
    no_root!(serialize_seq(Option<usize>) -> Self::SerializeSeq);
    no_root!(serialize_tuple(usize) -> Self::SerializeSeq);
    no_root!(serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct);

    no_root!(
        serialize_tuple_variant(
            &'static str,
            u32,
            &'static str,
            usize,
        ) -> Self::SerializeTupleVariant
    );

    no_root!(
        serialize_struct_variant(
            &'static str,
            u32,
            &'static str,
            usize,
        ) -> Self::SerializeStructVariant
    );
}

pub struct SerializerMap<'a, W: Write> {
    ser: &'a mut Serializer<W>,
    key: Option<Vec<u8>>,
    header: Option<DelayedHeader>,
    trailer: Option<Tag>,
}

fn write_header(writer: &mut impl Write, header: DelayedHeader, actual_tag: Tag) -> Result<()> {
    match header {
        DelayedHeader::Root {
            root_name: outer_name,
        } => {
            if actual_tag != Tag::Compound {
                // TODO: Test case for this.
                return Err(Error::no_root_compound());
            }
            writer.write_tag(Tag::Compound)?;
            writer.write_size_prefixed_str(&outer_name)?;
        }
        DelayedHeader::MapEntry { ref outer_name } => {
            writer.write_tag(actual_tag)?;
            writer.write_u16::<BigEndian>(outer_name.len() as u16)?;
            writer.write_all(outer_name)?;
        }
        DelayedHeader::List { len } => {
            writer.write_tag(actual_tag)?;
            writer.write_len(len)?;
        }
    };
    Ok(())
}

impl<'a, W: Write> serde::ser::SerializeMap for SerializerMap<'a, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        // Get the name ahead of time.
        let mut name = Vec::new();
        key.serialize(&mut NameSerializer { name: &mut name })?;
        self.key = Some(name);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let name = self.key.take().ok_or_else(|| {
            Error::bespoke("serialize_value called before serialize_key".to_string())
        })?;

        let outer_tag = match std::str::from_utf8(&name) {
            Ok(BYTE_ARRAY_TOKEN) => Tag::ByteArray,
            Ok(INT_ARRAY_TOKEN) => Tag::IntArray,
            Ok(LONG_ARRAY_TOKEN) => Tag::LongArray,
            _ => Tag::Compound,
        };

        if let Some(header) = self.header.take() {
            write_header(&mut self.ser.writer, header, outer_tag)?;
        }

        match std::str::from_utf8(&name) {
            Ok(BYTE_ARRAY_TOKEN) => {
                self.trailer = None;
                value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::ByteArray,
                })
            }
            Ok(INT_ARRAY_TOKEN) => {
                self.trailer = None;
                value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::IntArray,
                })
            }
            Ok(LONG_ARRAY_TOKEN) => {
                self.trailer = None;
                value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::LongArray,
                })
            }
            _ => value.serialize(&mut Delayed {
                ser: &mut *self.ser,
                header: Some(DelayedHeader::MapEntry { outer_name: name }),
                is_list: false,
            }),
        }
    }

    fn end(mut self) -> Result<()> {
        if let Some(tag) = self.trailer {
            if let Some(header) = self.header.take() {
                // if we still have a header, that means that we haven't seen a
                // single key, so it must be an empty compound, we need to write
                // the bytes we have delayed then close off the compound.
                write_header(&mut self.ser.writer, header, Tag::Compound)?;
            }
            self.ser.writer.write_tag(tag)?;
        }
        Ok(())
    }
}

impl<'a, W: Write> serde::ser::SerializeStruct for SerializerMap<'a, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

pub struct SerializerTuple<'a, W: Write> {
    pub(crate) ser: &'a mut Serializer<W>,
    pub(crate) len: usize,
    pub(crate) first: bool,
}

impl<'a, W: 'a + Write> serde::ser::SerializeSeq for SerializerTuple<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        <Self as serde::ser::SerializeTuple>::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        <Self as serde::ser::SerializeTuple>::end(self)
    }
}

impl<'a, W: 'a + Write> serde::ser::SerializeTuple for SerializerTuple<'a, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut Delayed {
            ser: self.ser,
            header: self.first.then_some(DelayedHeader::List { len: self.len }),
            is_list: true,
        })?;
        self.first = false;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: 'a + Write> serde::ser::SerializeTupleStruct for SerializerTuple<'a, W> {
    type Ok = ();

    type Error = Error;

    fn end(self) -> Result<()> {
        Ok(())
    }

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.serialize_element(value)
    }
}

impl<'a, W: 'a + Write> serde::ser::SerializeTupleVariant for SerializerTuple<'a, W> {
    type Ok = ();

    type Error = Error;

    fn end(self) -> Result<()> {
        Ok(())
    }

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.serialize_element(value)
    }
}

struct Delayed<'a, W: Write + 'a> {
    ser: &'a mut Serializer<W>,
    header: Option<DelayedHeader>,
    is_list: bool,
}

impl<'a, W: Write + 'a> Delayed<'a, W> {
    fn write_header(&mut self, tag: Tag) -> Result<()> {
        if let Some(header) = self.header.take() {
            write_header(&mut self.ser.writer, header, tag)?;
        }
        Ok(())
    }
}

impl<'a, W: 'a + Write> serde::ser::Serializer for &'a mut Delayed<'a, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializerTuple<'a, W>;
    type SerializeTuple = SerializerTuple<'a, W>;
    type SerializeTupleStruct = SerializerTuple<'a, W>;
    type SerializeTupleVariant = SerializerTuple<'a, W>;
    type SerializeMap = SerializerMap<'a, W>;
    type SerializeStruct = SerializerMap<'a, W>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write_header(Tag::Byte)?;
        self.ser.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_header(Tag::Byte)?;
        self.ser.writer.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_header(Tag::Short)?;
        self.ser.writer.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_header(Tag::Int)?;
        self.ser.writer.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_header(Tag::Long)?;
        self.ser.writer.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.write_header(Tag::IntArray)?;
        self.ser.writer.write_len(4)?;
        self.ser.writer.write_u32::<BigEndian>((v >> 96) as u32)?;
        self.ser.writer.write_u32::<BigEndian>((v >> 64) as u32)?;
        self.ser.writer.write_u32::<BigEndian>((v >> 32) as u32)?;
        self.ser.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_header(Tag::Byte)?;
        self.ser.writer.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_header(Tag::Short)?;
        self.ser.writer.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_header(Tag::Int)?;
        self.ser.writer.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.write_header(Tag::Long)?;
        self.ser.writer.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.write_header(Tag::Float)?;
        self.ser.writer.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.write_header(Tag::Double)?;
        self.ser.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.write_header(Tag::Int)?;
        self.ser.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_header(Tag::String)?;
        self.ser.writer.write_size_prefixed_str(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_header(Tag::List)?;
        self.ser.writer.write_tag(Tag::Byte)?;
        self.ser.writer.write_len(v.len())?;
        self.ser.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        // we could write the list with None's missing, but that would require
        // us to delay writing the length of the list until we're finished
        // serializing the actua list. Users can filter a list with None values.
        if self.is_list {
            Err(Error::bespoke("cannot serialize None in list".to_string()))
        } else {
            Ok(())
        }
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::bespoke("cannot serialize unit: ()".to_string()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        Err(Error::bespoke(format!(
            "cannot serialize unit struct: {name}"
        )))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.write_header(Tag::String)?;
        self.ser.writer.write_size_prefixed_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        Err(Error::bespoke(
            "cannot serialize newtype variant, please open fastnbt issue".to_string(),
        ))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len =
            len.ok_or_else(|| Error::bespoke("sequences must have a known length".to_string()))?;

        self.serialize_tuple(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.write_header(Tag::List)?;
        if len == 0 {
            // Weird case, serialize_element will never be called so we won't
            // get chance to write the tag type of the list. Worse still we have
            // no idea what the element type of this list will be because the
            // relevant serialize call never happens.

            // This is talked about a bit here:
            // https://minecraft.wiki/w/NBT_format
            // A list of end tags seems to be the way to go.

            self.ser.writer.write_tag(Tag::End)?;
            self.ser.writer.write_u32::<BigEndian>(0)?; // ie len
        }

        Ok(SerializerTuple {
            ser: self.ser,
            first: true,
            len,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializerMap {
            ser: self.ser,
            key: None,
            header: self.header.take(),
            trailer: Some(Tag::End),
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::bespoke(
            "cannot serialize struct variant, please open fastnbt issue".to_string(),
        ))
    }
}
