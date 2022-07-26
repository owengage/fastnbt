use serde::de;
use serde::de::value::BorrowedBytesDeserializer;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::value::BytesDeserializer;

use crate::de2::Deserializer;
use crate::error::{Error, Result};
use crate::BYTE_ARRAY_TOKEN;
use crate::INT_ARRAY_TOKEN;
use crate::LONG_ARRAY_TOKEN;

use super::input::Input;
use super::input::Reference;

enum State {
    Unread,
    Read,
}

pub(crate) struct ArrayWrapperAccess<'a, In: 'a> {
    de: &'a mut Deserializer<In>,
    token: &'static str,
    bytes_size: usize,
    state: State,
}

impl<'a, In: 'a> ArrayWrapperAccess<'a, In> {
    pub(crate) fn bytes(de: &'a mut Deserializer<In>, size: usize) -> Result<Self> {
        Ok(Self {
            de,
            bytes_size: size
                .checked_mul(1)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?,
            token: BYTE_ARRAY_TOKEN,
            state: State::Unread,
        })
    }

    pub(crate) fn ints(de: &'a mut Deserializer<In>, size: usize) -> Result<Self> {
        Ok(Self {
            de,
            bytes_size: size
                .checked_mul(4)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?,
            token: INT_ARRAY_TOKEN,
            state: State::Unread,
        })
    }

    pub(crate) fn longs(de: &'a mut Deserializer<In>, size: usize) -> Result<Self> {
        Ok(Self {
            de,
            bytes_size: size
                .checked_mul(8)
                .ok_or_else(|| Error::bespoke("nbt array too large".to_string()))?,
            token: LONG_ARRAY_TOKEN,
            state: State::Unread,
        })
    }
}

impl<'de, 'a, In: Input<'de> + 'a> de::MapAccess<'de> for ArrayWrapperAccess<'a, In> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let State::Unread = self.state {
            self.state = State::Read;
            seed.deserialize(BorrowedStrDeserializer::new(self.token))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        let data = self
            .de
            .input
            .consume_bytes(self.bytes_size, &mut self.de.scratch)?;

        match data {
            Reference::Borrowed(bs) => seed.deserialize(BorrowedBytesDeserializer::new(bs)),
            Reference::Copied(bs) => seed.deserialize(BytesDeserializer::new(bs)),
        }
    }
}
