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

pub enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

pub trait Input<'de>: private::Sealed {
    #[doc(hidden)]
    fn consume_byte(&mut self) -> Result<u8>;

    // #[doc(hidden)]
    // fn discard(&mut self);

    #[doc(hidden)]
    fn ignore_str(&mut self) -> Result<()>;

    fn consume_tag(&mut self) -> Result<Tag> {
        let tag = self.consume_byte()?;
        Tag::try_from(tag).map_err(|_| Error::invalid_tag(tag))
    }

    fn consume_str<'s>(&'s mut self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>>;
}

pub(crate) struct Slice<'de> {
    pub data: &'de [u8],
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
}

// fn consume_size_prefixed_string(&mut self) -> Result<Cow<'de, str>> {
//     let len = self.0.read_u16::<BigEndian>()? as usize;
//     let str_data = self.subslice(0..len)?;
//     let s =
//         cesu8::from_java_cesu8(str_data).map_err(|_| Error::nonunicode_string(&self.0[..len]))?;

//     self.0 = &self.0[len..];
//     Ok(s)
// }

pub(crate) struct Reader<R: Read> {
    pub reader: R,
}

impl<'de, R: Read> private::Sealed for Reader<R> {}

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
}
