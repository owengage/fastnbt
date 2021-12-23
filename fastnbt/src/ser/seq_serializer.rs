use std::{convert::TryInto, io::Write};

use byteorder::{BigEndian, WriteBytesExt};
use serde::{
    ser::{self, Impossible},
    Serialize,
};

use crate::{
    error::{Error, Result},
    Tag,
};

use super::{
    name_serializer::NameSerializer,
    serializer::{Serializer, SerializerTuple},
    write_nbt::WriteNbt,
    AnonSerializer,
};

#[derive(Debug)]
pub(crate) enum SeqState {
    First(usize), // len
    Rest,
}

pub struct SeqSerializer<'a, W: Write> {
    pub(crate) writer: W,
    pub(crate) state: &'a mut SeqState,
}

impl<'a, W: Write> SeqSerializer<'a, W> {
    fn try_write_len(&mut self, tag: Tag) -> Result<()> {
        if let SeqState::First(len) = self.state {
            self.writer.write_tag(tag)?;
            self.writer.write_u32::<BigEndian>(
                (*len)
                    .try_into()
                    .map_err(|e| Error::bespoke("len too large".to_owned()))?,
            )?;
            *self.state = SeqState::Rest;
            println!("Wrote len...");
        }
        Ok(())
    }
}

impl<'a, W: 'a + Write> serde::ser::Serializer for &'a mut SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = SerializerTuple<'a, W>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = &'a mut Serializer<'a, W>;
    type SerializeStruct = &'a mut Serializer<'a, W>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.try_write_len(Tag::Byte)?;
        self.writer.write_u8(v as u8)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.try_write_len(Tag::Byte)?;
        self.writer.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.try_write_len(Tag::Short)?;
        self.writer.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.try_write_len(Tag::Int)?;
        self.writer.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.try_write_len(Tag::Long)?;
        self.writer.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.try_write_len(Tag::Byte)?;
        self.writer.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.try_write_len(Tag::Short)?;
        self.writer.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.try_write_len(Tag::Int)?;
        self.writer.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.try_write_len(Tag::Long)?;
        self.writer.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.try_write_len(Tag::Float)?;
        self.writer.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.try_write_len(Tag::Double)?;
        self.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.try_write_len(Tag::Int)?;
        self.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.try_write_len(Tag::String)?;
        self.writer.write_size_prefixed_str(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<()> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
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

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.writer.write_tag(Tag::List)?;
        self.try_write_len(Tag::Byte)?;
        // self.writer.write_tag(Tag::Int)?; // FIXME: How do we know the tag
        // self.writer.write_i32::<BigEndian>(
        //     len.try_into()
        //         .map_err(|_| Error::bespoke("tuple len greater than i32::MAX".into()))?,
        // )?;

        todo!()
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

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.writer.write_tag(Tag::Compound)?;
        self.try_write_len(Tag::Byte)?;

        // ???
        // We need something to return here, but whatever we return is going to
        // expect to go through the entire map... But we need to write the field
        // name here before we can forward on to the next deserializer.
        //
        // Maybe this needs to be passed the field name and become the
        // TagFieldSerializer instead. It can then just create the serializer...
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
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
