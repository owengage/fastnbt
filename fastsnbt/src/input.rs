use serde::de::Error as SerdeError;

use crate::error::{Error, Result};

mod private {
    // Only this crate can implement this trait. Other traits can inherit from
    // Sealed in order to prevent other crates from creating implementations.
    pub trait Sealed {}
}

pub enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

pub enum Number {
    Float(f64),
    Integer(i64),
}

pub trait Input<'de>: private::Sealed {
    #[doc(hidden)]
    fn next(&mut self) -> Result<Option<u8>>;
    #[doc(hidden)]
    fn peek(&mut self) -> Result<Option<u8>>;

    #[doc(hidden)]
    fn discard(&mut self);

    fn discard_whitespace(&mut self) -> Result<()>;

    fn parse_ident<'scratch>(
        &mut self,
        quote: u8,
        scratch: &'scratch mut Vec<u8>,
    ) -> Result<Reference<'de, 'scratch, str>>;

    fn parse_number<'scratch>(&mut self) -> Result<Number>;
}

pub(crate) struct SliceInput<'de> {
    data: &'de [u8],
    index: usize,
}

impl<'de> SliceInput<'de> {
    pub fn new(data: &'de [u8]) -> Self {
        Self { data, index: 0 }
    }
}

impl private::Sealed for SliceInput<'_> {}

impl<'de> Input<'de> for SliceInput<'de> {
    fn next(&mut self) -> Result<Option<u8>> {
        let b = self.peek()?;
        self.discard();
        Ok(b)
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        Ok(self.data.get(self.index).cloned())
    }

    fn discard(&mut self) {
        self.index += 1;
    }

    fn parse_ident<'scratch>(
        &mut self,
        quote: u8,
        scratch: &'scratch mut Vec<u8>,
    ) -> Result<Reference<'de, 'scratch, str>> {
        scratch.clear();

        // TODO: Could return borrowed.
        loop {
            let ch = self.data.get(self.index);
            match ch {
                Some(ch) if *ch == quote => {
                    let s = std::str::from_utf8(&*scratch)
                        .map_err(|_| Error::nonunicode_string(&[]))?;
                    return Ok(Reference::Copied(s));
                }
                Some(ch) => {
                    scratch.push(*ch);
                }
                None => return Err(Error::unexpected_eof()),
            }
            self.index += 1;
        }
    }

    fn parse_number<'scratch>(&mut self) -> Result<Number> {
        let start = self.index;
        let mut is_float = false;

        loop {
            let ch = self.data.get(self.index);
            match ch {
                Some(ch) => {
                    // TODO negative numbers...
                    if !ch.is_ascii_digit() && *ch != b'.' {
                        // Now got all the number.
                        let s = &self.data[start..self.index];
                        let s = std::str::from_utf8(s).map_err(|_| Error::nonunicode_string(s))?;

                        return Ok(Number::Integer(
                            s.parse().map_err(|_| Error::custom("not a number"))?,
                        ));
                    }
                    if *ch == b'.' {
                        is_float = true;
                    }
                }
                None => return Err(Error::unexpected_eof()),
            }
            self.index += 1;
        }
    }

    fn discard_whitespace(&mut self) -> Result<()> {
        loop {
            let ch = self.data.get(self.index);
            match ch {
                Some(ch) if !ch.is_ascii_whitespace() => {
                    return Ok(());
                }
                Some(_) => {}
                None => return Err(Error::unexpected_eof()),
            };

            self.index += 1;
        }
    }
}
