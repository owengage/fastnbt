use std::io::Write;

use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use serde::ser::Impossible;

use crate::{error::Error, error::Result, Tag};

use super::{serializer::Serializer, write_nbt::WriteNbt};

/// ArraySerializer is for serializing the NBT Arrays ie ByteArray, IntArray and
/// LongArray.
pub(crate) struct ArraySerializer<'a, W: Write> {
    pub(crate) ser: &'a mut Serializer<W>,
    pub(crate) tag: Tag,
}

macro_rules! only_bytes {
    ($v:ident, $t:ty) => {
        fn $v(self, _: $t) -> Result<()> {
            Err(Error::array_as_other())
        }
    };
}

impl<'a, W: Write> serde::Serializer for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    only_bytes!(serialize_bool, bool);
    only_bytes!(serialize_i8, i8);
    only_bytes!(serialize_i16, i16);
    only_bytes!(serialize_i32, i32);
    only_bytes!(serialize_i64, i64);
    only_bytes!(serialize_i128, i128);
    only_bytes!(serialize_u8, u8);
    only_bytes!(serialize_u16, u16);
    only_bytes!(serialize_u32, u32);
    only_bytes!(serialize_u64, u64);
    only_bytes!(serialize_u128, u128);
    only_bytes!(serialize_f32, f32);
    only_bytes!(serialize_f64, f64);
    only_bytes!(serialize_char, char);
    only_bytes!(serialize_str, &str);
    only_bytes!(serialize_unit_struct, &'static str);

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let stride = match self.tag {
            Tag::ByteArray => 1,
            Tag::IntArray => 4,
            Tag::LongArray => 8,
            _ => panic!(),
        };
        let len = v.len() / stride;
        self.ser.writer.write_len(len)?;
        self.ser.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::array_as_other())
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error::array_as_other())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::array_as_other())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::array_as_other())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error::array_as_other())
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        Err(Error::array_as_other())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::array_as_other())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::array_as_other())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::array_as_other())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::array_as_other())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::array_as_other())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::array_as_other())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::array_as_other())
    }
}
