use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::io::Read;
use std::str;

type Name = Option<String>;
use crate::nbt::Tag;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    ShortRead,
    InvalidTag(u8),
    InvalidName,
    EOF,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Tag(Tag),
    Name(&'a str),
    Byte(u8),
    Short(u16),
}

pub struct Tokenizer<'a> {
    input: &'a [u8],
    state: State,
}

pub enum State {
    ExpectTag,
    ExpectName(Tag),
    ExpectValue(Tag),
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            state: State::ExpectTag,
        }
    }

    pub fn next(&mut self) -> Result<Token> {
        match self.state {
            State::ExpectTag => {
                let tag = self.consume_tag()?;
                self.state = State::ExpectName(tag);
                Ok(Token::Tag(tag))
            }
            State::ExpectName(tag) => {
                self.state = State::ExpectValue(tag);
                self.consume_name()
            }
            State::ExpectValue(tag) => {
                self.state = State::ExpectTag;
                Ok(Token::Byte(self.consume_byte()?))
            }
        }
    }

    fn consume_tag(&mut self) -> Result<Tag> {
        let tag_byte = self.input.read_u8()?;
        u8_to_tag(tag_byte)
    }

    fn consume_name(&mut self) -> Result<Token> {
        self.consume_size_prefixed_string()
            .map(|name| Token::Name(name))
    }

    fn consume_byte(&mut self) -> Result<u8> {
        Ok(self.input.read_u8()?)
    }

    fn consume_size_prefixed_string(&mut self) -> Result<&'a str> {
        let len = self.input.read_u16::<BigEndian>()? as usize;
        let s = str::from_utf8(&self.input[..len]).map_err(|_| Error::InvalidName);
        self.input = &self.input[len..];
        s
    }
}

fn u8_to_tag(tag: u8) -> Result<Tag> {
    Tag::try_from(tag).or_else(|_| Err(Error::InvalidTag(tag)))
}

#[derive(Clone)]
enum Layer {
    List(Tag, usize),
    Compound,
}
