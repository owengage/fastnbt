use std::{io::Write, vec};

use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;

use crate::{
    error::{Error, Result},
    Tag,
};

pub fn to_bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut result = vec![];
    let mut serializer = Serializer {
        writer: WriteHelper { inner: &mut result },
        rooted: false,
        field_name: "",
    };
    v.serialize(&mut serializer)?;
    Ok(result)
}

struct WriteHelper<W: Write> {
    inner: W,
}

impl<W: Write> WriteHelper<W> {
    fn write_tag(&mut self, tag: Tag) -> Result<()> {
        self.inner.write_u8(tag as u8)?;
        Ok(())
    }

    fn write_size_prefixed_str(&mut self, key: &str) -> Result<()> {
        let key = cesu8::to_java_cesu8(key);
        let len_bytes = key.len() as u16;
        self.inner.write_u16::<BigEndian>(len_bytes)?;
        self.inner.write_all(&key)?;
        Ok(())
    }
}

pub struct Serializer<'ser, W: Write> {
    writer: WriteHelper<W>,
    rooted: bool,
    field_name: &'ser str,
}

impl<'ser, 'a, W: Write> serde::Serializer for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Byte)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Short)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Int)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Long)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Byte)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Short)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Int)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.writer.write_tag(Tag::Long)?;
        self.writer.write_size_prefixed_str(self.field_name)?;
        self.writer.inner.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
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
        if self.rooted {
            // Nothing to be done, the name should already have been written if
            // necessary.
        } else {
            self.rooted = true;
            self.writer.write_tag(Tag::Compound)?;
            self.writer.write_size_prefixed_str("")?;
        }

        Ok(self)
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

impl<'ser, 'a, W: Write> serde::ser::SerializeSeq for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<'ser, 'a, W: Write> serde::ser::SerializeMap for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<'ser, 'a, W: Write> serde::ser::SerializeTuple for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<'ser, 'a, W: Write> serde::ser::SerializeTupleStruct for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<'ser, 'a, W: Write> serde::ser::SerializeTupleVariant for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<'ser, 'a, W: Write> serde::ser::SerializeStruct for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        // Payload
        value.serialize(&mut Serializer {
            writer: WriteHelper {
                inner: &mut (**self).writer.inner,
            },
            rooted: true,
            field_name: key,
        })?;

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.writer.write_tag(Tag::End)?;
        Ok(())
    }
}

impl<'ser, 'a, W: Write> serde::ser::SerializeStructVariant for &'a mut Serializer<'ser, W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}
