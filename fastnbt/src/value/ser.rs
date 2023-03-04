use core::result;
use std::collections::HashMap;

use serde::{ser::Impossible, serde_if_integer128, Serialize};

use crate::{
    error::{Error, Result},
    ByteArray, IntArray, LongArray, Tag, Value, BYTE_ARRAY_TOKEN, INT_ARRAY_TOKEN,
    LONG_ARRAY_TOKEN,
};

use super::array_serializer::ArraySerializer;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Byte(v) => serializer.serialize_i8(*v),
            Value::Short(v) => serializer.serialize_i16(*v),
            Value::Int(v) => serializer.serialize_i32(*v),
            Value::Long(v) => serializer.serialize_i64(*v),
            Value::Float(v) => serializer.serialize_f32(*v),
            Value::Double(v) => serializer.serialize_f64(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::ByteArray(v) => v.serialize(serializer),
            Value::IntArray(v) => v.serialize(serializer),
            Value::LongArray(v) => v.serialize(serializer),
            Value::List(v) => v.serialize(serializer),
            Value::Compound(v) => v.serialize(serializer),
        }
    }
}

//
// Everything below is copied and modified from serde_json:
// https://github.com/serde-rs/json/blob/52a9c050f5dcc0dc3de4825b131b8ff05219cc82/src/value/ser.rs
//
// For which the license is MIT:
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//

/// Serializer whose output is a `Value`.
///
/// This is the serializer that backs [`fastnbt::to_value`][crate::to_value].
/// Unlike the main fastnbt serializer which goes from some serializable
/// value of type `T` to NBT bytes, this one goes from `T` to
/// `fastnbt::Value`.
///
/// The `to_value` function is implementable as:
///
/// ```
/// use serde::Serialize;
/// use fastnbt::{error::Error, Value};
///
/// pub fn to_value<T>(input: T) -> Result<Value, Error>
/// where
///     T: Serialize,
/// {
///     input.serialize(&mut fastnbt::value::Serializer)
/// }
/// ```
pub struct Serializer;

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value> {
        Ok(Value::Byte(value as i8))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value> {
        Ok(Value::Byte(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value> {
        Ok(Value::Short(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value> {
        Ok(Value::Int(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Value> {
        Ok(Value::Long(value))
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Value> {
            let v = v as u128;
            Ok(Value::IntArray(IntArray::new(vec![
                (v >> 96) as i32,
                (v >> 64) as i32,
                (v >> 32) as i32,
                v as i32,
            ])))
        }

        fn serialize_u128(self, v: u128) -> Result<Value> {
            Ok(Value::IntArray(IntArray::new(vec![
                (v >> 96) as i32,
                (v >> 64) as i32,
                (v >> 32) as i32,
                v as i32,
            ])))
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value> {
        Ok(Value::Byte(value as i8))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value> {
        Ok(Value::Short(value as i16))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value> {
        Ok(Value::Int(value as i32))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value> {
        Ok(Value::Long(value as i64))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value> {
        Ok(Value::Float(value))
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Value> {
        Ok(Value::Double(value))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value> {
        Ok(Value::Int(value as i32))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value> {
        Ok(Value::List(
            value.iter().map(|byte| Value::Byte(*byte as i8)).collect(),
        ))
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        match variant {
            crate::BYTE_ARRAY_TOKEN => value.serialize(ArraySerializer {
                ser: self,
                tag: Tag::ByteArray,
            }),
            crate::INT_ARRAY_TOKEN => value.serialize(ArraySerializer {
                ser: self,
                tag: Tag::IntArray,
            }),
            crate::LONG_ARRAY_TOKEN => value.serialize(ArraySerializer {
                ser: self,
                tag: Tag::LongArray,
            }),
            _ => todo!("newtype variants that are not nbt arrays"),
        }
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            name: variant.into(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructVariant {
            name: variant.into(),
            map: HashMap::new(),
        })
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Value>
    where
        T: std::fmt::Display,
    {
        Ok(Value::String(value.to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        todo!()
    }
}

pub struct SerializeVec {
    vec: Vec<Value>,
}

pub struct SerializeTupleVariant {
    name: String,
    vec: Vec<Value>,
}

pub struct SerializeMap {
    map: HashMap<String, Value>,
    next_key: Option<String>,
}

pub struct SerializeStructVariant {
    name: String,
    map: HashMap<String, Value>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(crate::to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::List(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(crate::to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = HashMap::new();

        object.insert(self.name, Value::List(self.vec));

        Ok(Value::Compound(object))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");

        self.map.insert(key, crate::to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        if self.map.len() == 1 {
            let (key, val) = self.map.iter().next().unwrap();
            let data = || match val {
                Value::List(bs) => bs
                    .iter()
                    .map(|v| v.as_i64().unwrap() as u8)
                    .collect::<Vec<u8>>(),
                _ => panic!(),
            };

            Ok(match key.as_str() {
                BYTE_ARRAY_TOKEN => Value::ByteArray(ByteArray::from_bytes(&data())),
                INT_ARRAY_TOKEN => Value::IntArray(IntArray::from_bytes(&data())?),
                LONG_ARRAY_TOKEN => Value::LongArray(LongArray::from_bytes(&data())?),
                _ => Value::Compound(self.map),
            })
        } else {
            Ok(Value::Compound(self.map))
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> Error {
    Error::bespoke("Key must be a string".to_string())
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _value: bool) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, _value: f32) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<String> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String> {
        Ok(value.to_string())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<String>
    where
        T: std::fmt::Display,
    {
        Ok(value.to_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Value> {
        serde::ser::SerializeMap::end(self)
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(String::from(key), crate::to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = HashMap::new();

        object.insert(self.name, Value::Compound(self.map));

        Ok(Value::Compound(object))
    }
}
