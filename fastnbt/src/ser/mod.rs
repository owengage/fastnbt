mod anon_serializer;
mod array_serializer;
mod name_serializer;
mod seq_serializer;
mod serializer;
mod write_nbt;

use anon_serializer::*;
use array_serializer::*;
use name_serializer::*;
use seq_serializer::*;
use serializer::*;
use write_nbt::*;

use std::{io::Write, vec};

use serde::{ser, Serialize};

use crate::{
    error::{Error, Result},
    Tag,
};

pub fn to_bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut result = vec![];
    let mut serializer = Serializer {
        writer: &mut result,
        field: String::new(),
    };
    v.serialize(&mut serializer)?;
    Ok(result)
}

pub struct SerializerTuple<'a, W: 'a + Write> {
    ser: &'a mut AnonSerializer<W>,
}

impl<'ser, 'a, W: Write> serde::ser::SerializeSeq for &'a mut AnonSerializer<W> {
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

impl<'ser, 'a, W: Write> serde::ser::SerializeTupleStruct for &'a mut AnonSerializer<W> {
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

impl<'ser, 'a, W: Write> serde::ser::SerializeTupleVariant for &'a mut AnonSerializer<W> {
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

impl<'ser, 'a, W: Write> serde::ser::SerializeStructVariant for &'a mut AnonSerializer<W> {
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
