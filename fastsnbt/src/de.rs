use fastnbt::Tag;
use serde::{
    de::{self, Error as SerdeError},
    forward_to_deserialize_any,
};

use crate::{
    error::{Error, Result},
    input::{self, Input, Number, Reference},
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

    fn parse_number(&mut self) -> Result<Number> {
        self.input.parse_number()
    }
}

impl<'a> Deserializer<input::SliceInput<'a>> {
    /// Create Deserializer for a `T` from some sNBT string. See the [`de`] module
    /// for more information.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(data: &'a str) -> Self {
        Deserializer::new(input::SliceInput::new(data.as_bytes()))
    }
}

impl<'de, 'a, In> Deserializer<In>
where
    In: Input<'de>,
{
    fn parse_whitespace(&mut self) -> Result<Option<u8>> {
        loop {
            match self.input.peek()? {
                Some(b' ') | Some(b'\n') | Some(b'\t') | Some(b'\r') => {
                    self.input.discard();
                }
                other => {
                    return Ok(other);
                }
            }
        }
    }
}

impl<'de, 'a, In> de::Deserializer<'de> for &'a mut Deserializer<In>
where
    In: Input<'de>,
{
    type Error = Error;

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit unit_struct seq tuple tuple_struct
        identifier ignored_any bytes enum newtype_struct byte_buf option map struct
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.input.discard_whitespace()?;

        let peek = match self.input.peek()? {
            Some(c) => c,
            None => return Err(Error::unexpected_eof()),
        };

        let value = match peek {
            b'{' => {
                self.input.discard();

                let value = visitor.visit_map(MapAccess::new(self))?;

                let close_compound = match self.input.peek()? {
                    Some(c) => c,
                    None => return Err(Error::unexpected_eof()),
                };

                self.input.discard();

                Ok(value)
            }
            b'"' | b'\'' => {
                self.input.discard();
                let ident = self.input.parse_ident(peek, &mut self.scratch)?;
                let res = match ident {
                    Reference::Borrowed(ident) => visitor.visit_borrowed_str(ident),
                    Reference::Copied(ident) => visitor.visit_str(ident),
                };

                let close_quote = match self.input.peek()? {
                    Some(c) => c,
                    None => return Err(Error::unexpected_eof()),
                };

                if close_quote != peek {
                    // TODO: Better error message
                    return Err(Error::custom("mismatching quote"));
                }

                self.input.discard();
                res
            }
            b'0'..=b'9' | b'-' => {
                let num = self.parse_number()?;
                // TODO: Going to have to be a peek since it could be
                // nothing/comma etc, which is fine for an int (?)
                let peek = match self.input.peek()? {
                    Some(c) => c,
                    None => return Err(Error::unexpected_eof()),
                };
                match peek {
                    b'b' | b'B' => {
                        self.input.discard();
                        match num {
                            Number::Integer(n) => visitor.visit_i8(n as i8), // FIXME: Overflow
                            _ => return Err(Error::custom("expected byte")),
                        }
                    }
                    _ => return Err(Error::custom("unexpected number type")),
                }
            }
            _ => {
                // TODO: How does Minecraft handle unquoted field names?
                // Can't have spaces if unquoted.
                let ident = self.input.parse_ident(todo!(), &mut self.scratch)?;
                let res = match ident {
                    Reference::Borrowed(ident) => visitor.visit_borrowed_str(ident),
                    Reference::Copied(ident) => visitor.visit_str(ident),
                };

                res
            }
        };

        value
    }
}

struct MapAccess<'a, In: 'a> {
    de: &'a mut Deserializer<In>,
    first: bool,
}

impl<'a, In: 'a> MapAccess<'a, In> {
    pub fn new(de: &'a mut Deserializer<In>) -> Self {
        Self { de, first: true }
    }
}

impl<'de, 'a, In: Input<'de> + 'a> de::MapAccess<'de> for MapAccess<'a, In> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.de.input.discard_whitespace()?;

        if self.first {
            self.first = false;
        } else {
            let peek = match self.de.input.peek()? {
                Some(c) => c,
                None => return Err(Error::unexpected_eof()),
            };
            if peek == b'}' {
                return Ok(None);
            }
            if peek != b',' {
                return Err(Error::custom("expected ','"));
            }
            self.de.input.discard();
        }
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.de.input.discard_whitespace()?;

        let peek = match self.de.input.peek()? {
            Some(c) => c,
            None => return Err(Error::unexpected_eof()),
        };
        if peek != b':' {
            return Err(Error::custom("expected ':'"));
        }
        self.de.input.discard();

        self.de.input.discard_whitespace()?;

        seed.deserialize(&mut *self.de)
    }
}

// struct MapKey<'a, In> {
//     de: &'a mut Deserializer<In>,
// }
