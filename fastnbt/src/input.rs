use std::{borrow::Cow, io::Read, ops::Range};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::{Error, Result},
    Tag,
};

mod private {
    // Only this crate can implement this trait. Other traits can inherit from
    // Sealed in order to prevent other crates from creating implementations.
    pub trait Sealed {}
}

fn try_size(size: i32, multiplier: usize) -> Result<usize> {
    let size: usize = size
        .try_into()
        .map_err(|_| Error::bespoke("size was negative".to_string()))?;

    size.checked_mul(multiplier)
        .ok_or_else(|| Error::bespoke("size too large".to_string()))
}
pub enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c> AsRef<[u8]> for Reference<'b, 'c, [u8]> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Reference::Borrowed(bs) => bs,
            Reference::Copied(bs) => bs,
        }
    }
}

pub trait Input<'de>: private::Sealed {
    #[doc(hidden)]
    fn consume_byte(&mut self) -> Result<u8>;

    // #[doc(hidden)]
    // fn discard(&mut self);

    #[doc(hidden)]
    fn ignore_str(&mut self) -> Result<()>;

    #[doc(hidden)]
    fn ignore_bytes(&mut self, size: usize) -> Result<()>;

    fn consume_tag(&mut self) -> Result<Tag> {
        let tag = self.consume_byte()?;
        Tag::try_from(tag).map_err(|_| Error::invalid_tag(tag))
    }

    fn consume_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>>;

    fn consume_bytes<'s>(
        &'s mut self,
        n: usize,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>>;

    fn consume_i16(&mut self) -> Result<i16>;
    fn consume_i32(&mut self) -> Result<i32>;
    fn consume_i64(&mut self) -> Result<i64>;
    fn consume_f32(&mut self) -> Result<f32>;
    fn consume_f64(&mut self) -> Result<f64>;

    fn ignore_value(&mut self, tag: Tag) -> Result<()> {
        match tag {
            Tag::Byte => {
                self.consume_byte()?;
            }
            Tag::Short => {
                self.consume_i16()?;
            }
            Tag::Int => {
                self.consume_i32()?;
            }
            Tag::Long => {
                self.consume_i64()?;
            }
            Tag::Float => {
                self.consume_f32()?;
            }
            Tag::Double => {
                self.consume_f64()?;
            }
            Tag::String => {
                self.ignore_str()?;
            }
            Tag::ByteArray => {
                let size = self.consume_i32()? as usize;
                self.ignore_bytes(size)?;
            }
            Tag::IntArray => {
                let size = self.consume_i32()?;
                self.ignore_bytes(try_size(size, std::mem::size_of::<i32>())?)?;
            }
            Tag::LongArray => {
                let size = self.consume_i32()?;
                self.ignore_bytes(try_size(size, std::mem::size_of::<i64>())?)?;
            }
            Tag::Compound => {
                // Need to loop and ignore each value until we reach an end tag.

                // we need to enter the compound, then ignore it's value.
                loop {
                    let tag = self.consume_tag()?;
                    if tag == Tag::End {
                        break;
                    }

                    // consume the name.
                    self.ignore_str()?;
                    self.ignore_value(tag)?;
                }
            }
            Tag::List => {
                let element_tag = self.consume_tag()?;
                let size = self.consume_i32()?;
                for _ in 0..size {
                    self.ignore_value(element_tag)?;
                }
            }
            Tag::End => {
                // If we are trying to ignore a list of empty compounds, that
                // list might be indicated by a series of End tags. If this
                // occurs then we should end the Compound branch of this match
                // statement, where the end tag will be consumed. So we should
                // never reach here.
                //
                // TODO: Write an explicit test for ignored list of compound.
                unreachable!()
            }
        }

        Ok(())
    }
}

pub struct Slice<'de> {
    pub(crate) data: &'de [u8],
}

impl<'de> private::Sealed for Slice<'de> {}
impl<'de> Slice<'de> {
    fn consume(&mut self, r: Range<usize>) -> Result<&'de [u8]> {
        if r.end <= self.data.len() {
            let ret = &self.data[r.start..r.end];
            self.data = &self.data[r.end..];
            Ok(ret)
        } else {
            Err(Error::unexpected_eof())
        }
    }
}

impl<'de> Input<'de> for Slice<'de> {
    fn consume_byte(&mut self) -> Result<u8> {
        Ok(self.consume(0..1)?[0])
    }

    fn ignore_str(&mut self) -> Result<()> {
        let len = self.consume(0..2)?.read_u16::<BigEndian>()? as usize;
        self.consume(0..len).map(|_| ())
    }

    fn consume_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>> {
        let len = self.consume(0..2)?.read_u16::<BigEndian>()? as usize;
        let str = self.consume(0..len)?;
        let str = cesu8::from_java_cesu8(str).map_err(|_| Error::nonunicode_string(str))?;

        Ok(match str {
            Cow::Borrowed(str) => Reference::Borrowed(str),
            Cow::Owned(str) => {
                *scratch = str.into_bytes();
                // we just converted scratch into the bytes of a string, so it
                // definitely utf8.
                Reference::Copied(unsafe { std::str::from_utf8_unchecked(scratch) })
            }
        })
    }

    fn consume_bytes<'s>(
        &'s mut self,
        n: usize,
        _scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        let bs = self.consume(0..n)?;
        Ok(Reference::Borrowed(bs))
    }

    fn consume_i16(&mut self) -> Result<i16> {
        let mut bs = self.consume(0..std::mem::size_of::<i16>())?;
        Ok(bs.read_i16::<BigEndian>()?)
    }

    fn consume_i32(&mut self) -> Result<i32> {
        let mut bs = self.consume(0..std::mem::size_of::<i32>())?;
        Ok(bs.read_i32::<BigEndian>()?)
    }

    fn consume_i64(&mut self) -> Result<i64> {
        let mut bs = self.consume(0..std::mem::size_of::<i64>())?;
        Ok(bs.read_i64::<BigEndian>()?)
    }

    fn consume_f32(&mut self) -> Result<f32> {
        let mut bs = self.consume(0..std::mem::size_of::<f32>())?;
        Ok(bs.read_f32::<BigEndian>()?)
    }

    fn consume_f64(&mut self) -> Result<f64> {
        let mut bs = self.consume(0..std::mem::size_of::<f64>())?;
        Ok(bs.read_f64::<BigEndian>()?)
    }

    fn ignore_bytes(&mut self, size: usize) -> Result<()> {
        self.consume(0..size)?;
        Ok(())
    }
}

pub struct Reader<R: Read> {
    pub(crate) reader: R,
}

impl<R: Read> private::Sealed for Reader<R> {}

impl<'de, R: Read> Input<'de> for Reader<R> {
    fn consume_byte(&mut self) -> Result<u8> {
        Ok(self.reader.read_u8()?)
    }

    fn ignore_str(&mut self) -> Result<()> {
        let len = self.reader.read_u16::<BigEndian>()? as usize;
        let mut buf = vec![0; len]; // TODO: try a scratch space to reduce allocs?
        Ok(self.reader.read_exact(&mut buf)?)
    }

    fn consume_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>> {
        let len = self.reader.read_u16::<BigEndian>()? as usize;
        scratch.clear();
        scratch.resize(len, 0);
        self.reader.read_exact(scratch)?;

        let str = cesu8::from_java_cesu8(scratch).map_err(|_| Error::nonunicode_string(scratch))?;

        Ok(match str {
            Cow::Borrowed(_) => {
                Reference::Copied(unsafe { std::str::from_utf8_unchecked(scratch) })
            }
            Cow::Owned(s) => {
                *scratch = s.into_bytes();
                Reference::Copied(unsafe { std::str::from_utf8_unchecked(scratch) })
            }
        })
    }

    fn consume_bytes<'s>(
        &'s mut self,
        n: usize,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        scratch.clear();
        scratch.resize(n, 0);
        self.reader.read_exact(scratch.as_mut_slice())?;

        Ok(Reference::Copied(scratch.as_slice()))
    }

    fn consume_i16(&mut self) -> Result<i16> {
        Ok(self.reader.read_i16::<BigEndian>()?)
    }

    fn consume_i32(&mut self) -> Result<i32> {
        Ok(self.reader.read_i32::<BigEndian>()?)
    }

    fn consume_i64(&mut self) -> Result<i64> {
        Ok(self.reader.read_i64::<BigEndian>()?)
    }

    fn consume_f32(&mut self) -> Result<f32> {
        Ok(self.reader.read_f32::<BigEndian>()?)
    }

    fn consume_f64(&mut self) -> Result<f64> {
        Ok(self.reader.read_f64::<BigEndian>()?)
    }

    fn ignore_bytes(&mut self, size: usize) -> Result<()> {
        let mut buf = vec![0; size]; // TODO: try a scratch space to reduce allocs?
        self.reader.read_exact(&mut buf)?;
        Ok(())
    }
}
