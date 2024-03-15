use std::io::Write;

use serde::{ser::Impossible, Serializer};

use crate::error::{Error, Result};

pub(crate) struct NameSerializer<W: Write> {
    pub(crate) name: W,
}

macro_rules! bespoke_error {
    ($name:literal) => {
        Err(Error::bespoke(format!(
            "field must be string-like, found {}",
            $name
        )))
    };
}

macro_rules! must_be_stringy {
    ($name:literal: $ser:ident($($t:ty),*) -> $res:ty) => {
        fn $ser(self, $(_: $t),*) -> Result<$res> {
            bespoke_error!($name)
        }
    };

    ($name:literal: $ser:ident<T>($($t:ty),*) -> $res:ty) => {
        fn $ser<T: ?Sized>(self, $(_: $t),*) -> Result<$res> {
            bespoke_error!($name)
        }
    };

    ($name:literal: $ser:ident($($t:ty),*)) => {
        must_be_stringy!($name: $ser($($t),*) -> ());
    };

    ($name:literal: $ser:ident<T>($($t:ty),*)) => {
        must_be_stringy!($name: $ser<T>($($t),*) -> ());
    };
}

/// NameSerializer is all about serializing the name of a field. It does not
/// write the length or the tag. We typically need to write this to a different
/// buffer than the main one we're writing to, because we need to write out the
/// field in tag, name, value order. In order the write the tag we need to know
/// what value we're serializing, which we can only do when we serialize the
/// value. So we save the name start serializing the value, which serializes the
/// tag, then the saved name, then the value.
impl<W: Write> Serializer for &mut NameSerializer<W> {
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
        self.name.write_all(v)?;
        Ok(())
    }

    fn serialize_char(self, c: char) -> Result<Self::Ok> {
        self.name.write_all(&cesu8::to_java_cesu8(&c.to_string()))?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.name.write_all(&cesu8::to_java_cesu8(v))?;
        Ok(())
    }

    must_be_stringy!("bool": serialize_bool(bool));
    must_be_stringy!("i8": serialize_i8(i8));
    must_be_stringy!("i16": serialize_i16(i16));
    must_be_stringy!("i32": serialize_i32(i32));
    must_be_stringy!("i64": serialize_i64(i64));
    must_be_stringy!("u8": serialize_u8(u8));
    must_be_stringy!("u16": serialize_u16(u16));
    must_be_stringy!("u32": serialize_u32(u32));
    must_be_stringy!("u64": serialize_u64(u64));
    must_be_stringy!("f32": serialize_f32(f32));
    must_be_stringy!("f64": serialize_f64(f64));
    must_be_stringy!("none": serialize_none());
    must_be_stringy!("some": serialize_some<T>(&T));
    must_be_stringy!("unit": serialize_unit());
    must_be_stringy!("unit_struct": serialize_unit_struct(&'static str));
    must_be_stringy!("unit_variant": serialize_unit_variant(&'static str, u32, &'static str));
    must_be_stringy!("newtype_struct": serialize_newtype_struct<T>(&'static str, &T));
    must_be_stringy!("newtype_variant": serialize_newtype_variant<T>(&'static str, u32, &'static str, &T));
    must_be_stringy!("seq": serialize_seq(Option<usize>) -> Self::SerializeSeq);
    must_be_stringy!("tuple": serialize_tuple(usize) -> Self::SerializeTuple);
    must_be_stringy!("tuple_struct": serialize_tuple_struct(&'static str, usize) -> Self::SerializeTupleStruct);
    must_be_stringy!("tuple_variant": serialize_tuple_variant(&'static str, u32, &'static str, usize) -> Self::SerializeTupleVariant);
    must_be_stringy!("map": serialize_map(Option<usize>) -> Self::SerializeMap);
    must_be_stringy!("struct": serialize_struct(&'static str, usize) -> Self::SerializeStruct);
    must_be_stringy!("struct_variant": serialize_struct_variant(&'static str, u32, &'static str, usize) -> Self::SerializeStructVariant);
}
