use std::io::Write;
use serde::ser::Impossible;
use serde::Serialize;
use crate::error::Error;
use super::{ArraySerializer, CompoundSerializer};

/// Serializer for compound values. This forwards everything except serialize_none to Serializer
/// The purpose is to be able to drop a field from the compound entirely if its None
pub(crate) struct CompoundValueSerializer<'a,'b: 'a, W> {
    pub(crate) ser: &'a mut CompoundSerializer<'b, W>,
    pub(crate) name: &'a[u8],
    pub(crate) is_first: bool
}

impl<'a,'b, W: Write> CompoundValueSerializer<'a,'b, W> {
    fn write_name(&mut self) -> Result<(), Error> {
        self.ser.serializer.writer.write_all(self.name)?;
        self.ser.serializer.writer.write_all(b":")?;
        Ok(())
    }
}

macro_rules! forward_serialisation {
    ($name:ident, $t:ty) => {
        fn $name(mut self, v: $t) -> Result<Self::Ok, Self::Error> {
            self.write_name()?;
            self.ser.serializer.$name(v)
        }
    };
}

impl<'a,'b, W: Write> serde::Serializer for CompoundValueSerializer<'a,'b, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = ArraySerializer<'a, W>;
    type SerializeTuple = ArraySerializer<'a, W>;
    type SerializeTupleStruct = ArraySerializer<'a, W>;
    type SerializeTupleVariant = ArraySerializer<'a, W>;
    type SerializeMap = CompoundSerializer<'a, W>;
    type SerializeStruct = CompoundSerializer<'a, W>;
    type SerializeStructVariant = Impossible<(), Error>;


    forward_serialisation!(serialize_bool, bool);
    forward_serialisation!(serialize_i8, i8);
    forward_serialisation!(serialize_i16, i16);
    forward_serialisation!(serialize_i32, i32);
    forward_serialisation!(serialize_i64, i64);
    forward_serialisation!(serialize_u8, u8);
    forward_serialisation!(serialize_u16, u16);
    forward_serialisation!(serialize_u32, u32);
    forward_serialisation!(serialize_u64, u64);
    forward_serialisation!(serialize_f32, f32);
    forward_serialisation!(serialize_f64, f64);
    forward_serialisation!(serialize_char, char);
    forward_serialisation!(serialize_str, &str);
    forward_serialisation!(serialize_bytes, &[u8]);

    /// Don't write the name and colon, if necessary reset has_first so no wrong comma is written
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if self.is_first {
            self.ser.has_first = false;
        }
        Ok(())
    }

    fn serialize_some<T>(mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.write_name()?;
        self.ser.serializer.serialize_some(value)
    }

    fn serialize_unit(mut self) -> Result<Self::Ok, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_unit()
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(mut self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T>(mut self, name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.write_name()?;
        self.ser.serializer.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T>(mut self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize
    {
        self.write_name()?;
        self.ser.serializer.serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_seq(len)
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_tuple(len)
    }

    fn serialize_tuple_struct(mut self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_map(len)
    }

    fn serialize_struct(mut self, name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_struct(name, len)
    }

    fn serialize_struct_variant(mut self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.write_name()?;
        self.ser.serializer.serialize_struct_variant(name, variant_index, variant, len)
    }
}