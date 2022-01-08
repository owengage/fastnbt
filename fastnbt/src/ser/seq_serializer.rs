use std::{convert::TryInto, io::Write};

use byteorder::{BigEndian, WriteBytesExt};
use serde::{
    ser::{self, Impossible},
    Deserialize, Serialize,
};

use crate::{
    error::{Error, Result},
    Tag,
};

use super::{
    array_serializer::ArraySerializer,
    name_serializer::NameSerializer,
    serializer::{Serializer, SerializerTuple},
    write_nbt::WriteNbt,
};

#[derive(Debug)]
pub(crate) enum SeqState {
    ListStart(usize), // len
    Rest,
}

pub struct SeqSerializer<'a, W: Write> {
    pub(crate) ser: &'a mut Serializer<W>,
    pub(crate) state: &'a mut SeqState,
}

impl<'a, W: Write> SeqSerializer<'a, W> {
    fn try_write_list_header(&mut self, tag: Tag) -> Result<()> {
        if let SeqState::ListStart(len) = self.state {
            self.ser.writer.write_tag(tag)?;
            self.ser.writer.write_u32::<BigEndian>(
                (*len)
                    .try_into()
                    .map_err(|e| Error::bespoke("len too large".to_owned()))?,
            )?;
            *self.state = SeqState::Rest;
        }
        Ok(())
    }
}

impl<'a, W: 'a + Write> serde::ser::Serializer for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializerTuple<'a, W>;
    type SerializeTuple = SerializerTuple<'a, W>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = &'a mut Serializer<W>;
    type SerializeStruct = &'a mut Serializer<W>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(mut self, v: bool) -> Result<()> {
        self.try_write_list_header(Tag::Byte)?;
        self.ser.writer.write_u8(v as u8)?;
        Ok(())
    }

    fn serialize_i8(mut self, v: i8) -> Result<()> {
        self.try_write_list_header(Tag::Byte)?;
        self.ser.writer.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(mut self, v: i16) -> Result<()> {
        self.try_write_list_header(Tag::Short)?;
        self.ser.writer.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(mut self, v: i32) -> Result<()> {
        self.try_write_list_header(Tag::Int)?;
        self.ser.writer.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(mut self, v: i64) -> Result<()> {
        self.try_write_list_header(Tag::Long)?;
        self.ser.writer.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u8(mut self, v: u8) -> Result<()> {
        self.try_write_list_header(Tag::Byte)?;
        self.ser.writer.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(mut self, v: u16) -> Result<()> {
        self.try_write_list_header(Tag::Short)?;
        self.ser.writer.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(mut self, v: u32) -> Result<()> {
        self.try_write_list_header(Tag::Int)?;
        self.ser.writer.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(mut self, v: u64) -> Result<()> {
        self.try_write_list_header(Tag::Long)?;
        self.ser.writer.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(mut self, v: f32) -> Result<()> {
        self.try_write_list_header(Tag::Float)?;
        self.ser.writer.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(mut self, v: f64) -> Result<()> {
        self.try_write_list_header(Tag::Double)?;
        self.ser.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(mut self, v: char) -> Result<()> {
        self.try_write_list_header(Tag::Int)?;
        self.ser.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_str(mut self, v: &str) -> Result<()> {
        self.try_write_list_header(Tag::String)?;
        self.ser.writer.write_size_prefixed_str(v)?;
        Ok(())
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<()> {
        todo!()
    }

    fn serialize_none(mut self) -> Result<()> {
        // what does it mean to have an optional thing in a list?
        unimplemented!()
    }

    fn serialize_some<T: ?Sized>(mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        // what does it mean to have an optional thing in a list?
        unimplemented!()
    }

    fn serialize_unit(mut self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<()> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        match name {
            "__fastnbt_byte_array" => {
                self.try_write_list_header(Tag::ByteArray)?;
                value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::ByteArray,
                })
            }
            "__fastnbt_int_array" => {
                self.try_write_list_header(Tag::IntArray)?;
                value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::IntArray,
                })
            }
            "__fastnbt_long_array" => {
                self.try_write_list_header(Tag::LongArray)?;
                value.serialize(ArraySerializer {
                    ser: self.ser,
                    tag: Tag::LongArray,
                })
            }
            _ => todo!(),
        }
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len =
            len.ok_or_else(|| Error::bespoke("sequences must have a known length".to_string()))?;

        self.serialize_tuple(len)
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple> {
        self.try_write_list_header(Tag::List)?;

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

        Ok(SerializerTuple {
            ser: self.ser,
            state: SeqState::ListStart(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.try_write_list_header(Tag::Compound)?;

        // We're in a list at the moment, so the compound we're about to write
        // is anonymouse, ie the name and tag of the compound are effectively
        // already written, so we just need to write out the fields of the compound.
        Ok(self.ser)
    }

    fn serialize_struct(mut self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}
