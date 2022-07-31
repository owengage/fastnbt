use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{
    ser::{self, Impossible, SerializeTuple},
    serde_if_integer128, Serialize,
};

use crate::{
    error::{Error, Result},
    IntArray, Tag, BYTE_ARRAY_TOKEN, INT_ARRAY_TOKEN, LONG_ARRAY_TOKEN,
};

use super::{
    array_serializer::ArraySerializer, name_serializer::NameSerializer, write_nbt::WriteNbt,
};

pub struct Serializer<W: Write> {
    pub(crate) writer: W,
    pub(crate) seen_root: bool,
}

impl<'a, W: Write> Serializer<W> {
    // fn try_write_header(&mut self, tag: Tag) -> Result<()> {
    //     match &mut self.state {
    //         State::ListStart { len } => {
    //             self.writer.write_tag(tag)?;
    //             self.writer.write_len(*len)?;
    //             self.state = State::ListRest;
    //         }
    //         State::ListRest => {}
    //         State::Compound { current_field } => {
    //             self.writer.write_tag(tag)?;
    //             self.writer.write_size_prefixed_str(current_field)?;
    //         }
    //     }
    //     Ok(())
    // }
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

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer.write_u8(v as u8)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.writer.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.writer.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.writer.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.writer.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<()> {
            IntArray::new(vec![
                (v >> 96) as i32,
                (v >> 64) as i32,
                (v >> 32) as i32,
                v as i32,
            ]).serialize(self)
        }

        fn serialize_u128(self, v: u128) -> Result<()> {
            self.serialize_i128(v as i128)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.writer.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.writer.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.writer.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.writer.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.writer.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.writer.write_size_prefixed_str(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.writer.write_tag(Tag::Byte)?;
        self.writer.write_len(v.len())?;
        self.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        // TODO: What happens if we serialize a list of optionals?
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.writer.write_size_prefixed_str(variant)
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
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len =
            len.ok_or_else(|| Error::bespoke("sequences must have a known length".to_string()))?;

        self.serialize_tuple(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        if len == 0 {
            // Weird case, serialize_element will never be called so we won't
            // get chance to write the tag type of the list. Worse still we have
            // no idea what the element type of this list will be because the
            // relevant serialize call never happens.

            // This is talked about a bit here:
            // https://minecraft.fandom.com/wiki/NBT_format
            // A list of end tags seems to be the way to go.

            self.writer.write_tag(Tag::End)?;
            self.writer.write_u32::<BigEndian>(0)?; // ie len
        }

        Ok(SerializerTuple {
            ser: self,
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
        if !self.seen_root {
            u8::from(Tag::Compound).serialize(&mut *self)?;
            "".serialize(&mut *self)?;
            self.seen_root = true;
        }

        Ok(SerializerMap {
            ser: self,
            first: true,
            // name: "", // will be blank for root, but what about inner?
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
        todo!()
    }
}

pub struct SerializerMap<'a, W: Write> {
    ser: &'a mut Serializer<W>,
    first: bool,
}

impl<'ser, 'a, W: Write> serde::ser::SerializeMap for SerializerMap<'a, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        self.ser.writer.write_tag(Tag::End)
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        // Get the name ahead of time.
        let mut name = Vec::new();
        key.serialize(&mut NameSerializer { name: &mut name })?;

        match std::str::from_utf8(&name) {
            Ok(BYTE_ARRAY_TOKEN) => {
                return value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::ByteArray,
                });
            }
            Ok(INT_ARRAY_TOKEN) => {
                return value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::IntArray,
                })
            }
            Ok(LONG_ARRAY_TOKEN) => {
                return value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::LongArray,
                })
            }
            _ => {}
        };

        // TODO

        // At this point, we still need to write the tag and name of the current
        // field, but we can't know the tag to write until we start serializing
        // the value. We need to store these up somewhere and write them in one
        // go. I think a new serializer type should be created here while passed
        // the name. Once a serialize call is made for the value it writes the
        // appropriate tag, then name, then calls the core serializer to write
        // the value.
        value.serialize(&mut DelayedEntry {
            ser: &mut *self.ser,
            name,
        })
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
        self.ser.writer.write_tag(Tag::End)?;
        Ok(())
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
        value.serialize(&mut DelayedList {
            ser: self.ser,
            first: self.first,
            len: self.len,
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

struct DelayedEntry<'a, W: Write + 'a> {
    ser: &'a mut Serializer<W>,
    name: Vec<u8>,
}

impl<'a, W: Write + 'a> DelayedEntry<'a, W> {
    fn write_header(&mut self, tag: Tag) -> Result<()> {
        self.ser.writer.write_tag(tag)?;
        self.ser
            .writer
            .write_u16::<BigEndian>(self.name.len() as u16)?;

        self.ser.writer.write_all(&self.name)?;
        Ok(())
    }
}

impl<'a, W: 'a + Write> serde::ser::Serializer for &'a mut DelayedEntry<'a, W> {
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
        self.ser.serialize_i8(v)
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
        self.write_header(Tag::IntArray)?;
        IntArray::new(vec![
            (v >> 96) as i32,
            (v >> 64) as i32,
            (v >> 32) as i32,
            v as i32,
        ])
        .serialize(self)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.serialize_i128(v as i128)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_header(Tag::Byte)?;
        self.ser.serialize_u8(v)
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
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        // TODO: What happens if we serialize a list of optionals?
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
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
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
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
            // https://minecraft.fandom.com/wiki/NBT_format
            // A list of end tags seems to be the way to go.

            self.ser.writer.write_tag(Tag::End)?;
            self.ser.writer.write_u32::<BigEndian>(0)?; // ie len
        }

        // We should be writing the tag and len of the list here, but we don't
        // have the tag yet. Same problem we have with fields in general. Can we
        // repeat the technique of delaying serializing it?
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
        self.write_header(Tag::Compound)?;
        Ok(SerializerMap {
            ser: self.ser,
            first: false,
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
        todo!()
    }
}

struct DelayedList<'a, W: Write + 'a> {
    ser: &'a mut Serializer<W>,
    len: usize,
    first: bool,
}

impl<'a, W: Write + 'a> DelayedList<'a, W> {
    fn write_header(&mut self, tag: Tag) -> Result<()> {
        if self.first {
            self.ser.writer.write_tag(tag)?;
            self.ser.writer.write_u32::<BigEndian>(self.len as u32)?;
        }
        Ok(())
    }
}

impl<'a, W: 'a + Write> serde::ser::Serializer for &'a mut DelayedList<'a, W> {
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
        self.ser.serialize_i8(v)
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
        self.write_header(Tag::IntArray)?;
        IntArray::new(vec![
            (v >> 96) as i32,
            (v >> 64) as i32,
            (v >> 32) as i32,
            v as i32,
        ])
        .serialize(self)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.serialize_i128(v as i128)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_header(Tag::Byte)?;
        self.ser.serialize_u8(v)
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
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        // TODO: What happens if we serialize a list of optionals?
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
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
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
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
            // https://minecraft.fandom.com/wiki/NBT_format
            // A list of end tags seems to be the way to go.

            self.ser.writer.write_tag(Tag::End)?;
            self.ser.writer.write_u32::<BigEndian>(0)?; // ie len
        }

        // We should be writing the tag and len of the list here, but we don't
        // have the tag yet. Same problem we have with fields in general. Can we
        // repeat the technique of delaying serializing it?
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
        self.write_header(Tag::Compound)?;
        Ok(SerializerMap {
            ser: self.ser,
            first: false,
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
        todo!()
    }
}
