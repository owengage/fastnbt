use std::io::Write;

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
    ($v:ident($($t:ty),* $(,)?) -> $r:ty) => {
        fn $v(self, $(_: $t),*) -> Result<$r> {
            Err(Error::array_as_other())
        }
    };

    ($v:ident<T>($($t:ty),* $(,)?)) => {
        fn $v<T: ?Sized>(self, $(_: $t),*) -> Result<Self::Ok> {
            Err(Error::array_as_other())
        }
    };

    ($v:ident($($t:ty),* $(,)?)) => {
        only_bytes!{$v($($t,)*) -> ()}
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

    only_bytes!(serialize_bool(bool));
    only_bytes!(serialize_i8(i8));
    only_bytes!(serialize_i16(i16));
    only_bytes!(serialize_i32(i32));
    only_bytes!(serialize_i64(i64));
    only_bytes!(serialize_i128(i128));
    only_bytes!(serialize_u8(u8));
    only_bytes!(serialize_u16(u16));
    only_bytes!(serialize_u32(u32));
    only_bytes!(serialize_u64(u64));
    only_bytes!(serialize_u128(u128));
    only_bytes!(serialize_f32(f32));
    only_bytes!(serialize_f64(f64));
    only_bytes!(serialize_char(char));
    only_bytes!(serialize_str(&str));
    only_bytes!(serialize_none());
    only_bytes!(serialize_some<T>(&T));
    only_bytes!(serialize_unit());
    only_bytes!(serialize_unit_struct(&'static str));
    only_bytes!(serialize_unit_variant(&'static str, u32, &'static str));
    only_bytes!(serialize_newtype_struct<T>(&'static str, &T));
    only_bytes!(serialize_newtype_variant<T>(&'static str, u32, &'static str, &T));
    only_bytes!(serialize_seq(Option<usize>) -> Self::SerializeSeq);
    only_bytes!(serialize_tuple(usize) -> Self::SerializeSeq);
    only_bytes!(serialize_map(Option<usize>) -> Self::SerializeMap);
    only_bytes!(serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct);
    only_bytes!(serialize_struct(&'static str, usize) -> Self::SerializeStruct);

    only_bytes!(
        serialize_tuple_variant(
            &'static str,
            u32,
            &'static str,
            usize,
        ) -> Self::SerializeTupleVariant
    );

    only_bytes!(
        serialize_struct_variant(
            &'static str,
            u32,
            &'static str,
            usize,
        ) -> Self::SerializeStructVariant
    );
}
