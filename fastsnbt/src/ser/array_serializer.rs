use std::io::Write;

use byteorder::{ReadBytesExt, BigEndian};
use serde::ser::{Impossible, SerializeSeq};

use crate::{error::Error, error::Result};

use super::Serializer;

/// ArraySerializer is for serializing the NBT Arrays ie ByteArray, IntArray and
/// LongArray.
pub(crate) struct ArraySerializer<'a, W> {
    pub(crate) ser: &'a mut Serializer<W>,
    pub(crate) stride: usize,
    pub(crate) prefix: &'static str,
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
        let mut serializer = super::ArraySerializer::new(self.prefix, self.ser)?;
        match self.stride {
            1 => {
                let data = unsafe { &*(v as *const [u8] as *const [i8]) };
                data.iter().try_for_each(|i| SerializeSeq::serialize_element(&mut serializer, i))?
            }
            4 => v.chunks_exact(4).map(|mut bs| bs.read_i32::<BigEndian>())
                .try_for_each(|i| SerializeSeq::serialize_element(&mut serializer, &i?))?,
            8 => v.chunks_exact(8).map(|mut bs| bs.read_i64::<BigEndian>())
                .try_for_each(|l| SerializeSeq::serialize_element(&mut serializer, &l?))?,
            _ => panic!(),
        }
        SerializeSeq::end(serializer)
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
