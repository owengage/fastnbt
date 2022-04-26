use std::collections::HashMap;

use serde::{
    de::{DeserializeSeed, Visitor},
    Deserialize,
};

use crate::{ByteArray, IntArray, LongArray, Value};

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("valid NBT")
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Byte(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Short(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Int(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Long(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Double(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.to_owned()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // I think I either need to do the same trick as I did for NBT
                // Arrays and have dedicated types to this, or just accept that
                // it's a Vec<Value> and that you can construct invalid NBT with
                // it (by adding values of different type).

                // Feel like the case of list of lists will break me if I try
                // the dedicated type.
                let mut v = Vec::<Value>::with_capacity(seq.size_hint().unwrap_or(0));

                while let Some(el) = seq.next_element()? {
                    v.push(el);
                }

                Ok(Value::List(v))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                match map.next_key_seed(KeyClassifier)? {
                    Some(KeyClass::Compound(first_key)) => {
                        let mut compound = HashMap::new();

                        compound.insert(first_key, map.next_value()?);
                        while let Some((key, value)) = map.next_entry()? {
                            compound.insert(key, value);
                        }

                        Ok(Value::Compound(compound))
                    }
                    Some(KeyClass::ByteArray) => {
                        let data = map.next_value::<&[u8]>()?;
                        Ok(Value::ByteArray(ByteArray::from_bytes(data)))
                    }
                    Some(KeyClass::IntArray) => {
                        let data = map.next_value::<&[u8]>()?;
                        IntArray::from_bytes(data)
                            .map(Value::IntArray)
                            .map_err(|_| serde::de::Error::custom("could not read int array"))
                    }
                    Some(KeyClass::LongArray) => {
                        let data = map.next_value::<&[u8]>()?;
                        LongArray::from_bytes(data)
                            .map(Value::LongArray)
                            .map_err(|_| serde::de::Error::custom("could not read long array"))
                    }
                    // No keys just means an empty compound.
                    None => Ok(Value::Compound(Default::default())),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

struct KeyClassifier;

enum KeyClass {
    Compound(String),
    ByteArray,
    IntArray,
    LongArray,
}

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, deserializer: D) -> Result<KeyClass, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an nbt field string")
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match s.as_str() {
            crate::BYTE_ARRAY_TOKEN => Ok(KeyClass::ByteArray),
            crate::INT_ARRAY_TOKEN => Ok(KeyClass::IntArray),
            crate::LONG_ARRAY_TOKEN => Ok(KeyClass::LongArray),
            _ => Ok(KeyClass::Compound(s)),
        }
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match s {
            crate::BYTE_ARRAY_TOKEN => Ok(KeyClass::ByteArray),
            crate::INT_ARRAY_TOKEN => Ok(KeyClass::IntArray),
            crate::LONG_ARRAY_TOKEN => Ok(KeyClass::LongArray),
            _ => Ok(KeyClass::Compound(s.to_string())),
        }
    }
}
