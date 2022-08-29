use serde::{de, forward_to_deserialize_any};

use crate::{
    error::{Error, Result},
    input::{self, Input},
};

pub struct Deserializer<In> {
    input: In,
    scratch: Vec<u8>,
    seen_root: bool,
}

impl<'de, In> Deserializer<In>
where
    In: Input<'de>,
{
    pub fn new(input: In) -> Self {
        Self {
            input,
            scratch: Vec::new(),
            seen_root: false,
        }
    }
}

impl<'a> Deserializer<input::StrInput<'a>> {
    /// Create Deserializer for a `T` from some sNBT string. See the [`de`] module
    /// for more information.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(data: &'a str) -> Self {
        Deserializer::new(input::StrInput {
            data: data.as_bytes(),
        })
    }
}

impl<'de, 'a, In> de::Deserializer<'de> for &'a mut Deserializer<In>
where
    In: Input<'de>,
{
    type Error = Error;

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct
        identifier ignored_any bytes enum newtype_struct byte_buf option
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if !self.seen_root {
            // let peek = self.input.consume_tag()?;

            // match peek {
            //     // Tag::Compound => self.input.ignore_str()?,
            //     _ => return Err(Error::no_root_compound()),
            // }

            self.seen_root = true;
        }

        // visitor.visit_map(MapAccess::new(self))
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}
