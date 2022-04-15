use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{
    ser::{self, Impossible, SerializeTuple},
    Serialize,
};

use crate::{
    error::{Error, Result},
    Tag,
};

use super::{
    array_serializer::ArraySerializer, name_serializer::NameSerializer, write_nbt::WriteNbt,
};

#[derive(Debug)]
pub(crate) enum State {
    ListStart { len: usize },
    ListRest,
    Compound { current_field: String },
}

#[derive(Debug)]
pub(crate) enum TupleState {
    Start { len: usize },
    Rest,
}

pub struct Serializer<W: Write> {
    pub(crate) writer: W,
    pub(crate) state: State,
}

impl<'a, W: Write> Serializer<W> {
    fn try_write_header(&mut self, tag: Tag) -> Result<()> {
        match &mut self.state {
            State::ListStart { len } => {
                self.writer.write_tag(tag)?;
                self.writer.write_len(*len)?;
                self.state = State::ListRest;
            }
            State::ListRest => {}
            State::Compound { current_field } => {
                self.writer.write_tag(tag)?;
                self.writer.write_size_prefixed_str(current_field)?;
            }
        }
        Ok(())
    }
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
        self.try_write_header(Tag::Byte)?;
        self.writer.write_u8(v as u8)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.try_write_header(Tag::Byte)?;
        self.writer.write_i8(v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.try_write_header(Tag::Short)?;
        self.writer.write_i16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.try_write_header(Tag::Int)?;
        self.writer.write_i32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.try_write_header(Tag::Long)?;
        self.writer.write_i64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.try_write_header(Tag::Byte)?;
        self.writer.write_u8(v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.try_write_header(Tag::Short)?;
        self.writer.write_u16::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.try_write_header(Tag::Int)?;
        self.writer.write_u32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.try_write_header(Tag::Long)?;
        self.writer.write_u64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.try_write_header(Tag::Float)?;
        self.writer.write_f32::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.try_write_header(Tag::Double)?;
        self.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.try_write_header(Tag::Int)?;
        self.writer.write_u32::<BigEndian>(v as u32)?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.try_write_header(Tag::String)?;
        self.writer.write_size_prefixed_str(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.try_write_header(Tag::List)?;
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
        self.try_write_header(Tag::String)?;
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
        match variant {
            "__fastnbt_byte_array" => {
                self.try_write_header(Tag::ByteArray)?;
                value.serialize(ArraySerializer {
                    ser: self,
                    tag: Tag::ByteArray,
                })
            }
            "__fastnbt_int_array" => {
                self.try_write_header(Tag::IntArray)?;
                value.serialize(ArraySerializer {
                    ser: self,
                    tag: Tag::IntArray,
                })
            }
            "__fastnbt_long_array" => {
                self.try_write_header(Tag::LongArray)?;
                value.serialize(ArraySerializer {
                    ser: self,
                    tag: Tag::LongArray,
                })
            }
            _ => todo!("newtype variants that are not nbt arrays"),
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len =
            len.ok_or_else(|| Error::bespoke("sequences must have a known length".to_string()))?;

        self.serialize_tuple(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.try_write_header(Tag::List)?;

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
            state: TupleState::Start { len },
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
        self.try_write_header(Tag::Compound)?;
        Ok(SerializerMap { ser: self })
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

        self.ser.state = State::Compound {
            current_field: cesu8::from_java_cesu8(&name)
                .map_err(|_| Error::bespoke("field name was invalid cesu8".to_string()))?
                .to_string(),
        };
        value.serialize(&mut *self.ser)
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
    pub(crate) state: TupleState,
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
        match self.state {
            TupleState::Start { len } => {
                self.ser.state = State::ListStart { len };
                self.state = TupleState::Rest;
                value.serialize(&mut *self.ser)
            }
            TupleState::Rest => {
                self.ser.state = State::ListRest;
                value.serialize(&mut *self.ser)
            }
        }
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
