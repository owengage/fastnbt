// use serde::Deserialize;

// use serde::forward_to_deserialize_any;

// use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
// use std::io::Read;

// use crate::nbt::error::{Error, Result};

// use super::{Parser, Value};

// pub struct Deserializer<R: Read> {
//     parser: Parser<R>,
// }

// impl<'de, R: Read> Deserializer<R> {
//     pub fn from_reader(r: R) -> Self {
//         Deserializer {
//             parser: Parser::new(r),
//         }
//     }
// }

// pub fn from_reader<'a, R: Read, T>(r: R) -> Result<T>
// where
//     T: Deserialize<'a>,
// {
//     let mut deserializer = Deserializer::from_reader(r);
//     T::deserialize(&mut deserializer)
// }

// impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
//     type Error = Error;

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         Err(Error::Message("unimpl".to_owned()))
//     }

//     fn deserialize_struct<V>(
//         self,
//         name: &'static str,
//         fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         let v = self.parser.next().unwrap();

//         match v {
//             Value::Compound(_) => Err(Error::Message("HEY".to_owned())), // a map accessor?
//             _ => Err(Error::Message("expected compound".to_owned())),
//         }
//     }

//     forward_to_deserialize_any! {
//         bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
//         bytes byte_buf option unit unit_struct newtype_struct seq tuple
//         tuple_struct map enum identifier ignored_any
//     }
// }

// struct CompoundAccessor<R: Read> {
//     parser: Parser<R>,
//     current: Option<Value>,
// }

// // impl<'de, R: Read> MapAccess<'de> for CompoundAccessor<R> {
// //     type Error = Error;

// //     fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
// //     where
// //         K: DeserializeSeed<'de>,
// //     {
// //         self.current = self.current.or_else(|| self.parser.next().ok());
// //         match self.current.ok_or(Error::Message("soemthig".to_owned())) {
// //             _ => Err(Error::Message("".to_owned())),
// //         }
// //     }

// //     fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
// //     where
// //         V: DeserializeSeed<'de>,
// //     {
// //     }

// //     fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>>
// //     where
// //         K: DeserializeSeed<'de>,
// //         V: DeserializeSeed<'de>,
// //     {
// //         self.current = self.current.or_else(|| self.parser.next().ok());
// //         match self.current {
// //             Some(Value::Byte(name, b)) => Ok((name, b)),
// //             _ => Err(Error::Message("compound contained unsupported type")),
// //         }
// //     }
// // }
