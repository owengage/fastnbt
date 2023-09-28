//! Allows streaming of NBT data without prior knowledge of the structure.

use super::Tag;
use byteorder::{BigEndian, ReadBytesExt};
use std::{convert::TryFrom, io::Read, str};

/// An optional `String`.
pub type Name = Option<String>;

/// A shallow NBT value.
///
/// For every value except compounds and lists, this contains the value of the tag. For example, a `Value::Byte` will
/// contain the name and the byte of that NBT tag.
///
/// The name part of each variant is optional, since elements in an NBT list do not have names. The end of lists do not
/// have a name in the binary format, so it isn't included here either.
///
/// See `Parser` for more information.
#[derive(Debug, PartialEq)]
pub enum Value {
    CompoundEnd,
    Byte(Name, i8),
    Short(Name, i16),
    Int(Name, i32),
    Long(Name, i64),
    Float(Name, f32),
    Double(Name, f64),
    ByteArray(Name, Vec<i8>),
    String(Name, String),
    List(Name, Tag, i32),
    ListEnd,
    Compound(Name),
    IntArray(Name, Vec<i32>),
    LongArray(Name, Vec<i64>),
}

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
    kind: ErrorKind,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Any other errors. Users should not match on this variant and should
    /// instead use a wildcard `_`. Errors in this category may be moved to new variants.
    Other,

    /// End of file. This occurs when EOF happens at the end of a tag and value,
    /// so it may not be an error, it could be the natural end of the NBT. The
    /// parser does not have enough context to tell the difference as it does
    /// not track the overall structure of the NBT.
    Eof,

    /// EOF that occurred part way through some NBT value.
    UnexpectedEof,
    InvalidTag,

    /// Expected unicode data but was not valid. Parser remains valid if just
    /// this value was not unicode. Contained bytes are the invalid unicode data.
    Nonunicode(Vec<u8>),
}

impl Error {
    /// Get the kind of error.
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.kind, ErrorKind::Eof)
    }

    fn bespoke(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            kind: ErrorKind::Other,
        }
    }

    fn invalid_tag(t: u8) -> Self {
        Self {
            msg: format!("invalid tag: {}", t),
            kind: ErrorKind::InvalidTag,
        }
    }

    fn nonunicode(d: Vec<u8>) -> Self {
        Self {
            msg: format!(
                "invalid string, non-unicode: {}",
                String::from_utf8_lossy(&d),
            ),
            kind: ErrorKind::Nonunicode(d),
        }
    }

    fn eof() -> Self {
        Self {
            msg: "EOF".into(),
            kind: ErrorKind::Eof,
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::UnexpectedEof => Self {
                msg: e.to_string(),
                kind: ErrorKind::UnexpectedEof,
            },
            // Probably want to include the IO error in future.
            _ => Self {
                msg: e.to_string(),
                kind: ErrorKind::Other,
            },
        }
    }
}

/// Convenience type for Result.
pub type Result<T> = std::result::Result<T, Error>;

/// Parser can take any reader and parse it as NBT data. Does not do decompression.
///
/// # Examples
///
/// ## Dump NBT
/// The following takes a stream of GZip compressed data from stdin and dumps it out in Rust's `Debug` format, with
/// some indentation to help see the structure.
///
/// ```
/// use fastnbt::stream::{Parser, Value};
/// use flate2::read::GzDecoder;
///
/// let stdin = std::io::stdin();
/// let decoder = GzDecoder::new(stdin);
///
/// let mut parser = Parser::new(decoder);
/// let mut indent = 0;
///
/// loop {
///     match parser.next() {
///         Err(e) => {
///             println!("{:?}", e);
///             break;
///         }
///         Ok(value) => {
///             match value {
///                 Value::CompoundEnd => indent -= 4,
///                 Value::ListEnd => indent -= 4,
///                 _ => {}
///             }
///
///             println!("{:indent$}{:?}", "", value, indent = indent);
///
///             match value {
///                 Value::Compound(_) => indent += 4,
///                 Value::List(_, _, _) => indent += 4,
///                 _ => {}
///             }
///         }
///     }
/// }
/// ```
/// ## Finding a heightmap
/// Here we assume we've parsed up until we have entered the `Heightmaps` compound of the
/// [Minecraft Anvil chunk format](https://minecraft.wiki/w/Chunk_format). We keep parsing until we find the
/// `WORLD_SURFACE` long array. We avoid entering nested compounds by skipping them if we enter one. We know we have
/// finished with the current compound when we see the `CompoundEnd` value.
///
/// ```ignore
/// use fastnbt::stream::{Parser, Value, skip_compound};
/// use fastanvil::expand_heightmap;
///
/// # use fastnbt::Result;
/// # fn f() -> Result<Option<Vec<u16>>> {
/// let mut parser = /* ... */
/// # Parser::new(&[1u8,2,3][..]);
///
/// loop {
///     match parser.next()? {
///         Value::LongArray(Some(ref name), data) if name == "WORLD_SURFACE" => {
///             skip_compound(&mut parser)?;
///             return Ok(Some(expand_heightmap(data.as_slice())));
///         }
///         Value::Compound(_) => {
///             // don't enter another compound.
///             skip_compound(&mut parser)?;
///         }
///         Value::CompoundEnd => {
///             // No heightmap found, it happens.
///             return Ok(None);
///         }
///         // also need to not enter lists
///         _ => {}
///     }
/// }
/// # }
/// ```
pub struct Parser<R: Read> {
    reader: R,
    layers: Vec<Layer>,
}

impl<R: Read> Parser<R> {
    /// Create new parser for the given reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            layers: Vec::new(),
        }
    }

    /// Parse the next value from the input.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Value> {
        self.next_inner()
    }

    /// Gets a reference to the underlying value in this parser.
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Gets a mutable reference to the underlying value in this parser.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consumes this parser, returning the underlying value.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Get the next value from the reader. Returns EOF if the stream ended sucessfully, and
    /// IO(err) for any other IO error.
    fn next_inner(&mut self) -> Result<Value> {
        let last_layer = self.layers.last().map(|l| (*l).clone());
        match last_layer {
            Some(Layer::List(_, 0)) => {
                self.layers.pop();
                return Ok(Value::ListEnd);
            }
            Some(_) => {}
            None => {}
        }

        if let Some(layer) = self.layers.last_mut() {
            match layer {
                Layer::List(_, remainder) => {
                    *remainder -= 1;
                }
                Layer::Compound => {}
            };
        }

        let last_layer = self.layers.last().map(|l| (*l).clone());
        if let Some(layer) = last_layer {
            match layer {
                Layer::List(tag, _) => return self.read_payload(tag, None),
                Layer::Compound => {}
            };
        }

        // If we get EOF reading a tag, it means we completed a tag to get here, so this is a
        // natural end of stream.
        let tag = match self.reader.read_u8() {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Err(Error::eof()),
            e => e?,
        };

        let tag = u8_to_tag(tag)?;

        if tag == Tag::End {
            // End tags have no name or value.
            let last_layer = self.layers.last().map(|l| (*l).clone());
            return match last_layer {
                Some(Layer::Compound) => {
                    self.layers.pop();
                    Ok(Value::CompoundEnd)
                }
                Some(_) => Err(Error::bespoke("expected to be in compound")),
                None => Err(Error::bespoke("expected to be in compound")),
            };
        }

        let name = Some(self.read_size_prefixed_string()?);

        self.read_payload(tag, name)
    }

    fn read_size_prefixed_string(&mut self) -> Result<String> {
        let name_len = self.reader.read_u16::<BigEndian>()? as usize;

        let mut buf = vec![0; name_len];
        self.reader.read_exact(&mut buf[..])?;

        Ok(cesu8::from_java_cesu8(&buf[..])
            .map_err(|_| Error::nonunicode(Vec::from(&buf[..])))?
            .into_owned())
    }

    fn read_payload(&mut self, tag: Tag, name: Name) -> Result<Value> {
        match tag {
            Tag::Byte => Ok(Value::Byte(name, self.reader.read_i8()?)),
            Tag::Short => Ok(Value::Short(name, self.reader.read_i16::<BigEndian>()?)),
            Tag::Int => Ok(Value::Int(name, self.reader.read_i32::<BigEndian>()?)),
            Tag::Long => Ok(Value::Long(name, self.reader.read_i64::<BigEndian>()?)),
            Tag::Float => Ok(Value::Float(name, self.reader.read_f32::<BigEndian>()?)),
            Tag::Double => Ok(Value::Double(name, self.reader.read_f64::<BigEndian>()?)),
            Tag::Compound => {
                self.layers.push(Layer::Compound);
                Ok(Value::Compound(name))
            }
            Tag::End => panic!("end tag should have returned early"),
            Tag::List => {
                let element_tag = self.reader.read_u8()?;
                let element_tag = u8_to_tag(element_tag)?;
                let size = self.reader.read_i32::<BigEndian>()?;
                self.layers.push(Layer::List(element_tag, size));
                Ok(Value::List(name, element_tag, size))
            }
            Tag::String => Ok(Value::String(name, self.read_size_prefixed_string()?)),
            Tag::ByteArray => {
                let size = self.reader.read_i32::<BigEndian>()?;
                let mut buf = vec![0u8; size as usize];
                self.reader.read_exact(&mut buf[..])?;
                Ok(Value::ByteArray(name, vec_u8_into_i8(buf)))
            }
            Tag::IntArray => {
                let size = self.reader.read_i32::<BigEndian>()?;
                let mut buf = vec![0i32; size as usize];
                for i in 0..size {
                    buf[i as usize] = self.reader.read_i32::<BigEndian>()?;
                }

                Ok(Value::IntArray(name, buf))
            }
            Tag::LongArray => {
                let size = self.reader.read_i32::<BigEndian>()?;
                let mut buf = vec![0i64; size as usize];
                for i in 0..size {
                    buf[i as usize] = self.reader.read_i64::<BigEndian>()?;
                }

                Ok(Value::LongArray(name, buf))
            }
        }
    }
}

/// Parse the input until the compound we are currently inside is complete.
/// Handles inner compounds by skipping those as well.
pub fn skip_compound<R: Read>(parser: &mut Parser<R>) -> Result<()> {
    let mut depth = 1;

    while depth != 0 {
        let value = parser.next()?;
        match value {
            Value::CompoundEnd => depth -= 1,
            Value::Compound(_) => depth += 1,
            _ => {}
        }
    }
    Ok(())
}

/// Parse until the compound with the given name is found. This will enter other
/// compounds and lists, rather than find a compound at the current level.
pub fn find_compound<R: Read>(parser: &mut Parser<R>, name: Option<&str>) -> Result<()> {
    loop {
        match parser.next()? {
            //Value::Compound(n) if n == name.map(|s| s.to_owned()) => break,
            Value::Compound(n) if n.as_deref() == name => break,
            _ => {}
        }
    }
    Ok(())
}

/// Parse until the list with the given name is found. This will enter other
/// compounds and lists, rather than find a list at the current level.
pub fn find_list<R: Read>(parser: &mut Parser<R>, name: Option<&str>) -> Result<usize> {
    loop {
        match parser.next()? {
            Value::List(n, _, size) if n.as_deref() == name => return Ok(size as usize),
            _ => {}
        }
    }
}

// Thanks to https://stackoverflow.com/a/59707887
fn vec_u8_into_i8(v: Vec<u8>) -> Vec<i8> {
    // ideally we'd use Vec::into_raw_parts, but it's unstable,
    // so we have to do it manually:

    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = std::mem::ManuallyDrop::new(v);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

fn u8_to_tag(tag: u8) -> Result<Tag> {
    Tag::try_from(tag).map_err(|_| Error::invalid_tag(tag))
}

#[derive(Clone)]
enum Layer {
    List(Tag, i32),
    Compound,
}
