use byteorder::{NativeEndian, ReadBytesExt};
use serde::ser::Impossible;

use crate::{error::Error, ByteArray, IntArray, LongArray, Tag, Value};

use super::ser::Serializer;

/// ArraySerializer is for serializing the NBT Arrays ie ByteArray, IntArray and
/// LongArray.
pub struct ArraySerializer<'a> {
    pub ser: &'a mut Serializer,
    pub tag: Tag,
}

impl<'a> serde::Serializer for ArraySerializer<'a> {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        match self.tag {
            Tag::ByteArray => Ok(Value::ByteArray(ByteArray::from_bytes(v))),
            Tag::IntArray => Ok(Value::IntArray(IntArray::new(
                v.chunks_exact(4)
                    .map(|mut bs| bs.read_i32::<NativeEndian>())
                    .collect::<std::io::Result<Vec<i32>>>()?,
            ))),
            Tag::LongArray => Ok(Value::LongArray(LongArray::new(
                v.chunks_exact(8)
                    .map(|mut bs| bs.read_i64::<NativeEndian>())
                    .collect::<std::io::Result<Vec<i64>>>()?,
            ))),
            _ => unreachable!(),
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unimplemented!()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!()
    }
}
