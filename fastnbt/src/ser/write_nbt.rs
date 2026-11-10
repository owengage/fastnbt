use std::borrow::Cow;
use std::convert::TryInto;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

use crate::error::{Error, Result};
use crate::Tag;

/// `s` in Java's modified UTF-8, which NBT strings are written in.
///
/// ASCII without NUL is the same in both encodings, and it is nearly every name and most
/// values of real NBT, so it is recognised with two passes that the standard library
/// vectorises rather than with the byte-by-byte scan of the general conversion.
pub(crate) fn to_java_cesu8(s: &str) -> Cow<'_, [u8]> {
    if s.is_ascii() && !s.as_bytes().contains(&0) {
        Cow::Borrowed(s.as_bytes())
    } else {
        cesu8::to_java_cesu8(s)
    }
}

pub(crate) trait WriteNbt: Write {
    fn write_tag(&mut self, tag: Tag) -> Result<()> {
        self.write_u8(tag as u8)?;
        Ok(())
    }

    fn write_size_prefixed_str(&mut self, key: &str) -> Result<()> {
        let key = to_java_cesu8(key);
        let len_bytes = key.len() as u16;
        self.write_u16::<BigEndian>(len_bytes)?;
        self.write_all(&key)?;
        Ok(())
    }

    fn write_len(&mut self, len: usize) -> Result<()> {
        self.write_i32::<BigEndian>(
            len.try_into()
                .map_err(|_| Error::bespoke("len too large".to_owned()))?,
        )?;

        Ok(())
    }
}

impl<T> WriteNbt for T where T: Write {}
