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
    array_serializer::ArraySerializer,
    name_serializer::NameSerializer,
    seq_serializer::{SeqSerializer, SeqState},
    write_nbt::WriteNbt,
};

pub struct Serializer<W: Write> {
    pub(crate) writer: W,
    pub(crate) field: String,
}

impl<'a, W: Write> serde::ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializerTuple<'a, W>;
    type SerializeTuple = SerializerTuple<'a, W>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer.write_tag(Tag::Byte)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_u8(v as u8)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.writer.write_tag(Tag::Byte)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.writer.write_tag(Tag::Short)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.writer.write_tag(Tag::Int)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.writer.write_tag(Tag::Long)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.writer.write_tag(Tag::Byte)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.writer.write_tag(Tag::Short)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.writer.write_tag(Tag::Int)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.writer.write_tag(Tag::Long)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.writer.write_tag(Tag::Float)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.writer.write_tag(Tag::Double)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.writer.write_tag(Tag::Int)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.writer.write_tag(Tag::String)?;
        self.writer.write_size_prefixed_str(&self.field)?;
        self.writer.write_size_prefixed_str(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        todo!()
    }

    fn serialize_none(self) -> Result<()> {
        // don't write field at all.
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

    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        // We get here if we are trying to serialize a byte/int/long array
        // because the tag type in it is a unit struct.
        // Even better, the `name` above is 'CompTag'. We can give the *type* a
        // unique name in order to detect this case.

        // Annoyingly I think we'll have entered a map by this point and already
        // written the compound tag.
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
        value.serialize(self)
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
        // We implement Serialize for NBT array types, creating a hidden inner
        // type that specifically causes us to end up in this serialize
        // function.
        //
        // Why a newtype variant? It's the only method that gives us access to a
        // name field we can control AND access to the actual data. This means
        // we can detect we're serializing an NBT Array and serialize it on one
        // go.

        // NOTES: This SeqSerializer still ends up writing the header...
        // somehow?? Can we use whatever type causes the byte buffer stuff to be
        // used in order to divert what gets called?

        match name {
            "__fastnbt_byte_array" => {
                self.writer.write_tag(Tag::ByteArray)?;
                self.writer.write_size_prefixed_str(&self.field)?;
                value.serialize(ArraySerializer {
                    ser: self,
                    tag: Tag::ByteArray,
                })
            }
            "__fastnbt_int_array" => {
                self.writer.write_tag(Tag::IntArray)?;
                self.writer.write_size_prefixed_str(&self.field)?;
                value.serialize(ArraySerializer {
                    ser: self,
                    tag: Tag::IntArray,
                })
            }
            "__fastnbt_long_array" => {
                self.writer.write_tag(Tag::LongArray)?;
                self.writer.write_size_prefixed_str(&self.field)?;
                value.serialize(ArraySerializer {
                    ser: self,
                    tag: Tag::LongArray,
                })
            }
            _ => todo!(),
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.writer.write_tag(Tag::List)?;
        self.writer.write_size_prefixed_str(&self.field)?;

        let len =
            len.ok_or_else(|| Error::bespoke("sequences must have a known length".to_string()))?;

        Ok(SerializerTuple {
            ser: self,
            state: SeqState::ListStart(len),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.writer.write_tag(Tag::List)?;
        self.writer.write_size_prefixed_str(&self.field)?;

        Ok(SerializerTuple {
            ser: self,
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

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.writer.write_tag(Tag::Compound)?;
        self.writer.write_size_prefixed_str(&self.field)?;

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

impl<'ser, 'a, W: Write> serde::ser::SerializeMap for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn end(self) -> Result<()> {
        self.writer.write_tag(Tag::End)
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: Serialize,
        V: Serialize,
    {
        // Get the name ahead of time.
        let mut name = Vec::new();
        key.serialize(&mut NameSerializer { name: &mut name })?;

        value.serialize(&mut Serializer {
            field: String::from_utf8(name).unwrap(), // FIXME nonunicode
            writer: &mut (*self).writer,
        })
    }
}

impl<'a, W: Write> serde::ser::SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<()> {
        self.writer.write_tag(Tag::End)?;
        Ok(())
    }
}

pub struct SerializerTuple<'a, W: Write> {
    pub(crate) ser: &'a mut Serializer<W>,
    pub(crate) state: SeqState,
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
        value.serialize(SeqSerializer {
            ser: self.ser,
            state: &mut self.state,
        })
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
